# NNUE Elo parity: checkpoint and interpolation experiments

Both candidates were chosen before games, using the protocol in
[the parity plan](STOCKFISH_NNUE_ELO_PARITY_PLAN.md). They retain the original
Stockfish transformer and adapt only dense heads. They are derived networks,
not networks trained from scratch. The original remains the default.

## Experiments and screening results

| Candidate | Changed head bytes | Holdout raw RMSE | 100-game W/D/L | Score | Relative Elo |
|---|---:|---:|---:|---:|---:|
| Phase 1: 95% teacher, final 300 s | 1,994 | 4.253 | 17/62/21 | 48% | -13.90 |
| Phase 2: 99% teacher, selected checkpoint | 1,213 | 1.633 | 20/63/17 | 51.5% | +10.43 |
| Phase 2: half of phase-1 head delta | 1,791 | 2.994 | 21/66/13 | 54% | +27.85 |

Every screen used the same 50 color-reversed screening positions, Threads=1,
Hash=64 MiB, 5+0 seconds for the whole game and six concurrent games pinned to
six physical cores. Point estimates from 100 games are noisy and neither establish
parity nor reliably rank the candidates. Their conservative 95% whole-pair
Hoeffding intervals were [-128.61,+153.07] and [-109.12,+174.61] Elo for the two
phase-2 candidates. No time forfeits, protocol failures or move caps occurred.
All 200 phase-2 games were independently replayed.

## Checkpoint selection

The 99%-teacher run trained for 300.001 seconds, processing 5,695,744 positions
with learning rate `1e-6`, batch 256, original-teacher targets and frozen transformer
rows/biases. Nine quantized checkpoints were saved at about 30-second intervals,
plus the final export. Checkpoint export and progress validation count toward the
training timer. Only dense GPU parameters are downloaded in frozen mode.

The predeclared rule selected the changed checkpoint with minimum mixed objective
on 2,048 validation records, subject to raw RMSE <=3, |bias| <=2 and slope in
[0.99,1.01]. Both model identities and eligibility decisions were frozen before
the first screen. The larger audit uses 65,536 separate records beginning at
record 2,048 and does not choose checkpoint time.

| Checkpoint time | Changed head bytes | Selection mixed MSE | Raw RMSE |
|---|---:|---:|---:|
| **30.01 s (selected)** | 1,213 | 1.46083411e-06 | 1.635 |
| 60.32 s | 1,444 | 1.60395386e-06 | 1.873 |
| 90.59 s | 1,546 | 1.78317774e-06 | 2.075 |
| 120.87 s | 1,568 | 1.8718853e-06 | 2.139 |
| 151.14 s | 1,657 | 1.93208928e-06 | 2.207 |
| 181.41 s | 1,687 | 1.93162637e-06 | 2.206 |
| 211.66 s | 1,728 | 1.93840901e-06 | 2.236 |
| 241.93 s | 1,736 | 2.03777031e-06 | 2.320 |
| 272.18 s | 1,751 | 2.02442735e-06 | 2.314 |
| 300.00 s | 1,762 | 2.06292257e-06 | 2.356 |

The selected model is the **30-second checkpoint**, not the five-minute final
checkpoint. Its validation objective, 0.00000146083, is **worse** than the unchanged
original's 0.000000841997. No saved changed checkpoint improved that objective.
Longer training increased drift and generally worsened the mixed loss. The
experiment therefore demonstrates preservation and checkpoint selection, not
successful optimization beyond the teacher.

The 50% blend was declared separately: normalized head parameters are
`original + .5*(phase1_trained-original)`, then quantized with ties-to-even rounding.
Nonlinear inference means this is not a 50% blend of scores. Header, transformer
rows and transformer biases remain identical. It passed the same drift gates on
selection data before screening.

## Validation and provenance

Twenty focused Maven tests passed, including full/frozen export equivalence,
interpolation endpoints, changed-transformer rejection, GPU gradients, pure-teacher
zero updates and integer inference. Each phase-2 candidate passed 2,080
trainer/CPU/Rust scalar/SIMD cases covering all 32 output heads, plus incremental
sibling checks. The sampled audit covers 30 heads; explicit fixtures test numerical
correctness for the two rare heads but do not establish representative playing
strength in those regimes.

Candidate hashes:

- Selected teacher99: `eba2c33289de669424f66f2ed3cb87a443bf6a34dc4ef6c3a7a68a0f774c84e0`.
- Half-delta95: `5fcf2a3b68af2344798cb638db3c70eee3b8248600b0c6afcf722a3bf2ef90df`.

Training, selection index, checkpoints, audits, source and trainer snapshots:
`target/training/nnue-parity-phase2-20261023/`.

Screen artifacts:
`target/elo/parity-teacher99-selected-1t-5s-50pairs-20261023/` and
`target/elo/parity-halfdelta95-1t-5s-50pairs-20261023/`.

