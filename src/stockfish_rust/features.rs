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

#[derive(Clone, Copy)]
struct Context {
    perspective: usize,
    psq_base: usize,
    psq_flip: usize,
    threat_flip: usize,
}
impl Context {
    fn new(state: &BoardState, perspective: usize) -> Self {
        let king = state.bitboard_of_side(perspective, KING).trailing_zeros() as usize;
        let relative = king ^ (56 * perspective);
        let bucket = (7 - relative / 8) * 4 + (relative & 7).min(7 - (relative & 7));
        Self {
            perspective,
            psq_base: THREATS
                + PAIRS
                + bucket * 1408
                + usize::from(state.bitboard_of_side(1 - perspective, QUEEN) != 0) * 704,
            psq_flip: (if king & 7 < 4 { 7 } else { 0 }) ^ (56 * perspective),
            threat_flip: (if king & 7 >= 4 { 7 } else { 0 }) ^ (56 * perspective),
        }
    }
    #[inline(always)]
    fn psq(self, piece: usize, square: usize) -> u32 {
        let pt = type_of(piece);
        let offset = if pt == KING {
            640
        } else {
            (2 * pt + (side_of(piece) ^ self.perspective)) * 64
        };
        (self.psq_base + offset + (square ^ self.psq_flip)) as u32
    }
    #[inline(always)]
    fn threat(
        self,
        tables: &ThreatTables,
        piece: usize,
        from: usize,
        target: usize,
        to: usize,
    ) -> u32 {
        let f = from ^ self.threat_flip;
        let t = to ^ self.threat_flip;
        tables.offsets[piece ^ (8 * self.perspective)][target ^ (8 * self.perspective)]
            [usize::from(f < t)]
            + tables.squares[(type_of(piece) * 64 + f) * 64 + t] as u32
    }
    #[inline(always)]
    fn pair(self, state: &BoardState, from: usize, to: usize) -> u32 {
        let a =
            48 * (side_of(state.piece_at(from)) ^ self.perspective) + (from ^ self.threat_flip) - 8;
        let b = 48 * (side_of(state.piece_at(to)) ^ self.perspective) + (to ^ self.threat_flip) - 8;
        let (hi, lo) = (a.max(b), a.min(b));
        (THREATS + hi * (hi - 1) / 2 + lo) as u32
    }
}
#[inline(always)]
fn threat_targets(state: &BoardState, square: usize, piece: usize) -> u64 {
    if piece == NONE || type_of(piece) == KING {
        return 0;
    }
    let pt = type_of(piece);
    let knights = state.bitboard_of_side(0, KNIGHT) | state.bitboard_of_side(1, KNIGHT);
    let rooks = state.bitboard_of_side(0, ROOK) | state.bitboard_of_side(1, ROOK);
    if pt == PAWN {
        return pawn_attacks(square, side_of(piece)) & (knights | rooks);
    }
    let kings = state.bitboard_of_side(0, KING) | state.bitboard_of_side(1, KING);
    let mut targets = state.all_pieces() & !kings;
    if pt == BISHOP || pt == ROOK {
        targets &= !(state.bitboard_of_side(0, QUEEN) | state.bitboard_of_side(1, QUEEN));
    }
    attacks(pt, square, state.all_pieces()) & targets
}
static PAWN_BANDS: LazyLock<[u64; 64]> = LazyLock::new(|| {
    std::array::from_fn(|square| {
        let file = 0x0101_0101_0101_0101u64 << (square & 7);
        let left = if square & 7 > 0 { file >> 1 } else { 0 };
        let right = if square & 7 < 7 { file << 1 } else { 0 };
        (file | left | right) & !PAWN_FINAL_RANKS & !(1 << square)
    })
});
fn pair_changes(
    state: &BoardState,
    changed: u64,
    context: Context,
    output: &mut Vec<u32>,
    weights: &[i8],
) {
    let pawns = state.bitboard_of_side(0, PAWN) | state.bitboard_of_side(1, PAWN);
    let mut dirty = pawns & changed;
    while dirty != 0 {
        let from = pop(&mut dirty);
        let mut others = pawns & PAWN_BANDS[from];
        // A pair with two changed endpoints is emitted exactly once.
        others &= !(changed & ((1u64 << from) - 1));
        while others != 0 {
            emit(output, context.pair(state, from, pop(&mut others)), weights);
        }
    }
}

