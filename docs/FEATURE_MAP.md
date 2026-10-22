# Rust engine progression

This port follows the feature order of `../javafish`, using the saved Rust
checkpoints as its implementation base. Each commit introduces the next feature.

1. Establish Rust board state, legal moves, FEN and perft
2. Describe the Rust engine feature progression
3. Introduce fixed-depth negamax with classical scores
4. Bound negamax with alpha and beta
5. Deepen the search one completed iteration at a time
6. Honor stop requests and search deadlines
7. Resolve tactical leaves with quiescence
8. Recognize repeated positions and rule draws in search
9. Cache search bounds and prioritize the stored move
10. Rank tactical moves by material gain
11. Remember quiet cutoff moves as killers
12. Update quiet ordering from search history
13. Accumulate capture ordering statistics
14. Condition ordering on the preceding move
15. Use exchange analysis to rank tactics
16. Produce ordered moves in lazy stages
17. Probe nonprincipal moves with narrow windows
18. Retry iterations with expanding score windows
19. Extend checked positions within a path budget
20. Reduce late moves and verify promising results
21. Store raw evaluation alongside transposition bounds
22. Cut shallow nodes with reverse futility margins
23. Confirm low evaluations through tactical search
24. Try null moves with material guards and verification
25. Reduce search depth when hash guidance is absent
26. Stop searching sufficiently late quiet candidates
27. Discard quiet moves below futility margins
28. Filter shallow quiets with weak history scores
29. Apply depth-sensitive exchange pruning
30. Bound quiescence gains with delta margins
31. Exclude losing exchanges from tactical search

The neural evaluation stages retain runtime-selected AVX2/SSE2 kernels,
incremental accumulators and king-bucket refresh caching.
