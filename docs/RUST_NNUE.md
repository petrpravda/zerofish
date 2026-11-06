# Direct Rust NNUE and linked-library comparison

The default engine now evaluates the Stockfish network entirely in Rust. It does
not call or link `libzerofish_nnue.so`. The C++ implementation remains an optional
oracle under the `stockfish-nnue` comparison feature; search still uses Rust when
that feature is enabled. The original SCReLU evaluator remains available through
`--evaluation-backend zerofish-nnue`, with its existing AVX2/SSE2 and refresh-cache improvements.

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
  the first affine layer and both later affine layers. The default
  `Kernel::Avx512Stockfish` adds paired signed-byte widening, explicit
  16-register tiles, fused parent-to-child updates, fused activation/nonzero
  gathering, and three independent VNNI chains. Propagation runs inside one
  AVX-512 function so the layers inline without repeated SIMD call boundaries.
  The earlier `Kernel::Avx512Vnni` and Reckless-style
  `Kernel::Avx512SparseVnni` remain selectable for comparisons.
  A scalar implementation remains for other CPUs and differential checks.
- `mod.rs`: aligned per-ply accumulators and sibling-safe position validation.
  The default directly emits dirty piece, threat-edge and pawn-pair indices,
  avoiding sorting/merging unchanged features, then updates only changed rows. Discovered slider attacks, captures,
  en passant, promotions, Chess960 castling, king orientation changes and opponent
  queen-presence changes are included. Null transitions preserve the accumulators.

The network is an external file loaded at runtime. The Rust loader checks its
SHA-256, version, feature-transformer hash, all stack hashes, exact lengths and EOF. Default network path:
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

- 10,491 transition checks per SIMD variant compare Rust scalar/full, Rust SIMD/incremental,
  linked/full and linked/incremental raw scores. Both 1,024-neuron accumulators,
  centipawn scores and exact internal rule-50 scaling also agree.
- A separate timing corpus checks 6,033 evaluated positions over 64 seeded legal
  walks, rooted at startpos and 15 EPD positions from the earlier match.
- Special cases include null moves, sibling rewinds, captures, underpromotions,
  en passant, orthodox castling and Chess960 castling geometries.
- SIMD/scalar boundary checks cover signed-short accumulator overflow, negative
  and extreme activation inputs, every output stack and randomized affine inputs.
  Sparse-list tests also cover empty, single, odd/even, vector-boundary and dense
  lists through all 256 input groups, with high-bit bytes and shuffled positions.
- Invalid-board/feature-index checks run before SIMD memory reads, with regression
  coverage for pawns on promotion ranks and out-of-range network rows.
- Full suites passed: 70 tests for the direct-Rust default, 74 with the linked
  oracle enabled, and 66 for the preserved legacy evaluator.
- `ldd target/release/zerofish` shows only normal runtime libraries; no NNUE or
  C++ library is linked. Disassembly contains actual ZMM `vpdpbusd` instructions.

These are exact agreements on the tested corpus, not a claim of exhaustive proof
for every possible board. The optional library's previous independent executable
oracle tests remain in the combined suite.

## Measured comparison on 2026-10-23

CPU: AMD Ryzen 7 8845HS (AVX-512BW/VNNI). Compiler: rustc 1.93.0. Rust release
profile uses opt-level 3, fat LTO and one codegen unit; the reference library uses
the existing native `-O3 -march=native` build. One evaluation stream at a time.

Median of seven batches, five repetitions of all 6,033 positions per batch.
All five backends were remeasured together on 2026-10-23:

| Path | Baseline Rust VNNI | Reckless-style Rust VNNI | Stockfish-style Rust VNNI | Rust scalar | Linked reference |
|---|---:|---:|---:|---:|---:|
| Full rebuild + inference | 3.540 µs | 3.429 µs | 2.865 µs | 17.192 µs | 4.319 µs |
| Incremental push + inference | 2.715 µs | 2.579 µs | 1.490 µs | 13.443 µs | 1.414 µs |

A longer run (15 repetitions per batch) measured 2.939 µs full rebuild and
1.437 µs incremental for the new kernel versus 4.303 µs and 1.441 µs for the
library. Incremental evaluation reached parity within the observed timing noise;
this is not evidence of a meaningful incremental speed advantage. The full
rebuild includes different board setup costs, described below.

