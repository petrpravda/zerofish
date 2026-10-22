# Rust engine progression

This port follows the feature order of `../javafish`, using the saved Rust
checkpoints as its implementation base. Each commit introduces the next feature.

1. Establish Rust board state, legal moves, FEN and perft
2. Describe the Rust engine feature progression
3. Introduce fixed-depth negamax with classical scores
4. Bound negamax with alpha and beta

The neural evaluation stages retain runtime-selected AVX2/SSE2 kernels,
incremental accumulators and king-bucket refresh caching.
