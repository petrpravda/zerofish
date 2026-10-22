# Chess960

Ported from the Java engine in `../zerofish-tutor` (HEAD `19afb9c`).

## Protocol and formats

Enable X-FEN interpretation and king-to-rook UCI castling with:

```text
setoption name UCI_Chess960 value true
position fen 4k3/8/8/8/8/8/8/1R1K2R1 w KQ - 0 1
go depth 8
```

Castling is encoded as king starting square to rook starting square. For example,
`d1g1` castles kingside in the position above; `d1b1` castles queenside.
Changing the option clears the transposition table and histories and resets the
position. `ucinewgame` retains the selected variant.

Shredder-FEN rook-file rights, such as `GB` or `DAda`, automatically select
Chess960, even without the option. Chess960 positions serialize with rook-file
rights. X-FEN `KQkq` chooses the outermost rook on each side of the king and
requires explicit Chess960 mode. A FEN with no rights cannot identify its variant;
keep explicit mode when reconstructing it.

Library callers can use `BoardState::from_fen_with_chess960(fen, true)` or
`BoardPosition::from_fen_with_chess960(fen, true)`; rook-file rights select
Chess960 automatically in `from_fen`.

PGN accepts `Chess960`, `Fischerandom`, and `FischerRandom` variant tags, requires
a starting FEN for these variants, and retains `O-O` / `O-O-O` SAN notation.

## Implementation and correctness

- Both castling pieces are removed before either is placed, supporting stationary
  pieces, overlapping destinations, and king/rook swaps.
- King transit squares and the final occupancy are checked for attacks, including
  checks exposed when the rook moves. Rook transit squares may be attacked.
- Castling hashes include the entitled rook's file, using Java's random draw order.
- NNUE deltas cover both starting and destination squares; no new network is used.
- The six castling files occupy the board struct's three padding bytes; the
  orthodox fields and layout are unchanged.
- One compilation serves both variants. Hot board, picker, search and perft
  functions take `const V: bool` and are instantiated twice; `it_deep` and
  `perft` dispatch once per call, so orthodox nodes never test the variant.
- Chess960-only generation, castling and hashing are `#[cold]` and out of line.
  `finish` stays out of line with its helpers inlined, as in the baseline build.

`cargo test --offline` (54 tests) covers all 960 starting arrangements
and every legal first move, all 84 single-rook castling geometries across both
colours, NNUE incremental/full-refresh equality, rights loss, SAN/PGN round trips,
and single-/multi-threaded UCI searches with legal PV replay.

Depth-4 perft agrees with the current Java classes, run locally with JDK 25:

| FEN | Nodes |
| --- | ---: |
| `4k3/8/8/8/8/8/8/1R1K2R1 w GB - 0 1` | 15,869 |
| `4k3/8/8/8/8/8/8/1R4KR w HB - 0 1` | 16,096 |
| `4k3/8/8/8/8/8/8/2RK3R w HC - 0 1` | 14,843 |
| `1r1k2r1/8/8/8/8/8/8/7K b gb - 0 1` | 2,674 |

## Performance (2026-10-21)

The host is heavily loaded (load average ~20 on 12 threads) and hardware
counters are unavailable (`perf_event_paranoid=4`, no valgrind/qemu), so three
load-independent methods were used, all against a baseline rebuilt from HEAD
`fc5cfae` that reproduces the pre-Chess960 binary instruction for instruction:

1. Exact user-mode instruction counts: a ptrace single-step tracer counts every
   retired instruction between two `ud2` markers around a fixed workload
   (ASLR off). Perft counts reproduce exactly; searches vary by <=0.005% from
   time-check calls.
2. Machine-code identity: address-normalized disassembly of every engine function.
3. Timing: baseline and candidate run simultaneously on the two SMT siblings of
   one physical core, sides swapped each round, thread CPU time, with an A/A
   control. Its resolution is about +-1-4% (searches) and +-11% (perft), so it
   only confirms large effects.

Five designs were implemented and measured (instruction delta vs baseline;
perft 2 / classical depth 3 / NNUE depth 2 over three positions):

| Idea | perft | classical | NNUE | Hot machine code |
| --- | ---: | ---: | ---: | --- |
| 1 Runtime flag in a packed word (previous version) | +1.94% | +0.04% | +0.21% | differs |
| 2 Layout-preserving padding bytes + cold out-of-line 960 code, tuned inlining | +0.96% | -0.04% | +0.10% | differs |
| 3 Unified data-driven castling (orthodox = one configuration, no variant branch) | +22.1% | +3.41% | +2.10% | differs |
| 4 Cargo feature, default build (no Chess960 support) | 0 | 0 | 0 | identical |
| 4b Idea 4 compiled twice into one binary (two engines) | 0 | 0 | 0 | identical |
| **5 Const-generic monomorphization, root dispatch, tuned inlining (chosen)** | **+0.51%** | **-0.23%** | **+0.07%** | differs |

Idea 5 is the cheapest design with a single engine compilation. It cannot reach
zero: with fat LTO and one codegen unit, a function's machine code depends on
all of its call sites, and the Chess960 code adds callers to shared helpers
(`attackers_from`, `finish`, `generate`), changing their register allocation and
inlining. Only a separate compilation (4b) avoids that, at the price of
maintaining two engine builds.

The search cost is at most about 0.1%, roughly 0.05-0.1 Elo at 3600; perft
measures raw move generation only. The negative classical figure is
code-generation noise, not a real gain.
