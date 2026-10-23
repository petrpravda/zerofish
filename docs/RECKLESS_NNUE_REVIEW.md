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

Generic techniques such as nonzero-index compaction and independent VNNI chains
could be adapted to Zerofish's Stockfish-shaped kernels. Such an experiment would
be a new Stockfish-compatible implementation inspired by those techniques, not
running Reckless's evaluator. It was not added here because the requested trial
was conditional on the architectures being the same. No Reckless source was
copied into the engine and no network, kernel or benchmark behavior was changed.

## Benchmark disposition

The [existing benchmark table](RUST_NNUE.md#measured-comparison-on-2026-10-22)
now records Reckless as **N/A: incompatible network** for full and incremental
inference. Its existing Rust/scalar/library timings remain unchanged. Timing
Reckless's smaller, differently encoded network alongside these numbers would
not measure SIMD algorithms doing the same evaluation work.

Review metadata and SHA-256 hashes of the inspected upstream files are saved in
`target/nnue/reckless-review-20261023/architecture.json`. This was a source and
documentation review; no engine or timing tests needed to be rerun.
