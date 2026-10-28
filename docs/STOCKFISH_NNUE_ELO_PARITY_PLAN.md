# Stockfish NNUE Elo parity plan

Status: the first operational noninferiority criterion was met on 2026-10-23
in the fixed 2,000-game independent confirmation. The original network remains
the default; broader validation and target/data work remain. Scope: a **trained, changed network** used by
Zerofish versus the original Stockfish network used by the same Zerofish build.
This is not parity with the full Stockfish engine. Keep the architecture and
Rust SIMD inference fixed so the experiment measures network quality.

## Baseline and evidence

- Original network SHA-256:
  `c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73`.
- First five-minute all-layer Vulkan run: 7,695,104 positions, batch 256,
  learning rate `1e-5`, SparseAdam transformer and Adam heads.
- Legacy-target validation MSE improved 4.82%, but the final checkpoint scored
  10 wins / 62 draws / 28 losses in 100 games: 41%, estimated -63.2 Elo.
  Conservative paired interval [-222.0,+71.9] includes equality.
- Exact export/GPU/CPU/Rust inference and incremental-accumulator checks passed.
  Mathematical checks do not establish playing strength.
- The first experiment optimized `sigmoid(raw/400)` against targets previously
  blended from old teacher evaluations and results. This is an objective/domain
  mismatch hypothesis, **not a demonstrated root cause** of the match result.

Prior evidence: [training and match report](VULKAN_NNUE_TRAINING.md).

## Success criteria and experimental discipline

A copied original network or a zero-update run is only a control, never success.
A candidate must have changed quantized parameters, exact inference/export checks,
and no unexplained protocol, clock or move-generation failures.

Operational target: demonstrate noninferiority within **10 Elo** against the
original network under the same search, network architecture, Threads=1,
Hash=64 MiB and 5+0 seconds. Use color-reversed opening pairs as the statistical
unit. Confirmation requires the one-sided 95% lower confidence bound above
-10 Elo using a predeclared paired analysis and an independent confirmation set.
A point estimate near zero or an interval containing zero is not proof of parity.

Use 100-game screens to reject obvious regressions, not certify parity. Freeze a
shortlist before an independent confirmation match, initially 2,000 games and
expand to a predeclared maximum (for example 10,000) if precision is insufficient.
Use a fixed-sample analysis or a valid sequential test with stated hypotheses;
do not repeatedly inspect ordinary intervals and stop when one looks favorable.
If the maximum budget cannot resolve the margin, report inconclusive.

Run six games concurrently on six distinct physical cores, one engine search
thread each, reserving two physical cores. Freeze binaries, networks, source,
commands, seeds, dataset provenance and results. Keep screening and confirmation
openings disjoint. Preserve earlier experiment artifacts.

## Phase 1 — objective audit and preservation controls (start here)

1. Audit raw network, scaled internal and reported centipawn units. Upstream loss
   uses separately transformed network/search scores; do not apply upstream
   defaults to this older dataset without validating its units and calibration.
2. Measure teacher/student raw drift, probability drift, bias and slope on a
   larger reserved sample. Break results down by material/output stack and eval
   range. Keep an untouched audit subset separate from checkpoint selection.
3. Add explicit teacher distillation: targets combine the original network's
   probability with the existing outcome/evaluation target. Record the formula,
   temperature and mixture. An identical student under pure distillation must
   produce zero gradients and a byte-identical export; test this on Vulkan.
4. Add `--freeze-transformer true` covering both transformer rows and biases.
   Check byte preservation in the export. Begin with conservative head adaptation
   and `1e-6`, reducing the first experiment's rate tenfold.
5. First changed-network experiment: 95% original-teacher target + 5% legacy
   target, frozen transformer, five minutes. Validate and run a 100-game screen.
   Teacher preservation limits drift; it does not guarantee better Elo.

Deliverables: implementation/tests, calibration/drift report, changed network,
exact inference checks, screen results and an updated experiment ledger below.

## Phase 2 — select conservative adaptation settings

Use a small, declared grid rather than an open-ended search:

- Teacher target weight: 0.95 and 0.99.
- Learning rate: `1e-6`, then `2e-6` only if drift and screens justify it.
- Transformer frozen first; unfreeze at a smaller separate rate only after heads
  preserve strength. Track optimizer semantics and quantized change counts.
- Save timed checkpoints; choose with validation plus teacher drift, then screen
  a fixed shortlist. Do not choose solely by the old target's MSE.
- Consider quantized weight interpolation (e.g. 25%/50% trained delta) as an
  explicitly labeled experiment. Revalidate each interpolated network.

