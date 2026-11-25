#!/usr/bin/env bash
set -euo pipefail

if [[ $# != 1 || ! $1 =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Usage: bash $0 vVERSION" >&2
    exit 2
fi

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.."
work=$PWD/target/release-work
version=${1#v}
prefix=zerofish-$version
stage=$(mktemp -d "$work/source.XXXXXXXX")
trap 'rm -rf -- "$stage"' EXIT
git archive --prefix="$prefix/" "refs/tags/$1" | tar -x -C "$stage"
read -r net_name net_sha < <(python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d["default_network"], d["network_sha256"])' "$stage/$prefix/native/UPSTREAM.json")
network=$work/nnue/$net_name
printf '%s  %s\n' "$net_sha" "$network" | sha256sum --check
mkdir -- "$stage/$prefix/release-inputs"
cp -- "$network" "$stage/$prefix/release-inputs/"
cp -- "$stage/$prefix/native/COPYING.Stockfish" "$stage/$prefix/COPYING"
tar -C "$stage" -czf "$work/dist/$prefix-source.tar.gz" "$prefix"

cd -- "$work/dist"
sha256sum "$prefix-linux-x86_64.tar.gz" "$prefix-linux-aarch64.tar.gz" \
    "$prefix-windows-x86_64.zip" "$prefix-source.tar.gz" > SHA256SUMS
sha256sum --check SHA256SUMS
