# Rust sparse SIMD versus Stockfish — 2026-10-23

Completed 40 games from 20 distinct EPD openings, each played twice with colors
reversed. Both engines used **one search thread**, Hash=64 MiB, and **five seconds
for the whole game with no increment** (`5+0` in the CLI runner). Each received a
two-second warmup outside game clocks.

| Measure | Result |
|---|---:|
| Zerofish wins / draws / losses | 0 / 10 / 30 |
| Zerofish score | 12.5% |
| Relative performance Elo (Zerofish minus Stockfish) | -338.0 |
| Conservative 95% whole-pair Hoeffding interval | [-infinity, -49.9] Elo |
| Pair totals for 0 / 0.5 / 1 / 1.5 / 2 points | 10 / 10 / 0 / 0 / 0 |
| Time forfeits / protocol failures / safety caps | 0 / 0 / 0 |
| Wall time including startup and warmup | 306.004 seconds |

Terminations: 30 checkmates, seven threefold repetitions, two insufficient-material
draws and one fifty-move draw. This is a small match's relative performance under
these conditions, not an absolute rating or an isolated measurement of SIMD Elo
impact. The interval assumes independently sampled distinct opening pairs.

## Frozen engines and network

- Zerofish source: `da65e1b`, using `Avx512SparseVnni`, the new direct-Rust
  Stockfish-compatible sparse SIMD evaluator. The frozen executable exactly matches
  the binary validated in the [SIMD comparison](RUST_NNUE.md). It has no NNUE shared
  library or C++ runtime dependency.
- Stockfish source: `effdc3fd3f6de6e75fe1374a51a7a7eed66f880b`. The executable is
  byte-identical to the opponent from the preceding 100-game experiment.
- Both embed `nn-c9e480e92425.nnue`, SHA-256:

```text
c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73
```

Opening source: `../javafish/docs/4mvs_+90_+99.epd.zip`, seed `20261023`.
The existing Java CLI runner was launched with custom engine executables,
`--pairs 20 --tc 5+0 --hash 64 --threads-a 1 --threads-b 1 --warmup-ms 2000`.
Its legacy `javafish` candidate label in PGN/protocol/CSV metadata denotes
Zerofish engine A. Automatically recorded Java heap/TT fields do not describe
these native engines; `match-config.json` records their actual identities.

## Validation and artifacts

Rust PGN replay validated all 40 games, all 4,884 legal plies, termination/result
claims and all 20 unique color-reversed opening pairs. Independent CSV scoring
agreed with the PGNs and runner summary. Frozen binary/launcher hashes remained
unchanged; the UCI transcript confirms Threads=1 and Hash=64 for both engines.
Sampled peak RSS was 294,132 KiB for Zerofish and 335,944 KiB for Stockfish.

Output directory: `target/elo/rust-sparse-stockfish-1t-5s-20pairs-20261023/`.
It contains the frozen executables/launchers, Rust source archive, configuration
and hashes, dependency evidence, runner output and independent replay/artifact
validation. The `match/` subdirectory contains selected EPDs, PGN, CSV, full UCI
protocol, runner metadata, memory samples and `summary.txt`.

Replay with:

```sh
cargo run --offline --example validate_match -- \
  target/elo/rust-sparse-stockfish-1t-5s-20pairs-20261023/match/games.pgn 40
```