Promote only candidates with meaningful changes and no clear screening regression.
One hundred games may not rank close candidates reliably.

## Phase 3 — improve targets and data

Add a record format retaining FEN/features, original teacher raw score, search
score with known units, game result, material bucket and provenance. Existing
132-byte blended-target records cannot recover separate score/outcome labels.

Generate fresh, diverse positions from Zerofish search/self-play. Label a fixed
subset with stronger Stockfish search and record depth/nodes/time and conversion
settings. Include all output heads, tactical and quiet positions, and endgames;
filter illegal/corrupt samples and avoid train/validation game overlap. Calibrate
network/search probability mappings on held-out results and compare candidate
objectives before increasing training duration.

Use larger balanced batches if measured beneficial. More training time or a
faster GPU helps only after targets and validation preserve strength.

## Phase 4 — independent confirmation and release decision

Freeze one changed candidate and its hyperparameters. Run the declared paired
confirmation match on untouched openings. Replay every PGN, independently score
the CSV, verify identical search settings and frozen hashes, and report raw WDL,
pentanomial pair counts, confidence bounds, terminations, time control and seeds.
Keep the original network as default until the noninferiority criterion is met.
If a candidate fails, record the failure and return to target/data work.

## Upstream research

Reviewed 2026-10-23; upstream master may evolve and is not a pinned recipe for our
network/dataset. These references motivate the audit and implementation:

