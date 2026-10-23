# Reckless NNUE architecture review — 2026-10-23

Reckless does **not** use the architecture of the Stockfish network currently
embedded in Zerofish. Its network and SIMD evaluator cannot be substituted into
the existing same-network benchmark without changing feature encoders, parameters,
layer shapes and numerical semantics.

Reviewed repository: [codedeliveryservice/Reckless](https://github.com/codedeliveryservice/Reckless),
revision [`ffe556dcfd38e1ac29f3c554137b69ee0ecf7904`](https://github.com/codedeliveryservice/Reckless/commit/ffe556dcfd38e1ac29f3c554137b69ee0ecf7904)
(main HEAD when checked; commit dated 2026-10-22). The review reads the source at
this exact revision, rather than inferring architecture from engine descriptions.

## Architecture comparison

Reckless's constants and parameter types are defined in
[`src/nnue.rs`](https://github.com/codedeliveryservice/Reckless/blob/ffe556dcfd38e1ac29f3c554137b69ee0ecf7904/src/nnue.rs).
Its scalar inference implementation makes the arithmetic explicit in
[`forward/scalar.rs`](https://github.com/codedeliveryservice/Reckless/blob/ffe556dcfd38e1ac29f3c554137b69ee0ecf7904/src/nnue/forward/scalar.rs).
Stockfish's pinned contracts are in this repository's
[native architecture](../native/stockfish/nnue/nnue_architecture.h),
[transformer](../native/stockfish/nnue/nnue_feature_transformer.h) and
[feature encoders](../native/stockfish/nnue/features/).

| Component | Reckless at reviewed revision | Our pinned Stockfish network |
|---|---|---|
| Piece-square inputs | 10 king buckets × 768 = 7,680 rows | K32Q2: 45,056 rows, 32 king buckets × two opponent-queen states |
| Threat inputs | 66,864 rows; includes different pawn/king interactions | FullThreats: 59,808 rows |
| Separate pawn-pair inputs | None | PP_3Wide: 4,560 rows |
| Transformer weights | Piece-square `i16`; threats `i8` | Piece-square, threats and pawn pairs all `i8` |
| Accumulators | Separate piece-square/threat `i16` accumulators, each 768 wide per perspective | Combined `i16` accumulator, 1,024 wide per perspective |
| Transformer activation | Clipped paired products; 384 outputs per perspective, concatenated to 768 | Clipped paired products; 512 outputs per perspective, concatenated to 1,024 |
| Output heads/stacks | Eight, selected by piece-count lookup | 32, selected by piece count and both sides' queen presence |
| Head dimensions | `768 → 16 → 32 → 1` | `1024 → 32 → 32 → 1`, with concatenated activations and skip term |
| Later arithmetic | First affine accumulates integer products, then dequantizes to `f32`; subsequent layers use `f32` FMA and clipped linear activations | Quantized integer affines and paired squared/linear clipped activations |
| Parameter loading | Reckless-specific embedded parameter structure | Versioned Stockfish file, feature/architecture hashes and exact pinned digest |

The networks share the broad NNUE approach and paired-product transformer
activation, but these differences prevent weight-file or evaluator compatibility.
In particular, matching a SIMD instruction such as `vpdpbusd` does not make the
network contracts interchangeable.

## SIMD logic reviewed

The implementation is in
[`forward/vectorized.rs`](https://github.com/codedeliveryservice/Reckless/blob/ffe556dcfd38e1ac29f3c554137b69ee0ecf7904/src/nnue/forward/vectorized.rs)
and [`simd/avx512.rs`](https://github.com/codedeliveryservice/Reckless/blob/ffe556dcfd38e1ac29f3c554137b69ee0ecf7904/src/nnue/simd/avx512.rs).
It combines the separate piece-square/threat accumulators during activation,
uses shifted signed multiply-high plus packing to form the paired-product output,
and prepares indices for nonzero four-byte input groups. The VNNI first affine
processes pairs of those groups through independent accumulation chains before
combining them. Later layers execute floating-point FMA with defined reduction
order. Its accumulator stack also performs lazy per-perspective updates and uses
a piece-square refresh cache.

## Stockfish-compatible follow-up

The follow-up request implemented these portable design techniques while retaining
the Stockfish network:

- `Kernel::Avx512SparseVnni` uses shifted multiply-high and unsigned packing for
  the transformer, restoring canonical neuron order after lane-local packing.
- AVX-512 nonzero masks and `vpcompressd` produce an ascending list of nonzero
  four-byte groups. This adapts the sparse-index idea using native 32-bit index
  compaction instead of Reckless's small lookup table.
- Pairs of those groups feed independent VNNI accumulation chains; the chains are
  combined once, with explicit handling of an odd final group.

The implementation is independent Rust code; Reckless's sources and network are
not copied. Stockfish feature encoders, quantization, output stacks, integer
activations, skip connection and score calibration remain unchanged. The existing
incremental delta machinery and accumulator-update kernel are shared between the
baseline and sparse variants. The sparse variant is now the AVX-512 default;
`Kernel::Avx512Vnni` retains the previous implementation for comparisons.

The [benchmark table](RUST_NNUE.md#measured-comparison-on-2026-10-23) now measures
this **Stockfish-compatible Reckless-style variant**, alongside the baseline,
scalar evaluator and linked reference, using identical weights and positions.
These are not timings of Reckless's incompatible native network. Both Rust SIMD
variants pass 10,491 reference transition checks each, and the timing harness
validates all 6,033 corpus positions before measurement.

Original review metadata and upstream file hashes remain in
`target/nnue/reckless-review-20261023/architecture.json`. Follow-up tests, timings,
frozen binaries and metadata are under
`target/nnue/reckless-style-avx512-20261023/`.
