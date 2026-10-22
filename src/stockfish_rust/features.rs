// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2004-2026 The Stockfish developers.
// Rust adaptation of the pinned NNUE contracts; see native/UPSTREAM.json.
// Feature contracts ported from the pinned Stockfish sources in native/stockfish.
use crate::{bitboard::*, board::BoardState, types::*};
use std::sync::LazyLock;

pub const THREATS: usize = 59_808;
pub const PAIRS: usize = 4_560;
pub const PSQ: usize = 45_056;
pub const DIMENSIONS: usize = THREATS + PAIRS + PSQ;
const VALID_TARGETS: [usize; 6] = [4, 10, 8, 8, 10, 0];
const TARGET_MAP: [[i32; 6]; 6] = [
    [-1, 0, -1, 1, -1, -1],
    [0, 1, 2, 3, 4, -1],
    [0, 1, 2, 3, -1, -1],
    [0, 1, 2, 3, -1, -1],
    [0, 1, 2, 3, 4, -1],
    [-1; 6],
];
struct ThreatTables {
    offsets: [[[u32; 2]; 16]; 16],
    squares: Vec<u16>,
}
static THREAT_TABLES: LazyLock<ThreatTables> = LazyLock::new(|| {
    let mut strides = [0; 6];
    let mut squares = vec![0; 6 * 64 * 64];
    for pt in PAWN..=KING {
        let mut offset = 0;
        for from in 0..64 {
            for to in 0..64 {
                let attacks = if pt == PAWN {
                    pawn_attacks(from, usize::from(to < from))
                } else {
                    attacks(pt, from, 0)
                };
                squares[(pt * 64 + from) * 64 + to] =
                    (offset + (attacks & ((1u64 << to) - 1)).count_ones()) as u16;
            }
            if pt != PAWN || (8..56).contains(&from) {
                offset += if pt == PAWN {
                    pawn_attacks(from, WHITE)
                } else {
                    attacks(pt, from, 0)
                }
                .count_ones();
            }
        }
        strides[pt] = offset as usize;
    }
    let mut bases = [0; 16];
    let mut cumulative = 0;
    for side in [WHITE, BLACK] {
        for pt in PAWN..=KING {
            bases[make_piece(side, pt)] = cumulative;
            cumulative += VALID_TARGETS[pt] * strides[pt];
        }
    }
    assert_eq!(cumulative, THREATS);
    let mut offsets = [[[THREATS as u32; 2]; 16]; 16];
    for side in [WHITE, BLACK] {
        for pt in PAWN..=KING {
            let attacker = make_piece(side, pt);
            for target_side in [WHITE, BLACK] {
                for target_pt in PAWN..=KING {
                    let target = make_piece(target_side, target_pt);
                    let map = TARGET_MAP[pt][target_pt];
                    if map < 0 {
                        continue;
                    }
                    let value = bases[attacker]
                        + (target_side * (VALID_TARGETS[pt] / 2) + map as usize) * strides[pt];
                    offsets[attacker][target][0] = value as u32;
                    if pt != target_pt {
                        offsets[attacker][target][1] = value as u32;
                    }
                }
            }
        }
    }
    ThreatTables { offsets, squares }
});
#[inline]
fn attacks(pt: usize, square: usize, occupied: u64) -> u64 {
    match pt {
        KNIGHT => knight_attacks(square),
        BISHOP => bishop_attacks(square, occupied),
        ROOK => rook_attacks(square, occupied),
        QUEEN => queen_attacks(square, occupied),
        KING => king_attacks(square),
        _ => unreachable!(),
    }
}
#[inline]
fn pop(bb: &mut u64) -> usize {
    let square = bb.trailing_zeros() as usize;
    *bb &= *bb - 1;
    square
}

