# Zerofish

Rust chess engine port following the feature progression in `../javafish`.
The commit history introduces board rules, search heuristics, NNUE, parallel
search, UCI, SAN/PGN and Chess960 in separate steps.

```sh
cargo test --offline
cargo build --release --offline
./target/release/zerofish
```

The executable speaks UCI. Use `uci`, `isready`, `position startpos`,
`go depth 8`, `stop` and `quit` to control it.

The default evaluator implements the pinned Stockfish NNUE architecture directly
in Rust: feature encoding, network loading, incremental accumulators, inference
and score calibration. Stockfish-style AVX-512BW/VNNI kernels, using paired
weight widening, fused incremental updates, vector nonzero compaction and
independent dot-product chains, are selected at runtime on
supported CPUs, with a scalar fallback. It does not link or call the Stockfish NNUE library.

Builds read `../stockfish/src/nn-c9e480e92425.nnue` by default. Set
`ZEROFISH_STOCKFISH_NETWORK` to another location of that exact network if needed.
The build checks its SHA-256 before embedding it; the Rust loader checks the
version, architecture hashes, lengths and end of file. See
[native/UPSTREAM.json](native/UPSTREAM.json) for the pinned source/network and
[native/COPYING.Stockfish](native/COPYING.Stockfish) for the GPLv3 license.

The original 8-bucket evaluator remains available with `--no-default-features`,
including its AVX2/SSE2 kernels and king-bucket refresh cache. That configuration
uses the bundled `resources/weights-original.nnue`.

The linked Stockfish evaluator is retained as an optional comparison oracle:

```sh
scripts/build_stockfish_nnue.sh
ZEROFISH_NNUE_LIBRARY="$PWD/target/stockfish-nnue/libzerofish_nnue.so" \
  cargo run --release --offline --features stockfish-nnue --example nnue_compare -- 5
```

An optional second argument selects a six-field FEN/EPD corpus, for example the
previous match's `target/elo/stockfish-1t-5s-50pairs/match/openings.epd`.
The comparison first checks full and incremental accumulators/scores, then times
both paths in baseline Rust SIMD, the Stockfish-compatible Reckless-style sparse
variant, the new Stockfish-style Rust kernel, Rust scalar and the linked library. Enabling the
`stockfish-nnue` comparison feature does not select library inference for search.

See [the NNUE Elo-parity plan and confirmed noninferiority result](docs/STOCKFISH_NNUE_ELO_PARITY_PLAN.md),
[Vulkan training and the trained-network match](docs/VULKAN_NNUE_TRAINING.md),
[the 100-game network comparison](docs/BENCHMARK_NETWORKS_100.md),
[the Rust/library comparison](docs/RUST_NNUE.md),
[the new SIMD 40-game Elo benchmark](docs/BENCHMARK_RUST_SIMD_40.md),
[the earlier 100-game benchmark](docs/BENCHMARK.md),
[the feature progression](docs/FEATURE_MAP.md) and [Chess960 notes](docs/chess960.md).
Java training and Elo-runner modules are outside this engine port.
