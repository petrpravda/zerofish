//! Static exchange evaluation.

use crate::bitboard::*;
use crate::board::BoardState;
use crate::moves::Move;
use crate::types::*;

const VALUES: [i32; 6] = [100, 320, 330, 500, 900, 20000];

#[inline(always)]
pub fn value(piece_type: usize) -> i32 {
    VALUES[piece_type]
}

/// Whether the exchange on the target square gains at least `threshold`; ignores pins.
pub fn see(state: &BoardState, mv: Move, threshold: i32) -> bool {
    let us = state.side_to_play();
    let from = mv.from();
    let to = mv.to();
    let en_passant = mv.flags() == Move::EN_PASSANT;
    let victim = if en_passant {
        Some(PAWN)
    } else if state.piece_at(to) == NONE {
        None
    } else {
        Some(state.piece_type_at(to))
    };
    let mut score = victim.map_or(0, |v| VALUES[v]) - threshold;
    if mv.is_promotion() {
        score += VALUES[mv.piece_type()] - VALUES[PAWN];
    }
    if score < 0 {
        return false;
    }
    let next = if mv.is_promotion() { mv.piece_type() } else { state.piece_type_at(from) };
    score -= VALUES[next];
    if score >= 0 {
        return true;
    }
    let mut occupied = state.all_pieces() & !(1u64 << from) | 1u64 << to;
    if en_passant {
        occupied &= !(1u64 << (to as i32 + if us == 0 { -8 } else { 8 }));
    }
    let diagonal = state.diagonal_sliders(0) | state.diagonal_sliders(1);
    let orthogonal = state.orthogonal_sliders(0) | state.orthogonal_sliders(1);
    let mut attackers = (pawn_attacks(to, 1) & state.bitboard_of_side(0, PAWN)
        | pawn_attacks(to, 0) & state.bitboard_of_side(1, PAWN)
        | knight_attacks(to) & (state.bitboard_of_side(0, KNIGHT) | state.bitboard_of_side(1, KNIGHT))
        | king_attacks(to) & (state.bitboard_of_side(0, KING) | state.bitboard_of_side(1, KING))
        | bishop_attacks(to, occupied) & diagonal
        | rook_attacks(to, occupied) & orthogonal)
        & occupied;
    let mut side = 1 - us;
    loop {
        let ours = attackers & state.all_pieces_of(side);
        if ours == 0 {
            break;
        }
        let mut piece_type = PAWN;
        while ours & state.bitboard_of_side(side, piece_type) == 0 {
            piece_type += 1;
        }
        let candidates = ours & state.bitboard_of_side(side, piece_type);
        occupied ^= candidates & candidates.wrapping_neg();
        if piece_type == PAWN || piece_type == BISHOP || piece_type == QUEEN {
            attackers |= bishop_attacks(to, occupied) & diagonal;
        }
        if piece_type == ROOK || piece_type == QUEEN {
            attackers |= rook_attacks(to, occupied) & orthogonal;
        }
        attackers &= occupied;
        score = -score - 1 - VALUES[piece_type];
        side = 1 - side;
        if score >= 0 {
            if piece_type == KING && attackers & state.all_pieces_of(side) != 0 {
                side = 1 - side;
            }
            break;
        }
    }
    us != side
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_exchanges() {
        // pawn takes a defended knight: wins material
        let state = BoardState::from_fen("4k3/8/3p4/4n3/3P4/8/8/4K3 w - - 0 1").unwrap();
        let capture = state.find_legal_move("d4e5").unwrap();
        assert!(see(&state, capture, 0));
        assert!(!see(&state, capture, 300));
        // queen takes a pawn defended by a pawn: loses material
        let state = BoardState::from_fen("4k3/8/3p4/4p3/8/8/4Q3/4K3 w - - 0 1").unwrap();
        let capture = state.find_legal_move("e2e5").unwrap();
        assert!(!see(&state, capture, 0));
    }
}
