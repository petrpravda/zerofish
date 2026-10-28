use super::*;
use crate::{fen::START_POS, random::JavaRandom, types::*};
fn compare(incremental: &mut Nnue, fresh: &mut Nnue, state: &BoardState, ply: usize) {
    fresh.refresh(state, 0);
    assert_eq!(
        fresh.raw(state, 0),
        incremental.raw(state, ply),
        "{}",
        state.to_fen()
    );
    assert_eq!(fresh.evaluate(state, 0), incremental.evaluate(state, ply));
    assert_eq!(
        fresh.full_internal(state, 0),
        incremental.full_internal(state, ply)
    );
    for p in 0..2 {
        assert_eq!(
            fresh.accumulator(state, 0, p),
            incremental.accumulator(state, ply, p)
        );
    }
}
#[test]
fn incremental_special_moves_and_walks_match_refresh() {
    let mut incremental = Nnue::new();
    let mut fresh = Nnue::with_kernel(Kernel::Scalar);
    for fen in [
        START_POS,
        "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
        "4k3/P7/8/3pP3/8/8/8/4K3 w - d6 0 1",
        "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
        "4k3/8/8/8/8/3K4/8/8 w - - 99 1",
        "4k3/8/8/8/3Pp3/8/p7/1R2K3 b - d3 0 1",
    ] {
        let root = BoardState::from_fen(fen).unwrap();
        incremental.reset(&root);
        for mv in root.generate_legal_moves().iter() {
            let child = root.do_move(mv);
            incremental.push_move(&root, &child, mv, 1);
            compare(&mut incremental, &mut fresh, &child, 1);
            compare(&mut incremental, &mut fresh, &root, 0);
        }
        let null_state = root.do_null_move();
        incremental.push(&root, &null_state, 1);
        compare(&mut incremental, &mut fresh, &null_state, 1);
    }
    let mut random = JavaRandom::new(73);
    let mut state = BoardState::from_fen(START_POS).unwrap();
    incremental.reset(&state);
    for ply in 1..=100 {
        let moves = state.generate_legal_moves();
        if moves.is_empty() {
            break;
        }
        let mv = moves.get(random.next_int(moves.len() as i32) as usize);
        let child = state.do_move(mv);
        incremental.push_move(&state, &child, mv, ply);
        compare(&mut incremental, &mut fresh, &child, ply);
        state = child;
    }
}

#[test]
fn chess960_incremental_castling_matches_refresh() {
    let mut incremental = Nnue::new();
    let mut fresh = Nnue::with_kernel(Kernel::Scalar);
    for side in [WHITE, BLACK] {
        for king in 1..=6 {
            for rook in 0..8 {
                if king == rook {
                    continue;
                }
                let mut rank = String::new();
                let mut empty = 0;
                for file in 0..8 {
                    if file == king || file == rook {
                        if empty != 0 {
                            rank.push_str(&empty.to_string());
                            empty = 0;
                        }
                        rank.push(if file == king { 'K' } else { 'R' });
                    } else {
                        empty += 1;
                    }
                }
                if empty != 0 {
                    rank.push_str(&empty.to_string());
                }
                let fen = if side == WHITE {
                    format!("8/7k/8/8/8/8/8/{rank} w {} - 0 1", (b'A' + rook) as char)
                } else {
                    format!(
                        "{}/8/8/8/8/8/7K/8 b {} - 0 1",
                        rank.to_ascii_lowercase(),
                        (b'a' + rook) as char
                    )
                };
                let before = BoardState::from_fen(&fen).unwrap();
                // Stockfish rejects synthetic roots where the opponent king is already attacked.
                if before.do_null_move().is_king_attacked() {
                    continue;
                }
                let castle = before
                    .generate_legal_moves()
                    .iter()
                    .find(|mv| mv.is_castling())
                    .unwrap();
                incremental.reset(&before);
                let after = before.do_move(castle);
                incremental.push_move(&before, &after, castle, 1);
                compare(&mut incremental, &mut fresh, &after, 1);
                compare(&mut incremental, &mut fresh, &before, 0);
            }
        }
    }
}

#[test]
fn reject_truncated_or_incompatible_networks() {
    assert!(network::Network::read(&[]).is_err());
    assert!(network::Network::read(&[0; 32]).is_err());
}

