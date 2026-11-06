// SPDX-License-Identifier: GPL-3.0-or-later
//! Differential and timing comparison: direct Rust versus the optional linked oracle.
//! Run with --features stockfish-nnue. Timings include feature encoding and propagation.
#[cfg(feature = "stockfish-nnue")]
mod comparison {
    use std::{hint::black_box, time::Instant};
    use zerofish::{
        board::BoardState,
        fen::START_POS,
        moves::Move,
        stockfish_rust::{Kernel, Nnue},
        random::JavaRandom,
        stockfish_reference,
    };
    trait Evaluator {
        fn reset(&mut self, state: &BoardState);
        fn refresh(&mut self, state: &BoardState, ply: usize);
        fn push(&mut self, before: &BoardState, after: &BoardState, mv: Move, ply: usize);
        fn raw(&mut self, state: &BoardState, ply: usize) -> i32;
    }
    macro_rules! evaluator {
        ($ty:ty) => {
            impl Evaluator for $ty {
                fn reset(&mut self, s: &BoardState) {
                    self.reset(s);
                }
                fn refresh(&mut self, s: &BoardState, p: usize) {
                    self.refresh_full(s, p);
                }
                fn push(&mut self, b: &BoardState, a: &BoardState, m: Move, p: usize) {
                    self.push_move(b, a, m, p);
                }
                fn raw(&mut self, s: &BoardState, p: usize) -> i32 {
                    self.raw(s, p)
                }
            }
        };
    }
    // The Rust refresh always starts from biases. The oracle must likewise
    // invalidate its selected Finny entries for a genuine full-rebuild timing.
    evaluator!(stockfish_reference::Nnue);
    impl Evaluator for Nnue {
        fn reset(&mut self, s: &BoardState) {
            self.reset(s);
        }
        fn refresh(&mut self, s: &BoardState, p: usize) {
            self.refresh(s, p);
        }
        fn push(&mut self, b: &BoardState, a: &BoardState, m: Move, p: usize) {
            self.push_move(b, a, m, p);
        }
        fn raw(&mut self, s: &BoardState, p: usize) -> i32 {
            self.raw(s, p)
        }
    }
    struct Walk {
        states: Vec<BoardState>,
        moves: Vec<Move>,
    }
    fn pass(
        evaluator: &mut dyn Evaluator,
        walks: &[Walk],
        incremental: bool,
        rounds: usize,
    ) -> (f64, i64) {
        let start = Instant::now();
        let mut checksum = 0i64;
        let mut count = 0;
        for _ in 0..rounds {
            for walk in walks {
                if incremental {
                    evaluator.reset(black_box(&walk.states[0]));
                }
                for (i, &mv) in walk.moves.iter().enumerate() {
                    let child = black_box(&walk.states[i + 1]);
                    let ply = if incremental {
                        evaluator.push(black_box(&walk.states[i]), child, mv, i + 1);
                        i + 1
                    } else {
                        evaluator.refresh(child, 0);
                        0
                    };
                    checksum += black_box(evaluator.raw(child, ply)) as i64;
                    count += 1;
                }
            }
        }
        (start.elapsed().as_secs_f64() * 1e9 / count as f64, checksum)
    }
    pub fn run() {
        assert!(
            Kernel::avx512_available(),
            "AVX-512/VNNI required for this comparison"
        );
        let rounds = std::env::args()
            .nth(1)
            .map(|s| s.parse().unwrap())
            .unwrap_or(30);
        assert!(rounds > 0);
        let mut roots = vec![START_POS.to_string()];
        if let Some(epd) = std::env::args().nth(2) {
            for line in std::fs::read_to_string(epd).unwrap().lines().take(15) {
                roots.push(
                    line.split_whitespace()
                        .take(6)
                        .collect::<Vec<_>>()
                        .join(" "),
                );
            }
        }
        let mut random = JavaRandom::new(20261022);
        let mut walks = Vec::new();
        for fen in roots {
            for _ in 0..4 {
                let mut walk = Walk {
                    states: vec![BoardState::from_fen(&fen).unwrap()],
                    moves: Vec::new(),
                };
                for _ in 0..96 {
                    let state = walk.states.last().unwrap();
                    let moves = state.generate_legal_moves();
                    if moves.is_empty() {
                        break;
                    }
                    let mv = moves.get(random.next_int(moves.len() as i32) as usize);
                    walk.states.push(state.do_move(mv));
                    walk.moves.push(mv);
                }
                walks.push(walk);
            }
        }
        // Validate this exact timing corpus before collecting measurements.
        let mut rust = Nnue::with_kernel(Kernel::Avx512Vnni);
        let mut sparse = Nnue::with_kernel(Kernel::Avx512SparseVnni);
        let mut sparse_full = Nnue::with_kernel(Kernel::Avx512SparseVnni);
        let mut stockfish = Nnue::with_kernel(Kernel::Avx512Stockfish);
        let mut stockfish_full = Nnue::with_kernel(Kernel::Avx512Stockfish);
        let mut reference = stockfish_reference::Nnue::new();
        let mut full = Nnue::with_kernel(Kernel::Scalar);
        let mut ref_full = stockfish_reference::Nnue::new();
        let mut positions = 0;
        for walk in &walks {
            rust.reset(&walk.states[0]);
            sparse.reset(&walk.states[0]);
            stockfish.reset(&walk.states[0]);
            reference.reset(&walk.states[0]);
            for (i, &mv) in walk.moves.iter().enumerate() {
                let (before, state) = (&walk.states[i], &walk.states[i + 1]);
                rust.push_move(before, state, mv, i + 1);
                sparse.push_move(before, state, mv, i + 1);
                sparse_full.refresh(state, 0);
                stockfish.push_move(before, state, mv, i + 1);
                stockfish_full.refresh(state, 0);
                reference.push_move(before, state, mv, i + 1);
                full.refresh(state, 0);
                ref_full.refresh_full(state, 0);
                let expected = ref_full.raw(state, 0);
                assert_eq!(expected, reference.raw(state, i + 1));
                assert_eq!(expected, stockfish.raw(state, i + 1));
                assert_eq!(expected, stockfish_full.raw(state, 0));
                assert_eq!(
                    reference.full_internal(state, i + 1),
                    stockfish.full_internal(state, i + 1)
                );
                assert_eq!(
                    reference.evaluate_position(state, i + 1),
                    stockfish.evaluate_position(state, i + 1)
                );
                assert_eq!(expected, rust.raw(state, i + 1));
                assert_eq!(expected, full.raw(state, 0));
                assert_eq!(expected, sparse.raw(state, i + 1));
                assert_eq!(expected, sparse_full.raw(state, 0));
                assert_eq!(
                    reference.evaluate_position(state, i + 1),
                    sparse.evaluate_position(state, i + 1)
                );
                assert_eq!(
                    reference.full_internal(state, i + 1),
                    sparse.full_internal(state, i + 1)
                );
                assert_eq!(
                    reference.evaluate_position(state, i + 1),
                    rust.evaluate_position(state, i + 1)
                );
                assert_eq!(
                    reference.full_internal(state, i + 1),
                    rust.full_internal(state, i + 1)
                );
                for p in 0..2 {
                    assert_eq!(
                        reference.accumulator(state, i + 1, p),
                        stockfish.accumulator(state, i + 1, p)
                    );
                    assert_eq!(
                        reference.accumulator(state, i + 1, p),
                        stockfish_full.accumulator(state, 0, p)
                    );
                    assert_eq!(
                        reference.accumulator(state, i + 1, p),
                        sparse.accumulator(state, i + 1, p)
                    );
                    assert_eq!(
                        reference.accumulator(state, i + 1, p),
                        sparse_full.accumulator(state, 0, p)
                    );
                    assert_eq!(
                        reference.accumulator(state, i + 1, p),
                        rust.accumulator(state, i + 1, p)
                    );
                }
                positions += 1;
            }
        }
        println!("Network SHA256: {}", zerofish::stockfish_rust::SHA256);
        println!("Validated {positions} positions over {} legal walks: baseline/sparse/Stockfish-style/scalar/full/incremental raw/scaled scores and both accumulator arrays match.", walks.len());
        println!("Timings: median of 7 batches, {rounds} corpus repetitions/batch; warm initialization excluded.");
        let mut evaluators: Vec<(&str, Box<dyn Evaluator>)> = vec![
            (
                "Rust AVX-512/VNNI",
                Box::new(Nnue::with_kernel(Kernel::Avx512Vnni)),
            ),
            (
                "Rust sparse VNNI",
                Box::new(Nnue::with_kernel(Kernel::Avx512SparseVnni)),
            ),
            (
                "Rust Stockfish SIMD",
                Box::new(Nnue::with_kernel(Kernel::Avx512Stockfish)),
            ),
            ("Rust scalar", Box::new(Nnue::with_kernel(Kernel::Scalar))),
            ("linked .so", Box::new(stockfish_reference::Nnue::new())),
        ];
        for incremental in [false, true] {
            for (_, evaluator) in &mut evaluators {
                black_box(pass(evaluator.as_mut(), &walks, incremental, 1));
            }
            let mut timings = vec![Vec::new(); evaluators.len()];
            let mut expected = None;
            for batch in 0..7 {
                for offset in 0..evaluators.len() {
                    let index = (batch + offset) % evaluators.len();
                    let (ns, checksum) =
                        pass(evaluators[index].1.as_mut(), &walks, incremental, rounds);
                    assert_eq!(*expected.get_or_insert(checksum), checksum);
                    timings[index].push(ns);
                }
            }
            println!(
                "{}:",
                if incremental {
                    "Incremental push + inference (root resets amortized)"
                } else {
                    "Full refresh + inference"
                }
            );
            for (i, samples) in timings.iter_mut().enumerate() {
                samples.sort_by(f64::total_cmp);
                println!(
                    "  {:20} {:10.1} ns/position  [min {:.1}, max {:.1}]",
                    evaluators[i].0, samples[3], samples[0], samples[6]
                );
            }
            println!("  checksum: {}", expected.unwrap());
        }
    }
}
fn main() {
    #[cfg(feature = "stockfish-nnue")]
    comparison::run();
    #[cfg(not(feature = "stockfish-nnue"))]
    panic!("Build this comparison with --features stockfish-nnue");
}