/// Sorted active rows of the serialized, unpermuted transformer, for both perspectives.
/// Threat and pawn-pair orientation intentionally differs from PSQ orientation.
pub fn active(state: &BoardState, output: &mut [Vec<u32>; 2]) {
    collect(state, [!0; 2], [!0; 2], [!0; 2], output);
}

fn collect(
    state: &BoardState,
    psq_mask: [u64; 2],
    attacker_mask: [u64; 2],
    pawn_mask: [u64; 2],
    output: &mut [Vec<u32>; 2],
) {
    for list in output.iter_mut() {
        list.clear();
    }
    let pawns = state.bitboard_of_side(WHITE, PAWN) | state.bitboard_of_side(BLACK, PAWN);
    assert_eq!(
        pawns & PAWN_FINAL_RANKS,
        0,
        "NNUE cannot encode pawns on promotion ranks"
    );
    for side in [WHITE, BLACK] {
        assert_eq!(
            state.bitboard_of_side(side, KING).count_ones(),
            1,
            "NNUE needs one king per side"
        );
    }
    let kings = [
        state.bitboard_of_side(WHITE, KING).trailing_zeros() as usize,
        state.bitboard_of_side(BLACK, KING).trailing_zeros() as usize,
    ];
    let orientations = kings.map(|sq| if sq & 7 >= 4 { 7 } else { 0 });
    // Piece-square rows.
    for perspective in [WHITE, BLACK] {
        if psq_mask[perspective] == 0 {
            continue;
        }
        let king = kings[perspective];
        let relative_king = king ^ (56 * perspective);
        let bucket = (7 - relative_king / 8) * 4 + (relative_king & 7).min(7 - (relative_king & 7));
        let queen = usize::from(state.bitboard_of_side(1 - perspective, QUEEN) != 0);
        let orient = (if king & 7 < 4 { 7 } else { 0 }) ^ (56 * perspective);
        for side in [WHITE, BLACK] {
            for pt in PAWN..=KING {
                let piece_offset = if pt == KING {
                    640
                } else {
                    (2 * pt + (side ^ perspective)) * 64
                };
                let base = THREATS + PAIRS + bucket * 1408 + queen * 704 + piece_offset;
                let mut bb = state.bitboard_of_side(side, pt) & psq_mask[perspective];
                while bb != 0 {
                    output[perspective].push((base + (pop(&mut bb) ^ orient)) as u32);
                }
            }
        }
    }
    // All attacks/defences on occupied squares, including pinned attackers.
    let tables = &*THREAT_TABLES;
    for side in [WHITE, BLACK] {
        for pt in PAWN..KING {
            let attacker = make_piece(side, pt);
            let mut pieces = state.bitboard_of(attacker) & (attacker_mask[0] | attacker_mask[1]);
            while pieces != 0 {
                let from = pop(&mut pieces);
                let mut targets = (if pt == PAWN {
                    pawn_attacks(from, side)
                } else {
                    attacks(pt, from, state.all_pieces())
                }) & state.all_pieces();
                while targets != 0 {
                    let to = pop(&mut targets);
                    let target = state.piece_at(to);
                    if TARGET_MAP[pt][type_of(target)] < 0 {
                        continue;
                    }
                    for perspective in [WHITE, BLACK] {
                        if attacker_mask[perspective] & (1u64 << from) == 0 {
                            continue;
                        }
                        let orientation = orientations[perspective] ^ (56 * perspective);
                        let a = attacker ^ (8 * perspective);
                        let b = target ^ (8 * perspective);
                        let f = from ^ orientation;
                        let t = to ^ orientation;
                        let index = tables.offsets[a][b][usize::from(f < t)]
                            + tables.squares[(pt * 64 + f) * 64 + t] as u32;
                        if index < THREATS as u32 {
                            output[perspective].push(index);
                        }
                    }
                }
            }
        }
    }
    // Unordered pawn pairs on the same or neighbouring files, on any legal pawn rank.
    if pawn_mask == [0; 2] {
        return;
    }
    let mut pawns = state.bitboard_of_side(WHITE, PAWN) | state.bitboard_of_side(BLACK, PAWN);
    while pawns != 0 {
        let from = pop(&mut pawns);
        let mut others = pawns;
        while others != 0 {
            let to = pop(&mut others);
            if (from & 7).abs_diff(to & 7) > 1 {
                continue;
            }
            for perspective in [WHITE, BLACK] {
                if pawn_mask[perspective] & ((1u64 << from) | (1u64 << to)) == 0 {
                    continue;
                }
                let orient = orientations[perspective] ^ (56 * perspective);
                let a = 48 * (side_of(state.piece_at(from)) ^ perspective) + (from ^ orient) - 8;
                let b = 48 * (side_of(state.piece_at(to)) ^ perspective) + (to ^ orient) - 8;
                let (hi, lo) = (a.max(b), a.min(b));
                output[perspective].push((THREATS + hi * (hi - 1) / 2 + lo) as u32);
            }
        }
    }
}

