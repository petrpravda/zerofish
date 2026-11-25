#!/usr/bin/env bash
set -euo pipefail

if [[ $# != 2 ]]; then
    echo "Usage: bash $0 VERSION /path/to/zerofish.exe" >&2
    exit 2
fi

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
exe=$(realpath -- "$2")
test -f "$exe"
runtime=${CONTAINER_RUNTIME:-podman}
image=localhost/zerofish-release-wine:debian13
if ! "$runtime" image inspect "$image" >/dev/null 2>&1; then
    "$runtime" build -t "$image" -f "$script_dir/Containerfile.wine" "$script_dir"
fi

"$runtime" run --rm --network=none --env WINEPREFIX=/tmp/zerofish-wine \
    --volume "$script_dir/smoke.py:/smoke.py:ro" \
    --volume "$exe:/zerofish.exe:ro" \
    "$image" sh -c '
        # Run the display wrapper as a child so it can receive the X server readiness signal.
        xvfb-run -a python3 /smoke.py "$1" /usr/lib/wine/wine64 /zerofish.exe
        exit "$?"
    ' _ "$1"
