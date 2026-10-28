# Stockfish-format Vulkan training — 2026-10-23

Added all-layer training support to the Maven `../javafish/nnue-training` module.
The new `--architecture stockfish` mode supports the exact Stockfish network used
by Zerofish's direct Rust SIMD evaluator: K32Q2, FullThreats and PP_3Wide inputs,
a 1,024-neuron gated transformer, and 32 output stacks. The legacy trainer remains
available as the default mode.

This experiment fine-tunes the existing Stockfish network. Its output is a derived
checkpoint, not a network trained from scratch.

## Five-minute iGPU run

| Measure | Result |
|---|---:|
| Device | AMD Radeon Graphics (RADV PHOENIX), integrated GPU |
| Training time | 300.012 seconds |
| Positions / optimizer updates | 7,695,104 / 30,059 |
| Average throughput | 25,649 positions/second |
| Held-out MSE before / after | 0.008419983 / 0.008013806 |
| Relative MSE reduction | 4.82% |
| Changed quantized transformer weights | 3,880,515 |
| Changed bias/head bytes | 86,152 |
| Peak Vulkan allocations | 1.276 GiB |

Batch size was 256, learning rate `1e-5`, Adam betas `.9/.999`, epsilon `1e-8`.
Transformer updates use SparseAdam on active feature rows; biases and output heads
use dense Adam. FP32 master weights and optimizer state stay on the GPU. Vulkan
performs forward inference, analytical backpropagation and optimizer updates.
CPU code prepares sparse batches and serializes checkpoints.

The forward pass uses signed-byte fake quantization, integer-compatible activation
rounding, square/linear paths, skip outputs and the pinned score conversion.
Sparse gradients group all visits to a feature row and give one GPU invocation
ownership of each updated parameter, avoiding dense zero gradients and float
atomic contention. This adapts the official trainer's sparse and quantization-aware
approaches to Vulkan; it does not reproduce upstream CUDA kernels or Ranger.
See the [upstream NNUE documentation](https://github.com/official-stockfish/nnue-pytorch/blob/master/docs/nnue.md),
[model](https://github.com/official-stockfish/nnue-pytorch/blob/master/model/model.py) and
[quantization implementation](https://github.com/official-stockfish/nnue-pytorch/blob/master/model/quantize.py).

## Data and validation

Training used the existing `data/python-replay-82826240.bin` stream. Every prepared
record retains a complete board in its first perspective's feature indices. The
trainer reconstructs the board under the original color/reflection symmetries and
uses the engine's exact Stockfish feature encoder. Targets retain the existing
`.8 * sigmoid(score/400) + .2 * outcome` blend. The objective is
`MSE(sigmoid(raw/400), target)`.

Validation used the first 2,048 positions from the million-position independent
reserved-chunk set. Its preparation skips the checkpoint-selection validation
chunk and excludes training chunks. Dataset hashes/provenance and the exact
command are retained in the artifact directory.

Focused Maven checks passed for file round trips, architecture rejection, feature
reconstruction, real-position positive/negative integer inference, analytical
Vulkan gradients, Adam state and quantized export/reload. A broader Maven engine
suite also reported two existing failures (a legacy evaluator fixture and a burst
UCI test); those logs are retained and are not reported as a passing full suite.

The training run checks every validation position against an independent integer
CPU implementation before training and after export. The trained file also passed
Zerofish Rust checks: 2,048 trainer/scalar/SIMD scores and both accumulator arrays,
plus 2,108 incremental sibling transitions versus full rebuilds.

Initial network SHA-256:
`c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73`.

Trained final checkpoint SHA-256:
`59ab55ab6c0a3ebafad5706db3a22e17edfd1367f02a17ae12e7a01d089667d2`.

The final checkpoint is used for playing strength evaluation. The lower loss
observed at an earlier progress check was not substituted for the final network.
A loss improvement on this subset does not establish a playing-strength gain.

## Build a checksum-pinned engine

The default engine still embeds the original pinned Stockfish network. A trained
network requires both its explicit path and expected SHA-256; all architecture,
layer-hash and length checks remain enabled:

```sh
ZEROFISH_STOCKFISH_NETWORK="$PWD/target/training/stockfish-vulkan-5min-20261023/run-final/weights-final.nnue" \
ZEROFISH_STOCKFISH_NETWORK_SHA256=59ab55ab6c0a3ebafad5706db3a22e17edfd1367f02a17ae12e7a01d089667d2 \
CARGO_TARGET_DIR=target/our-nnue cargo build --release --offline
```

Validate it with the same environment and:

```sh
cargo run --release --offline --example validate_trained_network -- \
  target/training/stockfish-vulkan-5min-20261023/run-final/cross-check.tsv
```

Trainer instructions and algorithm details:
`../javafish/nnue-training/STOCKFISH.md`.
Artifacts: `target/training/stockfish-vulkan-5min-20261023/`, including the frozen
trainer JAR, source archive, commands, data provenance, test logs, metrics,
exported network and cross-language validation output.

## 100-game match: final trained checkpoint versus original network

Both engines use the same current Rust search and `Avx512Stockfish` evaluator;
only their embedded network differs. Each uses one search thread and Hash=64 MiB.
The time control is five seconds per engine for the entire game, no increment.
Six games run concurrently on physical cores 0–5, with each runner and both its
alternating engines pinned to one core. Each receives a two-second unclocked
warmup. The machine has eight physical cores; two remain outside the match pool.

Fifty unique EPD positions are sampled with seed `20261023` and played with both
colors. This is the same selection as the preceding original-network comparison.
The final five-minute checkpoint is used without selection based on match results.

| Measure | Trained network versus original Stockfish network |
|---|---:|
| Wins / draws / losses | 10 / 62 / 28 |
| Score | 41.0% |
| Relative performance Elo | -63.2 |
| Conservative 95% whole-pair Hoeffding interval | [-222.0, +71.9] Elo |
| Time forfeits / protocol failures / move caps | 0 / 0 / 0 |
| Wall time including startup and warmup | 183.304 seconds |

The lower validation loss did not translate into a demonstrated playing-strength
gain. The match's point estimate favors the original network; its conservative
interval includes equality. This small experiment does not establish a definitive
strength difference or an absolute rating. The validation target, short adaptation
budget and dataset differ from the original network's training recipe.

Independent Rust replay validated all 100 games, 16,946 legal plies, all result and
termination claims, and 50 globally unique color-reversed opening pairs.
Terminations: 38 checkmates, 54 threefold repetitions, four fifty-move draws and
four insufficient-material draws. CSV aggregation, UCI Threads/Hash settings,
and frozen executable hashes were checked independently.

Match artifacts: `target/elo/our-vulkan-vs-stockfish-rust-1t-5s-50pairs-20261023/`.
It contains merged PGN/CSV, configuration, summary, frozen engines, exact build
logs, source archive/patch, six shard command files and all per-shard UCI logs.

```sh
cargo run --offline --example validate_match -- \
  target/elo/our-vulkan-vs-stockfish-rust-1t-5s-50pairs-20261023/games.pgn 100
```
