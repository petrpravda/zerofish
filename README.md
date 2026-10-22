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

NNUE uses the bundled `resources/weights-original.nnue` with incremental
accumulators, king-bucket refresh caching and runtime-selected AVX2/SSE2
kernels; other platforms use scalar evaluation kernels.

The Rust port includes the saved Rust checkpoint improvements. Java training
and Elo-runner modules are outside this engine port. See
[the feature progression](docs/FEATURE_MAP.md) and [Chess960 notes](docs/chess960.md).