The "Reckless-style" column uses our Stockfish features, weights and integer
layers throughout. It adapts sparse input indexing, paired independent VNNI
chains and multiply-high/packing from the reviewed techniques; it does not run
Reckless's incompatible network or float layers. The review is pinned at
`ffe556dcfd38e1ac29f3c554137b69ee0ecf7904`; see the
[architecture and SIMD review](RECKLESS_NNUE_REVIEW.md).

Timings include feature encoding/updates and propagation, not just dot products.
The library's full-rebuild path also includes its FEN/native-position reset and
invalidation of the two selected Finny entries. It starts accumulation from biases,
just like Rust; it cannot use cached pieces from a previous position. Incremental
measurements retain each backend's normal per-walk state; root resets are amortized
over at most 96 moves. Network loading, table initialization, construction and
corpus validation are excluded, with one untimed warmup per backend/mode. Backend
order rotates between batches. No CPU affinity or fixed frequency was imposed.
These are evaluator timings, not search-NPS or Elo measurements. Rust derives
dirty edges from the two supplied board states; the reference records them during
its native move. The native PSQ Finny cache remains specific to the reference.

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
cargo test --locked --offline
```

Current artifacts are under `target/nnue/stockfish-parity-20261023/`: test
logs, timing output, corpus, build/dependency/instruction evidence, metadata and
frozen binaries. The earlier comparison remains archived under
`target/nnue/rust-avx512-vnni-20261022/`; previous Elo engines in `target/elo/`
remain unchanged.

## Native source review and adaptation

The optional library builds the vendored source at Stockfish commit
`effdc3fd3f6de6e75fe1374a51a7a7eed66f880b`; it contains the actual implementation,
not an opaque external evaluator. Relevant files are
`native/stockfish/nnue/nnue_accumulator.cpp`, `nnue_feature_transformer.h`,
`layers/affine_transform_sparse_input.h`, `layers/affine_transform.h`, and
`features/{full_threats,pp_3wide,k32q2}.cpp`.

The Rust port applies the following approaches:

- Native accumulator tiling and source-to-destination updates remove a separate
  accumulator copy. Explicit register names avoid LLVM spilling an indexed
  16-vector array. Two signed-byte rows are interleaved and summed with
  `vpmaddubsw`, then added/subtracted with wrapping 16-bit arithmetic.
- At load time, each transformer weight cache line is rearranged in eight-byte
  blocks `[0,4,1,5,2,6,3,7]`. This makes paired byte unpacking produce canonical
  neuron order. Scalar and older kernels lazily retain their canonical layout;
  a normal engine initializes only its selected layout.
- Exact dirty threat edges and pawn pairs replace sorted affected-feature lists.
  Emission prefetches the changed row; this CPU measured better with T0 prefetch
  across the row than with only the native-style first-line T2 prefetch.
- Activation packs the multiplied halves while producing nonzero four-byte group
  indices. Register compression followed by a normal store avoids the expensive
  compress-store instruction on Zen 4. POPCNT is explicitly enabled and detected.
- Three independent VNNI chains feed both hidden affine layers; a single SIMD
  propagation function allows inlining across their activation boundaries.

Experiments with eight-register tiles and separately indexed vector arrays were
slower and were discarded. Tests cover paired odd/even row counts, in-place and
parent-to-child updates, wrapping overflow, all output stacks, and empty through
dense sparse lists. The network digest and all tested integer outputs are unchanged.

## Training-compatible network builds

The Maven module in `../javafish/nnue-training` now provides all-layer Vulkan
fine-tuning for this exact architecture. See [the training and match report](VULKAN_NNUE_TRAINING.md).
For a derived network, set both `ZEROFISH_STOCKFISH_NETWORK` and its expected
`ZEROFISH_STOCKFISH_NETWORK_SHA256` when running the engine. The default digest remains
pinned to the original network. All architecture, stack-hash, length and EOF checks
remain active. One engine binary can load either pinned network at startup.

Network files are loaded at runtime, never embedded. Evaluator modules are explicitly
`zerofish_nnue` and `stockfish_rust`; `nnue` provides the search-facing backend wrapper.
Use native `--evaluation-backend` and `--network` flags rather than Cargo feature selection.