#[test]
fn simd_kernels_match_scalar_at_arithmetic_boundaries() {
    if !Kernel::avx512_available() {
        return;
    }
    let mut random = JavaRandom::new(129);
    let mut acc = [0i16; HIDDEN];
    for _ in 0..64 {
        for value in &mut acc {
            *value = random.next_int(65536) as i16;
        }
        let mut scalar = [0; 512];
        let mut simd = scalar;
        Kernel::Scalar.transform(&acc, &mut scalar);
        Kernel::Avx512Vnni.transform(&acc, &mut simd);
        assert_eq!(scalar, simd);
        let input = std::array::from_fn(|_| random.next_long() as i32);
        for shift in [6, 7] {
            let mut scalar = [0; 64];
            let mut simd = scalar;
            Kernel::Scalar.activate(&input, &mut scalar, shift);
            Kernel::Avx512Vnni.activate(&input, &mut simd, shift);
            assert_eq!(scalar, simd);
        }
    }
    for stack in &NETWORK.stacks {
        for layer in [&stack.first, &stack.second, &stack.last] {
            let input: Vec<_> = (0..layer.inputs)
                .map(|_| random.next_int(128) as u8)
                .collect();
            let mut scalar = vec![0; layer.outputs];
            let mut simd = scalar.clone();
            Kernel::Scalar.affine(layer, &input, &mut scalar);
            for kernel in [Kernel::Avx512Vnni, Kernel::Avx512Stockfish] {
                kernel.affine(layer, &input, &mut simd);
                assert_eq!(scalar, simd);
            }
        }
    }
    let weights: Vec<_> = (0..4 * HIDDEN)
        .map(|_| random.next_int(256) as i8)
        .collect();
    let mut scalar = acc;
    let mut simd = acc;
    Kernel::Scalar.update(&mut scalar, &weights, &[0, 1], &[2, 3]);
    Kernel::Avx512Vnni.update(&mut simd, &weights, &[0, 1], &[2, 3]);
    assert_eq!(scalar, simd);
}

#[cfg(feature = "stockfish-nnue")]
#[test]
fn full_and_incremental_match_linked_reference() {
    use crate::stockfish_reference::Nnue as Reference;
    fn assert_position(
        rust: &mut Nnue,
        full: &mut Nnue,
        reference: &mut Reference,
        reference_full: &mut Reference,
        state: &BoardState,
        ply: usize,
    ) {
        full.refresh(state, 0);
        reference_full.refresh_full(state, 0);
        let expected = reference_full.raw(state, 0);
        assert_eq!(
            expected,
            reference.raw(state, ply),
            "reference incremental: {}",
            state.to_fen()
        );
        assert_eq!(
            expected,
            full.raw(state, 0),
            "Rust full: {}",
            state.to_fen()
        );
        assert_eq!(
            expected,
            rust.raw(state, ply),
            "Rust incremental: {}",
            state.to_fen()
        );
        assert_eq!(
            reference.evaluate_position(state, ply),
            rust.evaluate_position(state, ply)
        );
        assert_eq!(reference.evaluate(state, ply), rust.evaluate(state, ply));
        assert_eq!(
            reference.full_internal(state, ply),
            rust.full_internal(state, ply)
        );
        for p in 0..2 {
            let expected = reference.accumulator(state, ply, p);
            assert_eq!(
                expected,
                rust.accumulator(state, ply, p),
                "Rust accumulator p={p}: {}",
                state.to_fen()
            );
            assert_eq!(expected, full.accumulator(state, 0, p));
        }
    }
    let kernels: &[Kernel] = if Kernel::avx512_available() {
        &[
            Kernel::Avx512Vnni,
            Kernel::Avx512SparseVnni,
            Kernel::Avx512Stockfish,
        ]
    } else {
        &[Kernel::Scalar]
    };
    for &kernel in kernels {
        let mut rust = Nnue::with_kernel(kernel);
        let mut full = Nnue::with_kernel(Kernel::Scalar);
        let mut reference = Reference::new();
        let mut reference_full = Reference::new();
        let fens = [
            START_POS,
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            "4k3/P7/8/3pP3/8/8/8/4K3 w - d6 0 1",
            "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
            "4k3/8/8/8/3Pp3/8/p7/1R2K3 b - d3 0 1",
            "4k3/8/8/8/8/8/8/1R1K2R1 w GB - 0 1",
            "1r1k2r1/8/8/8/8/8/8/7K b gb - 0 1",
            "8/8/4k3/8/4K3/4P3/8/8 w - - 99 60",
            "r2qk2r/pp1n1ppp/2p1bn2/3p4/3P4/2NBPN2/PPQ2PPP/R3K2R w KQkq - 5 10",
        ];
        let mut checked = 0;
        for fen in fens {
            let root = BoardState::from_fen(fen).unwrap();
            rust.reset(&root);
            reference.reset(&root);
            assert_position(
                &mut rust,
                &mut full,
                &mut reference,
                &mut reference_full,
                &root,
                0,
            );
            for mv in root.generate_legal_moves().iter() {
                let child = root.do_move(mv);
                rust.push_move(&root, &child, mv, 1);
                reference.push_move(&root, &child, mv, 1);
                assert_position(
                    &mut rust,
                    &mut full,
                    &mut reference,
                    &mut reference_full,
                    &child,
                    1,
                );
                checked += 1;
            }
        }
        let mut random = JavaRandom::new(20261022);
        for _ in 0..24 {
            let mut state = BoardState::from_fen(START_POS).unwrap();
            rust.reset(&state);
            reference.reset(&state);
            for ply in 1..=110 {
                let moves = state.generate_legal_moves();
                if moves.is_empty() {
                    break;
                }
                for i in 0..moves.len().min(2) {
                    let mv = moves.get(i);
                    let child = state.do_move(mv);
                    rust.push_move(&state, &child, mv, ply);
                    reference.push_move(&state, &child, mv, ply);
                    assert_position(
                        &mut rust,
                        &mut full,
                        &mut reference,
                        &mut reference_full,
                        &child,
                        ply,
                    );
                    checked += 1;
                }
                if !state.is_king_attacked() {
                    let child = state.do_null_move();
                    rust.push(&state, &child, ply);
                    reference.push(&state, &child, ply);
                    assert_position(
                        &mut rust,
                        &mut full,
                        &mut reference,
                        &mut reference_full,
                        &child,
                        ply,
                    );
                    checked += 1;
                }
                let mv = moves.get(random.next_int(moves.len() as i32) as usize);
                let child = state.do_move(mv);
                rust.push_move(&state, &child, mv, ply);
                reference.push_move(&state, &child, mv, ply);
                assert_position(
                    &mut rust,
                    &mut full,
                    &mut reference,
                    &mut reference_full,
                    &child,
                    ply,
                );
                state = child;
                checked += 1;
            }
        }
        println!("Exact full/incremental reference comparisons ({kernel:?}): {checked} positions, both 1024-lane accumulators and raw/scaled scores");
    }
}

