# Zerofish versus Stockfish: 2026-10-22

The CLI Elo benchmark completed 100 games from 50 distinct EPD positions,
each played twice with colors reversed. The control was **five seconds per
engine for the whole game, no increment** (`5+0` in this runner). Both engines
used one search thread and Hash=64 MiB, with two-second warmups outside clocks.

| Measure | Result |
|---|---:|
| Zerofish wins / draws / losses | 1 / 35 / 64 |
| Zerofish score | 18.5% |
| Relative logistic Elo (Zerofish minus Stockfish) | -257.6 |
| Approximate 95% whole-pair bootstrap interval | [-301.3, -219.9] Elo |
| Conservative 95% whole-pair Hoeffding interval | [-infinity, -87.2] Elo |
| Pair totals for 0 / 0.5 / 1 / 1.5 / 2 points | 15 / 33 / 2 / 0 / 0 |
| Wall time including startup/warmup | 763.707 seconds |
| Time forfeits / protocol failures / safety caps | 0 / 0 / 0 |

Terminations: 65 checkmates, 24 repetitions, six insufficient-material draws,
four fifty-move draws and one stalemate. This is relative performance under
these conditions, not an absolute rating or a measurement of an isolated change.

## Same network and sanity validation

The original Rust SCReLU evaluator cannot read this Stockfish network. The
optional `stockfish-nnue` build instead uses the pinned native NNUE architecture,
with incremental positions/accumulators and per-worker caches. Zerofish retains
its own move generation, search, ordering, pruning, time management and SMP.
The default Rust build retains the previous SIMD evaluator.

Both match engines embed `nn-c9e480e92425.nnue`, SHA-256:

```text
c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73
```

- Candidate source: `081336a`.
- Opponent and native evaluator upstream: `effdc3fd3f6de6e75fe1374a51a7a7eed66f880b`.
- Both backend configurations passed their 66-test engine suites.
- Native raw evaluation and zero-clock calibrated scores matched the standalone
  Stockfish executable on the oracle positions.
- Incremental evaluation matched full refresh for legal walks, siblings, null
  moves, captures, en passant, promotions, orthodox and Chess960 castling.
- Independent Rust PGN replay verified all 100 games, 13,070 legal plies,
  results/termination claims and 50 unique color-reversed opening pairs.
- CSV score reconstruction agreed with PGN replay and the runner summary.
- Frozen binary/library hashes remained unchanged. The protocol confirmed
  `Threads=1` and `Hash=64` for both engines.

## Reproducibility artifacts

Output directory: `target/elo/stockfish-1t-5s-50pairs/` (ignored build output).
It contains frozen engines and library, source archive, actual-engine configuration
and SHA-256 hashes, both test logs, replay/artifact validation, and runner output.
The `match/` subdirectory contains PGN, CSV, selected EPD positions, full UCI
protocol, summary, metadata and RSS samples. The original EPD source was
`../javafish/docs/4mvs_+90_+99.epd.zip`, sampled with seed `20261022`.

The existing `../javafish` CLI runner was used with custom executable launchers,
`--pairs 50 --tc 5+0 --hash 64 --threads-a 1 --threads-b 1 --warmup-ms 2000`.
Its legacy candidate label `javafish` means **Zerofish engine A** in this run.
Its automatically archived Java sources/JAR and Java-specific heap/TT metadata
describe the runner environment, not the native candidate. Use the parent
`match-config.json`, frozen executables and Rust source archive for engine identity.
Sampled peak RSS was 300,628 KiB for Zerofish and 336,156 KiB for Stockfish.

Replay validation can be repeated with:

```sh
cargo run --offline --example validate_match -- \
  target/elo/stockfish-1t-5s-50pairs/match/games.pgn
```
