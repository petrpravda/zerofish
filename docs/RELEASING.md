# Build and verify a Zerofish release

Run from the repository on an x86-64 Linux host with Podman, Python 3, curl, tar,
zip, and unzip. Docker also works with `CONTAINER_RUNTIME=docker`. Both images
are built on demand and reused for later releases. Rebuild from their
Containerfiles when the dependencies change.

```sh
bash scripts/release/build.sh
```

The script uses the pinned Stockfish weights from the adjacent Stockfish checkout
or downloads them from Stockfish Fishtest. You can pass a path to the `.nnue` as
the first argument. When building from a published `-source.tar.gz` archive, pass
`release-inputs/nn-c9e480e92425.nnue`. The build checks the full SHA-256 against
`native/UPSTREAM.json` before compiling.

The output archives are under `target/release-work/dist/`. Linux packages use
glibc 2.35 as the baseline. The x86-64 binaries select AVX-512/VNNI at runtime
where supported and use scalar NNUE elsewhere. ARM64 uses scalar NNUE. Windows
uses the Rust GNU target with MinGW-w64. Release packages embed the default
network and can run offline from any working directory.

The script runs the Rust tests, verifies the unpacked Linux x86-64 binary on the
host and under a QEMU Haswell CPU, tests the unpacked ARM64 binary under QEMU,
inspects Windows DLL imports, and tests the exact Windows executable under Wine.
It runs a UCI handshake, NNUE evaluation, and short search with no network file
or environment override. To repeat the Windows check on its own, extract the
Windows ZIP and run the following command with the matching version and path.

```sh
bash scripts/release/test-windows.sh 0.1.0 /path/to/zerofish.exe
```

## Publish

Tag the tested source with the Cargo package version. Ship the three binary
archives, their SHA-256 manifest, and the corresponding source. The command
below adds the verified network input to the source archive from the tag and
generates the manifest. Binary archives include the GPL and third-party notices.

```sh
bash scripts/release/package-source.sh v0.1.0
```

Stage the assets in a GitHub draft, verify the manifest, and publish the release.
