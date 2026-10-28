# Zerofish network comparison — 2026-10-23

Completed 100 games from 50 distinct EPD positions, each played twice with colors
reversed. Both engines were built from the same current working tree and used
one search thread, Hash=64 MiB, and five seconds for the entire game with no
increment (`5+0` in the CLI runner). Warmup was two seconds per engine outside
game clocks.

| Measure | Stockfish-network Rust SIMD versus original network |
|---|---:|
| Wins / draws / losses | 47 / 46 / 7 |
| Score | 70.0% |
| Relative performance Elo | +147.2 |
| Conservative 95% whole-pair Hoeffding interval | [+5.5, +366.9] Elo |
| Time forfeits / protocol failures / move caps | 0 / 0 / 0 |
| Total wall time, including startup and warmup | 187.786 seconds |

This measures the complete evaluator change with the same search implementation,
including network architecture, weights, calibration and evaluation speed. It is
not an isolated SIMD speed measurement or an absolute engine rating. The interval
uses 50 independently sampled opening pairs as its sampling units.

The machine has eight physical cores and 16 logical CPUs. Six runner processes
played games concurrently, each pinned to a separate physical core (CPUs 0–5).
Each runner's two engines inherited that affinity and took alternating turns;
Threads=1 was sent to both. Two physical cores were left outside the match pool.

## Builds and networks

- Original-network Zerofish: `cargo build --release --offline --no-default-features`,
  using the preserved 1,024-hidden, eight-king-bucket SCReLU evaluator and its
  existing SIMD/cache improvements. Network SHA-256:
  `85959e702a2147cceaf6829a6933211865e2c271d434f394ac46063b90f7a6b3`.
- Stockfish-network Zerofish: `cargo build --release --offline`, selecting the new
  `Avx512Stockfish` direct Rust evaluator. Network SHA-256:
  `c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73`.
  No NNUE shared library or C++ runtime is linked.

Both builds use the working tree based on `0f26109`, including the uncommitted
Stockfish-style SIMD optimization. The artifact records the exact base commit,
working-tree patch, source archive, build logs and frozen binary hashes.

Openings came from `../javafish/docs/4mvs_+90_+99.epd.zip`. Python's random generator
with seed `20261023` selected 50 unique four-field positions, distributed across
six disjoint shards (9/9/8/8/8/8 pairs). The existing Java CLI runner shuffled each
shard with the same seed and played every position with both colors. Its legacy
candidate label `javafish` denotes the original-network Zerofish; the merged PGN
uses descriptive engine labels.

## Validation and artifacts

Independent Rust replay validates all 100 games, 17,277 legal plies, termination/result
claims, and 50 globally unique color-reversed opening pairs. Independent CSV
aggregation agrees with the results. The UCI transcripts confirm Threads=1 and
Hash=64 MiB for both engines in every shard. Frozen executable hashes were checked
after completion.

Terminations: 54 checkmates, 37 threefold repetitions, six fifty-move draws and
three insufficient-material draws.

Artifacts: `target/elo/original-vs-stockfish-rust-1t-5s-50pairs-20261023/` contains
merged `games.pgn`, `games.csv`, `summary.json`, `match-config.json`, frozen engines,
source snapshot, build/dependency evidence, replay output, orchestration and
aggregation scripts, and complete per-shard PGN/CSV/UCI logs.

```sh
cargo run --offline --example validate_match -- \
  target/elo/original-vs-stockfish-rust-1t-5s-50pairs-20261023/games.pgn 100
```
