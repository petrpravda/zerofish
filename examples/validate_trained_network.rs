//! Cross-check an exported network against trainer integer inference and Rust kernels.
fn main() {
    use std::{env, fs};
    use zerofish::{
        board::BoardState,
        stockfish_rust::{Kernel, Nnue},
    };
    let input = env::args()
        .nth(1)
        .expect("usage: validate_trained_network cross-check.tsv");
    let mut scalar = Nnue::with_kernel(Kernel::Scalar);
    let mut simd = Nnue::new();
    let mut cases = 0;
    let mut transitions = 0;
    for line in fs::read_to_string(input).unwrap().lines() {
        let (fen, score) = line.split_once('\t').expect("FEN<TAB>raw score");
        let state = BoardState::from_fen(fen).expect("valid FEN");
        let expected: i32 = score.parse().unwrap();
        scalar.reset(&state);
        simd.reset(&state);
        assert_eq!(expected, scalar.raw(&state, 0), "trainer/scalar: {fen}");
        assert_eq!(expected, simd.raw(&state, 0), "trainer/SIMD: {fen}");
        for p in 0..2 {
            assert_eq!(
                scalar.accumulator(&state, 0, p),
                simd.accumulator(&state, 0, p)
            );
        }
        if cases < 64 {
            for mv in state.generate_legal_moves().iter() {
                let child = state.do_move(mv);
                simd.push_move(&state, &child, mv, 1);
                scalar.refresh(&child, 0);
                assert_eq!(scalar.raw(&child, 0), simd.raw(&child, 1));
                for p in 0..2 {
                    assert_eq!(
                        scalar.accumulator(&child, 0, p),
                        simd.accumulator(&child, 1, p)
                    );
                }
                transitions += 1;
            }
        }
        cases += 1;
    }
    assert!(cases > 0);
    println!("Network SHA256: {}", zerofish::stockfish_rust::identity().unwrap().1);
    println!("Validated {cases} trainer/scalar/SIMD positions and {transitions} incremental sibling transitions, including both accumulator arrays.");
}
