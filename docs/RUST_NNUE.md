# Direct Rust NNUE and linked-library comparison

The default engine now evaluates the Stockfish network entirely in Rust. It does
not call or link `libzerofish_nnue.so`. The C++ implementation remains an optional
oracle under the `stockfish-nnue` comparison feature; search still uses Rust when
that feature is enabled. The original SCReLU evaluator remains available through
`--no-default-features`, with its existing AVX2/SSE2 and refresh-cache improvements.

## Implementation

The implementation is in `src/stockfish_rust/`:

- `features.rs`: K32Q2 piece-square features, FullThreats attack/defence features
  and PP_3Wide pawn pairs, with the pinned orientation/indexing rules.
- `network.rs`: little-endian network reader, version/architecture/length checks,
  64-byte-aligned weights, 32 output stacks, SCReLU activations, skip connection,
  material/correlation scaling and centipawn calibration.
- `simd.rs`: direct `std::arch::x86_64` intrinsics, dispatched at runtime.
  AVX-512BW widens signed byte weights into wrapping 16-bit accumulator updates
  and transforms paired accumulator halves. AVX-512 VNNI `vpdpbusd` implements
  the sparse first affine layer and both later affine layers. Squared/linear
  activations also execute in AVX-512. A scalar implementation is retained for
  other CPUs and for differential checks.
- `mod.rs`: aligned per-ply accumulators and sibling-safe position validation.
  Incremental updates re-encode affected pieces, attackers and pawn pairs, then
  add/remove only changed weight rows. Discovered slider attacks, captures,
  en passant, promotions, Chess960 castling, king orientation changes and opponent
  queen-presence changes are included. Null transitions preserve the accumulators.

The network is embedded at build time. Its SHA-256 is checked before compilation;
version, feature-transformer hash, all stack hashes, exact lengths and EOF are
checked again by the Rust loader. Default network path:
`../stockfish/src/nn-c9e480e92425.nnue`; override its location with
`ZEROFISH_STOCKFISH_NETWORK`. The expected digest is:

```text
c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73
```

Architecture/source provenance and license remain in `native/UPSTREAM.json` and
`native/COPYING.Stockfish`. The reference bridge adds diagnostic APIs for exporting
accumulators in canonical neuron order and invalidating only the two selected
Finny entries before an uncached full rebuild.

## Correctness checks

- 10,491 transition checks compare Rust scalar/full, Rust SIMD/incremental,
  linked/full and linked/incremental raw scores. Both 1,024-neuron accumulators,
  centipawn scores and exact internal rule-50 scaling also agree.
- A separate timing corpus checks 6,033 evaluated positions over 64 seeded legal
  walks, rooted at startpos and 15 EPD positions from the earlier match.
- Special cases include null moves, sibling rewinds, captures, underpromotions,
  en passant, orthodox castling and Chess960 castling geometries.
- SIMD/scalar boundary checks cover signed-short accumulator overflow, negative
  and extreme activation inputs, every output stack and randomized affine inputs.
- Invalid-board/feature-index checks run before SIMD memory reads, with regression
  coverage for pawns on promotion ranks and out-of-range network rows.
- Full suites passed: 68 tests for the direct-Rust default, 72 with the linked
  oracle enabled, and 66 for the preserved legacy evaluator.
- `ldd target/release/zerofish` shows only normal runtime libraries; no NNUE or
  C++ library is linked. Disassembly contains actual ZMM `vpdpbusd` instructions.

These are exact agreements on the tested corpus, not a claim of exhaustive proof
for every possible board. The optional library's previous independent executable
oracle tests remain in the combined suite.

## Measured comparison on 2026-10-22

CPU: AMD Ryzen 7 8845HS (AVX-512BW/VNNI). Compiler: rustc 1.93.0. Rust release
profile uses opt-level 3, fat LTO and one codegen unit; the reference library uses
the existing native `-O3 -march=native` build. One evaluation stream at a time.

Median of seven batches, five repetitions of all 6,033 positions per batch:

| Path | Rust AVX-512/VNNI | Rust scalar | Linked reference | Reckless |
|---|---:|---:|---:|---|
| Full rebuild + inference | 3.845 µs | 17.837 µs | 4.639 µs | N/A: incompatible network |
| Incremental push + inference | 2.726 µs | 13.971 µs | 1.518 µs | N/A: incompatible network |

Reckless was reviewed on 2026-10-23 at revision
`ffe556dcfd38e1ac29f3c554137b69ee0ecf7904`. Its feature encoding, dimensions,
weights and later-layer arithmetic differ from this Stockfish network, so its
evaluator cannot be inserted into this same-network benchmark. No Reckless
timing was measured; the original measurements above are unchanged. See the
[architecture and SIMD review](RECKLESS_NNUE_REVIEW.md).

Rust SIMD was about 17% faster than the linked full-rebuild path in this harness.
For incremental evaluation it was about 1.80 times the library's cost; the linked
implementation remains faster. Relative to Rust scalar, SIMD was 4.64 times
faster for full rebuilds and 5.13 times faster for incremental evaluation.
The two modes and all backends produced the same checksum, 10,283,525, in every
timed batch.

Timings include feature encoding/updates and propagation, not just dot products.
The library's full-rebuild path also includes its FEN/native-position reset and
invalidation of the two selected Finny entries. It starts accumulation from biases,
just like Rust; it cannot use cached pieces from a previous position. Incremental
measurements retain each backend's normal per-walk state; root resets are amortized
over at most 96 moves. Network loading, table initialization, construction and
corpus validation are excluded, with one untimed warmup per backend/mode. Backend
order rotates between batches. No CPU affinity or fixed frequency was imposed.
These are evaluator timings, not search-NPS or Elo measurements. Rust still
reconstructs affected attack/pair sets from board states, while the reference
uses native dirty-feature bookkeeping; there is room to improve Rust incremental
performance further.

## Reproduce

```sh
# Standalone engine: no native compiler/library needed.
cargo build --release --offline
cargo test --offline

# Optional library, comparisons and the complete differential test suite.
scripts/build_stockfish_nnue.sh
export ZEROFISH_NNUE_LIBRARY="$PWD/target/stockfish-nnue/libzerofish_nnue.so"
export STOCKFISH_BINARY="$PWD/../stockfish/src/stockfish"
cargo test --offline --features stockfish-nnue
cargo run --release --offline --features stockfish-nnue --example nnue_compare -- \
  5 target/elo/stockfish-1t-5s-50pairs/match/openings.epd

# Preserved original evaluator.
cargo test --offline --no-default-features
```

Artifacts are under `target/nnue/rust-avx512-vnni-20261022/`: test logs, timing
output, corpus, build/dependency/instruction evidence, metadata and frozen binaries.
The previous Elo-match engines in `target/elo/` remain unchanged.