/// Exact dirty feature lists: unchanged edges never become indices, so no
/// per-move sorting/merging is needed. Includes discovered attacks and changed
/// target identities, as in Stockfish's dirty threat/pawn-pair bookkeeping.
#[cfg_attr(
    target_arch = "x86_64",
    target_feature(enable = "avx512f,avx512bw,avx512vnni,popcnt")
)]
pub unsafe fn dirty(
    before: &BoardState,
    after: &BoardState,
    removed: &mut [Vec<u32>; 2],
    added: &mut [Vec<u32>; 2],
    weights: &[i8],
) {
    for list in removed.iter_mut().chain(added.iter_mut()) {
        list.clear();
    }
    let mut squares = 0;
    for side in [WHITE, BLACK] {
        assert_eq!(
            after.bitboard_of_side(side, KING).count_ones(),
            1,
            "NNUE needs one king per side"
        );
        for pt in PAWN..=KING {
            squares |= before.bitboard_of_side(side, pt) ^ after.bitboard_of_side(side, pt);
        }
    }
    let pawn_changes = (before.bitboard_of_side(0, PAWN) ^ after.bitboard_of_side(0, PAWN))
        | (before.bitboard_of_side(1, PAWN) ^ after.bitboard_of_side(1, PAWN));
    assert_eq!(
        (after.bitboard_of_side(0, PAWN) | after.bitboard_of_side(1, PAWN)) & PAWN_FINAL_RANKS,
        0,
        "NNUE cannot encode pawns on promotion ranks"
    );
    if squares == 0 {
        return;
    }
    let old = [Context::new(before, 0), Context::new(before, 1)];
    let new = [Context::new(after, 0), Context::new(after, 1)];
    let mirror_changed = [
        old[0].threat_flip != new[0].threat_flip,
        old[1].threat_flip != new[1].threat_flip,
    ];
    for p in 0..2 {
        let psq_context_changed =
            old[p].psq_base != new[p].psq_base || old[p].psq_flip != new[p].psq_flip;
        let mask = if psq_context_changed { !0 } else { squares };
        let mut bb = before.all_pieces() & mask;
        while bb != 0 {
            let sq = pop(&mut bb);
            emit(
                &mut removed[p],
                old[p].psq(before.piece_at(sq), sq),
                weights,
            );
        }
        let mut bb = after.all_pieces() & mask;
        while bb != 0 {
            let sq = pop(&mut bb);
            emit(&mut added[p], new[p].psq(after.piece_at(sq), sq), weights);
        }
        let pairs = if mirror_changed[p] { !0 } else { pawn_changes };
        if pairs != 0 {
            pair_changes(before, pairs, old[p], &mut removed[p], weights);
            pair_changes(after, pairs, new[p], &mut added[p], weights);
        }
    }
    let mut affected = squares;
    if mirror_changed[0] || mirror_changed[1] {
        affected = before.all_pieces() | after.all_pieces();
    } else {
        let knights = before.bitboard_of_side(0, KNIGHT)
            | before.bitboard_of_side(1, KNIGHT)
            | after.bitboard_of_side(0, KNIGHT)
            | after.bitboard_of_side(1, KNIGHT);
        let mut changed_squares = squares;
        while changed_squares != 0 {
            let sq = pop(&mut changed_squares);
            affected |= knight_attacks(sq) & knights;
            for state in [before, after] {
                let diagonal = bishop_attacks(sq, state.all_pieces());
                let straight = rook_attacks(sq, state.all_pieces());
                for side in 0..2 {
                    affected |= diagonal & state.diagonal_sliders(side);
                    affected |= straight & state.orthogonal_sliders(side);
                    affected |= pawn_attacks(sq, 1 - side) & state.bitboard_of_side(side, PAWN);
                }
            }
        }
    }
    let tables = &*THREAT_TABLES;
    while affected != 0 {
        let from = pop(&mut affected);
        let a = before.piece_at(from);
        let b = after.piece_at(from);
        let old_targets = threat_targets(before, from, a);
        let new_targets = threat_targets(after, from, b);
        for p in 0..2 {
            let same_attacker = a == b && !mirror_changed[p];
            let mut r = if same_attacker {
                old_targets & (!new_targets | squares)
            } else {
                old_targets
            };
            let mut n = if same_attacker {
                new_targets & (!old_targets | squares)
            } else {
                new_targets
            };
            while r != 0 {
                let to = pop(&mut r);
                let row = old[p].threat(tables, a, from, before.piece_at(to), to);
                if row < THREATS as u32 {
                    emit(&mut removed[p], row, weights);
                }
            }
            while n != 0 {
                let to = pop(&mut n);
                let row = new[p].threat(tables, b, from, after.piece_at(to), to);
                if row < THREATS as u32 {
                    emit(&mut added[p], row, weights);
                }
            }
        }
    }
}

#[inline(always)]
fn emit(output: &mut Vec<u32>, row: u32, weights: &[i8]) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        // The validated feature index addresses a complete 1024-byte row.
        let offset = row as usize * 1024;
        assert!(
            offset + 1024 <= weights.len(),
            "NNUE feature row outside network"
        );
        let ptr = weights.as_ptr().add(offset);
        for offset in (0..1024).step_by(128) {
            std::arch::x86_64::_mm_prefetch::<{ std::arch::x86_64::_MM_HINT_T0 }>(
                ptr.add(offset).cast(),
            );
        }
    }
    output.push(row);
}