## Independent confirmation

The teacher99 checkpoint is frozen for a fixed 2,000-game confirmation. It was
chosen for lower validation objective and drift, rather than choose the blend
because its noisy screen score was higher. The confirmation has 1,000 unique
opening pairs from a new, checksum-pinned six-move book, excludes screening
positions, and uses seed 20261024. It runs to the complete sample with no optional
stopping. The primary analysis is the predeclared one-sided 95% paired Student-t
lower bound (a large-sample approximation), requiring it to exceed -10 Elo;
a conservative Hoeffding bound is also reported. See the frozen protocol for
all assumptions and the degenerate-variance rule.

Status: **complete; the predeclared primary noninferiority criterion was met**.
This is operational parity within the tested margin/conditions, not universal
strength equivalence or a claim about the full Stockfish engine.

| Fixed confirmation result | Value |
|---|---:|
| Games / unique opening pairs | 2,000 / 1,000 |
| Candidate wins / draws / losses | 386 / 1,269 / 345 |
| Candidate score | 51.025% |
| Relative performance Elo | +7.123 |
| Primary one-sided 95% paired-t Elo lower bound | +0.343 |
| Required lower bound | > -10 Elo |
| One-sided noninferiority p-value | 0.00001747 |
| Secondary one-sided Hoeffding Elo lower bound | -19.792 |
| Forfeits / protocol failures / move caps | 0 / 0 / 0 |
| Wall time including startup/warmup | 3,149.459 s (52.49 min) |

The paired-t bound is a **large-sample approximation** based on 1,000 bounded
opening-pair means. The distribution-free Hoeffding bound is wider and does not
meet the -10 Elo margin; the result is specifically a pass under the primary
method frozen before games, not a distribution-free guarantee. No analysis method
was selected after seeing outcomes, and there was no outcome-based early stop.
This does not establish superiority or universal parity across books/time controls.

The fixed sample has pair-mean counts for 0 / .25 / .5 / .75 / 1 of
10 / 219 / 503 / 256 / 12. Independent Rust replay validated all 2,000 games,
1,000 unique reversed opening pairs and 299,736 legal plies. Independent CSV/WDL
and pair-mean scoring agreed; all frozen executable hashes and analysis-script
hash matched the predeclared protocol. The UCI logs confirm Threads=1 and Hash=64
for both engines in every shard.

Terminations: 731 checkmates, 941 repetitions, 203 insufficient-material draws,
111 fifty-move draws and 14 stalemates. The 1,000 canonical starting positions
have no overlap with the screening set. The new book and its CC0 notice are frozen
with source commit and digest in the artifact directory.

The original network remains the default; no automatic promotion was performed.
The confirmed candidate is a changed, teacher-derived network. Its lower drift
and acceptable match result do not erase the fact that its mixed validation
objective was worse than initialization. Further training should improve labels
and calibration rather than simply increase this run's duration or learning rate.

Confirmed weights:
`target/training/nnue-parity-phase2-20261023/teacher99-selected.nnue`.
Confirmed executable:
`target/elo/parity-teacher99-confirmation-2000-20261023/frozen/zerofish-our-nnue`.

Reproduce the fixed-sample analysis (after complete PGN validation):

```sh
python3 scripts/analyze_paired_match.py \
  target/elo/parity-teacher99-confirmation-2000-20261023/games.csv \
  --expected-games 2000 --elo-margin 10 --alpha .05
```

The statistical implementation passed analytic t(1)/t(2) checks, an independent
PDF integration check at 999 degrees of freedom, and three synthetic fixtures
covering exact pair scoring and incomplete/zero-variance sample safeguards.
Artifacts: `target/elo/parity-teacher99-confirmation-2000-20261023/`.

## Durable export outside build directories

The confirmed weights and compressed evidence are also retained under
`../javafish/data/runs/stockfish-parity-eba2c33289de/`, which survives `cargo clean`
and Maven build cleanup. It contains `weights-confirmed.nnue`, compressed PGN/CSV,
source snapshots, selected openings, frozen analysis/protocol, checks, audits and
Stockfish/book provenance. The directory is ignored by Git like the existing
training datasets; the original weights are not overwritten.

Build the confirmed alternative explicitly:

```sh
ZEROFISH_STOCKFISH_NETWORK="$PWD/../javafish/data/runs/stockfish-parity-eba2c33289de/weights-confirmed.nnue" \
ZEROFISH_STOCKFISH_NETWORK_SHA256=eba2c33289de669424f66f2ed3cb87a443bf6a34dc4ef6c3a7a68a0f774c84e0 \
CARGO_TARGET_DIR=target/confirmed-teacher99 cargo build --release --offline
```

Run `target/confirmed-teacher99/release/zerofish`. The ordinary default build still
uses the original Stockfish network. All checksum and architecture checks remain
active for both builds.