- [Loss implementation: separate score transformations and outcome mixing](https://github.com/official-stockfish/nnue-pytorch/blob/master/model/nnue.py)
- [Loss configuration](https://github.com/official-stockfish/nnue-pytorch/blob/master/model/config.py)
- [Sparse training and quantization](https://github.com/official-stockfish/nnue-pytorch/blob/master/docs/nnue.md)

## Experiment ledger

| Experiment | Status | Network change | Match / decision |
|---|---|---|---|
| Original all-layer, legacy targets, `1e-5`, 300 s | Complete | 3,880,515 transformer weight bytes changed | 10/62/28; not promoted |
| Pure teacher, identical initialization | Passed on Vulkan | Zero gradients; byte-identical export | Correctness only; never evidence of learned parity |
| 95% teacher, frozen transformer, `1e-6`, 300 s | Screen complete | 1,994 head bytes changed; transformer rows/biases unchanged | 17/62/21, 48%, -13.9 Elo; not promoted |
| 99% teacher, frozen transformer, `1e-6`, saved checkpoints | Screen complete | Selected 30-second checkpoint; 1,213 head bytes changed | 20/63/17, 51.5%, +10.43 Elo; confirmation candidate |
| 50% quantized delta of phase-1 95% teacher | Screen complete | 1,791 head bytes changed | 21/66/13, 54%, +27.85 Elo; not chosen by noisy screen rank |
| Independent noninferiority confirmation | Complete; primary criterion met | Frozen 99%-teacher checkpoint | 386/1269/345, 51.025%, +7.12 Elo; one-sided paired-t lower bound +0.34 > -10 |

## Phase 1 progress

- Implemented explicit teacher targets, an inference-only GPU teacher, and
  transformer freezing (rows **and** biases). Frozen mode skips transformer
  optimizer allocations/updates and verifies byte preservation after export.
- Added Vulkan tests for exact zero-gradient/byte-identical pure distillation and
  nonzero mixed-target head changes with frozen transformer bytes. Focused tests:
  18 passing (including drift-statistics, existing Stockfish inference and legacy trainer checks).
- Added `StockfishAudit` and drift metrics by output head and score range.
  The separate audit begins at reserved validation record 2,048, excluding the
  2,048 records used for progress/checkpoint validation.
- Previous checkpoint on 65,536 audit records: raw RMSE 90.530, mean absolute
  drift 59.505, signed bias +21.839, probability RMSE 0.035133, fitted slope
  0.8520 and intercept +19.853 relative to original teacher scores. This measures
  substantial calibration drift; it does not prove that drift caused the Elo loss.
  The sample covers 30/32 output heads, so missing-head coverage remains work.
- Unit audit: Rust `raw()` precedes material/correlation scaling and centipawn
  conversion. `evaluate_position()` applies those extra mappings. A dataset field
  named score/cp therefore must not automatically be compared with raw output
  using the same divisor. The old 132-byte records retain only the blended target,
  so separate score/outcome calibration requires decoding richer source records.
  Teacher preservation aligns student/teacher raw units without claiming that the
  divisor 400 is an independently fitted win-rate model.
- Five-minute conservative run completed: 5,686,528 positions in 300.006 s,
  approximately 18,955 positions/s including original-teacher GPU inference.
  Export SHA-256:
  `d5423e92a301e80a5c238fb73c24391a329b74ad5596756bbac8f5ae77f8c4b0`.
- On 65,536 separate audit records, raw RMSE fell from 90.530 for the first
  checkpoint to 4.253; probability RMSE fell from 0.035133 to 0.001913. Score slope
  is 0.99634, intercept +1.008 and signed bias +1.057. This is much better teacher
  preservation, not proof of better chess.
- Audit legacy-target MSE: original 0.00927667; first all-layer checkpoint
  0.00882924; conservative checkpoint 0.00922614. Lower legacy-label loss alone
  does not choose the stronger network. On progress validation, the conservative
  **mixed objective worsened** from 0.0000210500 to 0.0000216014 (+2.62%), despite
  a lower legacy-label MSE. Checkpoint saving and gentler updates remain necessary.
- All transformer rows and biases stayed byte-identical; 1,994 head bytes changed.
  Audit heads 29 and 30 were absent from the sampled data but changed by 34 and
  35 bytes respectively. Explicit FEN fixtures now verify numerical correctness
  for all 32 heads; representative strength/generalization coverage for the rare
  heads remains incomplete.
- Exact trained-network checks: 2,048 trainer/CPU/Rust scalar/SIMD positions and
  2,108 incremental transitions, plus 32 explicit all-head fixtures and 806 more
  incremental transitions. Both accumulator arrays agree in the Rust checks.
- Paired 100-game screen: **17 wins / 62 draws / 21 losses**, 48%, -13.90 Elo.
  Conservative whole-pair interval [-157.29,+124.65] Elo. This is closer to zero
  than the first screen's -63.2 point estimate, but the difference between the
  screens is not established as statistically significant.
- Six concurrent games, Threads=1, Hash=64 MiB, 5+0 seconds, 50 unique reversed
  opening pairs, same screening seed 20261023. Independent replay validated all
  100 games and 16,345 legal plies. No forfeits, protocol failures or move caps.
  Wall time including startup/warmup: 185.443 s.
- Decision: retain this as a preservation baseline, **do not promote it to default**
  or claim parity. Next declared experiment: 99% teacher with the same frozen
  transformer and `1e-6`, add saved checkpoints, and retain objective/drift evidence
  before further screens. Improve representative coverage and freeze a candidate
  before the independent noninferiority confirmation.

Artifacts: `target/training/nnue-parity-phase1-20261023/`.

Screen artifacts: `target/elo/parity-teacher95-heads-1t-5s-50pairs-20261023/`.
The training directory contains the candidate, immutable-teacher controls,
65,536-record audit reports, per-head change counts, all-head FEN fixtures,
commands/pipeline, source archives, frozen trainer binaries and test evidence.

## Phase 2 protocol — declared before results

Two candidates will be screened without choosing between them using game results:

1. **Teacher99 checkpoint:** five minutes on the original network, teacher weight
   0.99, frozen transformer, learning rate `1e-6`, batch 256. Save quantized
   checkpoints every 30 seconds and include the final export in the candidate
   pool. On the same first 2,048 reserved validation records, select the changed
   checkpoint with lowest mixed objective subject to raw teacher RMSE <= 3,
   absolute raw bias <= 2, and score slope within [0.99,1.01]. Break exact ties
   by earlier elapsed time, then hash. Exclude the unchanged original. If every
   eligible changed checkpoint is worse than initialization, report that fact;
   do not call it an objective improvement. No eligible checkpoint means no screen.
2. **Half-delta95:** interpolate the already frozen phase-1 95%-teacher checkpoint
   with the original network at fraction 0.5, in normalized dense parameter space,
   followed by ties-to-even integer quantization. Transformer/header bytes must
   match. Apply the same drift limits on the first 2,048 records and reject an
   unchanged export. Its derivation is fixed independently of candidate 1's results.

For each eligible candidate: audit records 2,048–67,583 separately, check all
32 heads with explicit fixtures, validate trainer/CPU/Rust inference and incremental
accumulators, then screen 100 games using the existing 50 paired screening openings,
six concurrent games, one thread each, Hash=64 MiB and 5+0 seconds. A validation gate
failure is recorded rather than replaced with another candidate chosen post hoc.
Neither screen can certify the 10-Elo noninferiority margin. Keep the original
network as default and preserve all earlier artifacts.

Frozen-mode checkpoints preserve the read-only transformer from the initial file,
assert transformer-bias equality, and download only dense parameters. This avoids
repeated 448 MB transformer downloads during checkpointing. GPU/export parity tests
still check that the serialized checkpoint predicts what the GPU predicts.

Phase 2 artifacts: `target/training/nnue-parity-phase2-20261023/`.

## Independent confirmation protocol — frozen before confirmation games

The two phase-2 screens finished at 20/63/17 (51.5%, +10.43 Elo) for the selected
99%-teacher checkpoint and 21/66/13 (54%, +27.85 Elo) for the half-delta blend.
These noisy screen point estimates do not rank their true strength reliably.

Freeze **the 99%-teacher checkpoint** for confirmation because it has the lower
selection-validation objective and teacher drift, rather than choose the blend
because it scored higher in 100 games. Network SHA-256:
`eba2c33289de669424f66f2ed3cb87a443bf6a34dc4ef6c3a7a68a0f774c84e0`.
It is the 30-second checkpoint from the five-minute run, not the final checkpoint.

- Fixed sample: 2,000 games, 1,000 unique color-reversed opening pairs; no optional
  stopping or candidate changes. Seed 20261024. All screening positions excluded.
- New opening source: official-stockfish/books `6mvs_+90_+99.epd.zip`, commit
  `a804a97e709f40aa75a00f9d19e84915b7a3a45c`, CC0-1.0, ZIP SHA-256
  `0c2f4472bba97491d858fe1c7c1ed8d825bc5c7f5463568cc1faa1f707ca2f69`.
  The previous four-move book contains only 635 positions and cannot supply 1,000
  untouched pairs. The six-move book has 2,933 unique unscreened positions.
  This changes opening depth; conclusions apply to these confirmation conditions.
- Same Threads=1, Hash=64 MiB, 5+0 seconds, two-second unclocked warmup per engine,
  six physical cores with one active game per core. Frozen validated executables.
- Primary analysis: one-sided 95% **paired Student-t** lower bound on mean pair
  score (each pair contributes 0/.25/.5/.75/1), transformed to relative Elo.
  This is a large-sample approximation for bounded discrete pair scores, not a
  finite-sample exact guarantee. Null: Elo <= -10; threshold score
  `1/(1+10^(10/400))`. Accept operational noninferiority only if the lower bound
  is strictly above -10 after the complete sample and successful game replay.
- Also report a one-sided Hoeffding bound; do not choose the more favorable
  method after observing results. If sample variance is zero, use only the
  Hoeffding bound. No automatic switch of default weights.
- Statistical implementation: `scripts/analyze_paired_match.py`, frozen with its
  checksum in the confirmation protocol. Closed-form t(1)/t(2) checks and
  independent PDF integration check the numerical CDF/quantile implementation.

Source: [official Stockfish book collection](https://github.com/official-stockfish/books).
Artifacts/protocol: `target/elo/parity-teacher99-confirmation-2000-20261023/`.
Estimated wall time at the observed throughput: about one hour.

Detailed phase-2 evidence: [checkpoint/interpolation report](STOCKFISH_NNUE_ELO_PARITY_PHASE2.md).

## Confirmation outcome and disposition

The fixed 2,000-game match completed without optional stopping: **386 wins,
1,269 draws, 345 losses**, 51.025%, relative performance **+7.123 Elo**.
The predeclared one-sided 95% paired Student-t lower bound is **+0.343 Elo**,
above the -10 Elo noninferiority threshold (p=0.00001747 for that margin).
Thus the plan's operational parity criterion was met under these conditions.
The paired-t analysis is a large-sample approximation. The secondary one-sided
Hoeffding bound is -19.792 Elo and does not meet that margin; it was not substituted
for, or hidden in favor of, the primary method after results were known.

All 2,000 games, 1,000 unique reversed opening pairs and 299,736 legal plies were
independently replayed. CSV/WDL and pair scoring agree. There are no forfeits,
protocol failures or move caps, no screening-opening overlap, and no frozen
binary/analysis hash changes. Wall time: 3,149.459 seconds. Full evidence and
limitations are in [the phase-2 report](STOCKFISH_NNUE_ELO_PARITY_PHASE2.md).

Disposition: retain the original as default; the confirmed teacher-derived
checkpoint is available as a validated alternative. This work establishes
noninferiority for the chosen opening book, time control and analysis assumptions,
not universal equivalence or parity with the complete Stockfish engine.
Do not increase learning rate simply because the screen point estimates improved:
no changed teacher99 checkpoint improved the mixed objective. Next useful work is
richer score/outcome records, calibration, representative rare-head coverage, and
validation on other time controls/books before considering broader adoption.

Durable confirmed weights and compressed evidence: `../javafish/data/runs/stockfish-parity-eba2c33289de/` (outside build directories).