/// Re-encode only feature groups affected by this board transition. Unchanged
/// attackers are omitted; discovered slider attacks are included in both states.
/// King/queen context changes refresh only the feature groups whose indexing changes.
pub fn changed(
    before: &BoardState,
    after: &BoardState,
    old: &mut [Vec<u32>; 2],
    new: &mut [Vec<u32>; 2],
) {
    let mut squares = 0;
    for side in [WHITE, BLACK] {
        for pt in PAWN..=KING {
            squares |= before.bitboard_of_side(side, pt) ^ after.bitboard_of_side(side, pt);
        }
    }
    if squares == 0 {
        for list in old.iter_mut().chain(new.iter_mut()) {
            list.clear();
        }
        return;
    }
    let mut affected = squares;
    let mut changed_squares = squares;
    while changed_squares != 0 {
        let square = pop(&mut changed_squares);
        let knights = before.bitboard_of_side(WHITE, KNIGHT)
            | before.bitboard_of_side(BLACK, KNIGHT)
            | after.bitboard_of_side(WHITE, KNIGHT)
            | after.bitboard_of_side(BLACK, KNIGHT);
        let diagonal = bishop_attacks(square, 0);
        let straight = rook_attacks(square, 0);
        affected |= knight_attacks(square) & knights;
        for state in [before, after] {
            for side in [WHITE, BLACK] {
                affected |= diagonal
                    & (state.bitboard_of_side(side, BISHOP) | state.bitboard_of_side(side, QUEEN));
                affected |= straight
                    & (state.bitboard_of_side(side, ROOK) | state.bitboard_of_side(side, QUEEN));
                affected |= pawn_attacks(square, 1 - side) & state.bitboard_of_side(side, PAWN);
            }
        }
    }
    let mut psq = [squares; 2];
    let mut threats = [affected; 2];
    let mut pawns = [0; 2];
    for side in [WHITE, BLACK] {
        pawns[0] |= before.bitboard_of_side(side, PAWN) ^ after.bitboard_of_side(side, PAWN);
    }
    pawns[1] = pawns[0];
    for perspective in [WHITE, BLACK] {
        let old_king = before.bitboard_of_side(perspective, KING).trailing_zeros();
        let new_king = after.bitboard_of_side(perspective, KING).trailing_zeros();
        let old_queen = before.bitboard_of_side(1 - perspective, QUEEN) != 0;
        let new_queen = after.bitboard_of_side(1 - perspective, QUEEN) != 0;
        if old_king != new_king || old_queen != new_queen {
            psq[perspective] = !0;
        }
        if (old_king & 7 >= 4) != (new_king & 7 >= 4) {
            threats[perspective] = !0;
            pawns[perspective] = !0;
        }
    }
    collect(before, psq, threats, pawns, old);
    collect(after, psq, threats, pawns, new);
    for list in old.iter_mut().chain(new.iter_mut()) {
        list.sort_unstable();
    }
}