#[test]
fn invalid_positions_and_feature_rows_fail_before_simd_reads() {
    let state = BoardState::from_fen("4k3/8/8/8/8/8/8/P3K3 w - - 0 1").unwrap();
    let mut evaluator = Nnue::new();
    assert!(std::panic::catch_unwind(
        std::panic::AssertUnwindSafe(|| evaluator.refresh(&state, 0))
    )
    .is_err());
    let mut accumulator = [0; HIDDEN];
    let weights = vec![0; HIDDEN];
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| Kernel::best().update(
            &mut accumulator,
            &weights,
            &[],
            &[1]
        )))
        .is_err()
    );
}

#[test]
fn sparse_vnni_matches_scalar_for_empty_odd_dense_and_boundary_inputs() {
    if !Kernel::avx512_available() {
        return;
    }
    for kernel in [Kernel::Avx512SparseVnni, Kernel::Avx512Stockfish] {
        let mut random = JavaRandom::new(20261023);
        for _ in 0..64 {
            let acc = std::array::from_fn(|_| random.next_int(65536) as i16);
            let mut expected = [0; 512];
            let mut actual = expected;
            Kernel::Scalar.transform(&acc, &mut expected);
            kernel.transform(&acc, &mut actual);
            assert_eq!(expected, actual);
        }
        for stack in &NETWORK.stacks {
            let layer = &stack.first;
            // Exercise empty lists, pair boundaries, vector boundaries, odd remainders
            // and all 256 groups. Include high-bit bytes to check signed mask handling.
            for groups in [0, 1, 2, 3, 15, 16, 17, 127, 128, 129, 255, 256] {
                let mut input = [0u8; HIDDEN];
                for group in 0..groups {
                    let index = (group * 73 + 251) % 256;
                    for k in 0..4 {
                        input[index * 4 + k] = (random.next_int(255) + 1) as u8;
                    }
                }
                let mut expected = [0; 32];
                let mut actual = expected;
                Kernel::Scalar.affine(layer, &input, &mut expected);
                kernel.affine(layer, &input, &mut actual);
                assert_eq!(expected, actual, "active groups={groups}");
            }
        }
    }
}

#[test]
fn stockfish_paired_updates_preserve_wrapping_arithmetic_and_source() {
    if !Kernel::avx512_available() {
        return;
    }
    let mut random = JavaRandom::new(20261024);
    let mut canonical = vec![0i8; 5 * HIDDEN];
    for value in &mut canonical {
        *value = random.next_int(256) as i8;
    }
    let mut paired = canonical.clone();
    let order = [0, 4, 1, 5, 2, 6, 3, 7];
    for (input, output) in canonical.chunks_exact(64).zip(paired.chunks_exact_mut(64)) {
        for (block, &from) in order.iter().enumerate() {
            output[block * 8..block * 8 + 8].copy_from_slice(&input[from * 8..from * 8 + 8]);
        }
    }
    let source = std::array::from_fn(|_| random.next_int(65536) as i16);
    for (removed, added) in [
        (&[][..], &[][..]),
        (&[0][..], &[1, 2][..]),
        (&[0, 1][..], &[2, 3, 4][..]),
        (&[0, 1, 2][..], &[3][..]),
    ] {
        let mut expected = source;
        Kernel::Scalar.update(&mut expected, &canonical, removed, added);
        let mut actual = [0; HIDDEN];
        Kernel::Avx512Stockfish.update_from(&source, &mut actual, &paired, removed, added);
        assert_eq!(expected, actual);
        let mut in_place = source;
        Kernel::Avx512Stockfish.update(&mut in_place, &paired, removed, added);
        assert_eq!(expected, in_place);
    }
}
