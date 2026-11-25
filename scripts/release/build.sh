#!/usr/bin/env bash
set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.."
repo=$PWD
runtime=${CONTAINER_RUNTIME:-podman}
image=localhost/zerofish-release-build:rust1.93-ubuntu22.04
work=$repo/target/release-work
mkdir -p "$work/cargo" "$work/nnue" "$work/dist"
read -r net_name net_sha < <(python3 -c 'import json; d=json.load(open("native/UPSTREAM.json")); print(d["default_network"], d["network_sha256"])')
network=$work/nnue/$net_name
source_network=${1:-$repo/../stockfish/src/$net_name}
if [[ -f "$source_network" ]]; then
    if [[ ! "$source_network" -ef "$network" ]]; then
        cp -- "$source_network" "$network"
    fi
elif [[ $# != 0 ]]; then
    echo "Cannot read $source_network" >&2
    exit 2
elif [[ ! -f "$network" ]]; then
    curl --fail --show-error --silent --location --retry 3 \
        "https://tests.stockfishchess.org/api/nn/$net_name" -o "$network.part"
    mv -- "$network.part" "$network"
fi
printf '%s  %s\n' "$net_sha" "$network" | sha256sum --check

if ! "$runtime" image inspect "$image" >/dev/null 2>&1; then
    "$runtime" build -t "$image" -f scripts/release/Containerfile.build scripts/release
fi
container=()
if [[ "$runtime" == podman ]]; then
    container+=(--userns=keep-id)
else
    container+=(--user "$(id -u):$(id -g)")
fi
container+=(--rm --volume "$repo:/src:ro" --volume "$work:/release" --workdir /src
    --env CARGO_HOME=/release/cargo --env CARGO_TARGET_DIR=/release/cargo-target
    --env "ZEROFISH_EMBED_STOCKFISH_NETWORK=/release/nnue/$net_name"
    --env CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
    --env CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
    --env 'CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS=-C link-arg=-static')

version=$("$runtime" run "${container[@]}" "$image" cargo metadata --locked --no-deps --format-version 1 |
    python3 -c 'import json,sys; print(json.load(sys.stdin)["packages"][0]["version"])')

"$runtime" run "${container[@]}" "$image" bash -eu -c '
    cargo test --locked
    for target in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-gnu; do
        cargo build --locked --release --bin zerofish --target "$target"
    done
'

stage=$(mktemp -d "$work/package.XXXXXXXX")
trap 'rm -rf -- "$stage"' EXIT
targets=(x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-pc-windows-gnu)
platforms=(linux-x86_64 linux-aarch64 windows-x86_64)
for i in "${!targets[@]}"; do
    name=zerofish-$version-${platforms[i]}
    bin=zerofish
    if [[ ${platforms[i]} == windows-* ]]; then
        bin=zerofish.exe
    fi
    mkdir -- "$stage/$name"
    cp -- "$work/cargo-target/${targets[i]}/release/$bin" "$stage/$name/$bin"
    cp -- native/COPYING.Stockfish "$stage/$name/COPYING"
    cp -- scripts/release/NOTICE.txt "$stage/$name/NOTICE.txt"
    if [[ $bin == *.exe ]]; then
        (cd -- "$stage" && zip -q -r "$stage/$name.zip" "$name")
        mv -- "$stage/$name.zip" "$work/dist/"
        unzip -q "$work/dist/$name.zip" -d "$stage/verified"
    else
        tar -C "$stage" -czf "$work/dist/$name.tar.gz" "$name"
        mkdir -p "$stage/verified"
        tar -C "$stage/verified" -xzf "$work/dist/$name.tar.gz"
    fi
done

linux_x64=$stage/verified/zerofish-$version-linux-x86_64/zerofish
linux_arm=$stage/verified/zerofish-$version-linux-aarch64/zerofish
windows=$stage/verified/zerofish-$version-windows-x86_64/zerofish.exe
python3 scripts/release/smoke.py "$version" "$linux_x64"
"$runtime" run "${container[@]}" --network=none "$image" bash -euo pipefail -c '
    python3 scripts/release/smoke.py "$1" qemu-x86_64 -cpu Haswell "$2"
    python3 scripts/release/smoke.py "$1" qemu-aarch64 -L /usr/aarch64-linux-gnu "$3"
    x86_64-w64-mingw32-objdump -p "$4" | awk '\''
        /DLL Name:/ {
            print
            dll=tolower($3)
            if (dll ~ /^(libgcc|libstdc\+\+|libwinpthread)/) bad=1
        }
        END { exit bad }
    '\''
' _ "$version" "${linux_x64/#$work//release}" "${linux_arm/#$work//release}" "${windows/#$work//release}"
bash scripts/release/test-windows.sh "$version" "$windows"
printf 'Verified release archives in %s/dist\n' "$work"
