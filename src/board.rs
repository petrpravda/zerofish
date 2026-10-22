use std::fmt;

use crate::bitboard::*;
use crate::fen;
use crate::moves::{Move, MoveList};
use crate::pst::PST;
use crate::types::*;
use crate::zobrist::ZOBRIST;

pub const TOTAL_PHASE: i32 = 24;
pub const PIECE_PHASES: [i32; 6] = [0, 1, 1, 2, 4, 0];
const CHESSBOARD_LINE: &str = "+---+---+---+---+---+---+---+---+\n";

/// A position. States are immutable once built: a move produces a new state.
/// Pins, checkers and the position key are computed eagerly when a state is built.
#[derive(Clone)]
pub struct BoardState {
    // indexed by piece; the unused slots 7 and 15 hold the white and black occupancy
    bitboards: [u64; 16],
    items: [u8; 64],
    side: usize,
    // incremental Zobrist hash of pieces, side and en passant square
    zobrist: u64,
    key: u64,
    checkers: u64,
    pinned: u64,
    null_branch: bool,
    pub full_move_normalized: i32,
    pub half_move_clock: i32,
    pub phase: i32,
    mg: i32,
    eg: i32,
    /// Squares any piece moved from or to; castling rights are derived from it.
    pub movements: u64,
    pub en_passant: u64,
}

impl BoardState {
    pub fn new(items: &[usize; 64], side: usize, movements: u64, en_passant: u64, half_move_clock: i32, full_move_count: i32) -> Self {
        let mut state = BoardState {
            bitboards: [0; 16],
            items: [NONE as u8; 64],
            side,
            zobrist: 0,
            key: 0,
            checkers: 0,
            pinned: 0,
            null_branch: false,
            full_move_normalized: (full_move_count - 1) * 2 + if side == WHITE { 0 } else { 1 },
            half_move_clock,
            phase: TOTAL_PHASE,
            mg: 0,
            eg: 0,
            movements,
            en_passant,
        };
        for (square, &item) in items.iter().enumerate() {
            if item != NONE {
                state.set_piece_at(item, square);
            }
        }
        if side == BLACK {
            state.zobrist ^= ZOBRIST.side;
        }
        if en_passant != 0 {
            state.zobrist ^= ZOBRIST.en_passant[lsb(en_passant) & 7];
        }
        state.finish();
        state
    }

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        fen::from_fen(fen)
    }

    pub fn start_pos() -> Self {
        fen::from_fen(fen::START_POS).expect("start position")
    }

    pub fn to_fen(&self) -> String {
        fen::to_fen(self)
    }

    #[inline(always)]
    pub fn piece_at(&self, square: usize) -> usize {
        self.items[square] as usize
    }

    #[inline(always)]
    pub fn piece_type_at(&self, square: usize) -> usize {
        type_of(self.items[square] as usize)
    }

    fn set_piece_at(&mut self, piece: usize, square: usize) {
        self.phase -= PIECE_PHASES[type_of(piece)];
        self.mg += PST.mgs[piece][square];
        self.eg += PST.egs[piece][square];
        self.items[square] = piece as u8;
        self.bitboards[piece] |= 1 << square;
        self.bitboards[piece | 7] |= 1 << square;
        self.zobrist ^= ZOBRIST.pieces[piece][square];
    }

    fn remove_piece(&mut self, square: usize) {
        let piece = self.items[square] as usize;
        self.phase += PIECE_PHASES[type_of(piece)];
        self.mg -= PST.mgs[piece][square];
        self.eg -= PST.egs[piece][square];
        self.zobrist ^= ZOBRIST.pieces[piece][square];
        self.bitboards[piece] &= !(1 << square);
        self.bitboards[piece | 7] &= !(1 << square);
        self.items[square] = NONE as u8;
    }

    fn move_piece_quiet(&mut self, from: usize, to: usize) {
        let piece = self.items[from] as usize;
        let pst = &*PST;
        self.mg += pst.mgs[piece][to] - pst.mgs[piece][from];
        self.eg += pst.egs[piece][to] - pst.egs[piece][from];
        self.zobrist ^= ZOBRIST.pieces[piece][from] ^ ZOBRIST.pieces[piece][to];
        let change = 1u64 << from | 1u64 << to;
        self.bitboards[piece] ^= change;
        self.bitboards[piece | 7] ^= change;
        self.items[to] = piece as u8;
        self.items[from] = NONE as u8;
    }

    fn move_piece(&mut self, from: usize, to: usize) {
        self.remove_piece(to);
        self.move_piece_quiet(from, to);
    }

    /// Completes a freshly built state: pins, checkers and the position key.
    fn finish(&mut self) {
        self.compute_pins();
        self.key = self.compute_key();
    }

    /// The position key: pieces, side, castling rights and a legal en passant only.
    #[inline(always)]
    pub fn hash(&self) -> u64 {
        self.key
    }

    #[inline(always)]
    pub fn transposition_key(&self) -> u64 {
        self.key
    }

    fn compute_key(&self) -> u64 {
        let mut key = self.zobrist;
        for side in 0..2 {
            if self.movements & castling_pieces_kingside_mask(side) == 0 {
                key ^= ZOBRIST.castling[side * 2];
            }
            if self.movements & castling_pieces_queenside_mask(side) == 0 {
                key ^= ZOBRIST.castling[side * 2 + 1];
            }
        }
        if self.en_passant != 0 && !self.has_legal_en_passant() {
            key ^= ZOBRIST.en_passant[lsb(self.en_passant) & 7];
        }
        key
    }

    fn has_legal_en_passant(&self) -> bool {
        let us = self.side;
        let them = 1 - us;
        let target = lsb(self.en_passant);
        let king = lsb(self.bitboard_of_side(us, KING));
        let captured = if us == WHITE { self.en_passant >> 8 } else { self.en_passant << 8 };
        let pawns = self.bitboard_of_side(them, PAWN) & !captured;
        let mut candidates = pawn_attacks(target, them) & self.bitboard_of_side(us, PAWN);
        while candidates != 0 {
            let from = lsb(candidates);
            let occupancy = (self.all_pieces() & !(1 << from) & !captured) | self.en_passant;
            let attacks = pawn_attacks(king, us) & pawns
                | knight_attacks(king) & self.bitboard_of_side(them, KNIGHT)
                | king_attacks(king) & self.bitboard_of_side(them, KING)
                | rook_attacks(king, occupancy) & self.orthogonal_sliders(them)
                | bishop_attacks(king, occupancy) & self.diagonal_sliders(them);
            if attacks == 0 {
                return true;
            }
            candidates &= candidates - 1;
        }
        false
    }

    /// Positions reached after a null move never count as draws.
    #[inline(always)]
    pub fn is_null_branch(&self) -> bool {
        self.null_branch
    }

    pub fn has_insufficient_material(&self) -> bool {
        if (self.bitboard_of_side(0, PAWN) | self.bitboard_of_side(1, PAWN) | self.orthogonal_sliders(0) | self.orthogonal_sliders(1)) != 0 {
            return false;
        }
        let knights = self.bitboard_of_side(0, KNIGHT) | self.bitboard_of_side(1, KNIGHT);
        let bishops = self.bitboard_of_side(0, BISHOP) | self.bitboard_of_side(1, BISHOP);
        if (knights | bishops).count_ones() <= 1 {
            return true;
        }
        let dark = 0xaa55_aa55_aa55_aa55u64;
        knights == 0 && ((bishops & dark) == 0 || (bishops & !dark) == 0)
    }

    #[inline(always)]
    pub fn bitboard_of(&self, piece: usize) -> u64 {
        self.bitboards[piece]
    }

    #[inline(always)]
    pub fn bitboards(&self) -> &[u64; 16] {
        &self.bitboards
    }

    #[inline(always)]
    pub fn bitboard_of_side(&self, side: usize, piece_type: usize) -> u64 {
        self.bitboards[make_piece(side, piece_type)]
    }

    #[inline(always)]
    pub fn diagonal_sliders(&self, side: usize) -> u64 {
        self.bitboard_of_side(side, BISHOP) | self.bitboard_of_side(side, QUEEN)
    }

    #[inline(always)]
    pub fn orthogonal_sliders(&self, side: usize) -> u64 {
        self.bitboard_of_side(side, ROOK) | self.bitboard_of_side(side, QUEEN)
    }

    #[inline(always)]
    pub fn all_pieces_of(&self, side: usize) -> u64 {
        self.bitboards[side << 3 | 7]
    }

    #[inline(always)]
    pub fn all_pieces(&self) -> u64 {
        self.bitboards[7] | self.bitboards[15]
    }

    /// Kings are not counted as attackers.
    pub fn attackers_from(&self, square: usize, occ: u64, side: usize) -> u64 {
        pawn_attacks(square, flip(side)) & self.bitboard_of_side(side, PAWN)
            | knight_attacks(square) & self.bitboard_of_side(side, KNIGHT)
            | bishop_attacks(square, occ) & self.diagonal_sliders(side)
            | rook_attacks(square, occ) & self.orthogonal_sliders(side)
    }

    pub fn attackers_from_including_kings(&self, square: usize, occ: u64, side: usize) -> u64 {
        self.attackers_from(square, occ, side) | king_attacks(square) & self.bitboard_of_side(side, KING)
    }

    pub fn do_move(&self, mv: Move) -> BoardState {
        let mut state = self.clone();
        let from = mv.from();
        let to = mv.to();

        state.full_move_normalized += 1;
        state.half_move_clock += 1;
        state.movements |= 1 << to | 1 << from;

        if state.piece_type_at(from) == PAWN {
            state.half_move_clock = 0;
        }

        state.clear_en_passant();

        match mv.flags() {
            Move::QUIET => state.move_piece_quiet(from, to),
            Move::DOUBLE_PUSH => {
                state.move_piece_quiet(from, to);
                state.en_passant = 1 << (from as i32 + direction(FORWARD, state.side)) as usize;
                state.zobrist ^= ZOBRIST.en_passant[lsb(state.en_passant) & 7];
            }
            Move::OO => state.castle(G1, H1, F1),
            Move::OOO => state.castle(C1, A1, D1),
            Move::EN_PASSANT => {
                state.move_piece_quiet(from, to);
                state.remove_piece((to as i32 + direction(BACK, state.side)) as usize);
            }
            Move::PR_KNIGHT | Move::PR_BISHOP | Move::PR_ROOK | Move::PR_QUEEN => {
                state.remove_piece(from);
                state.set_piece_at(make_piece(state.side, mv.piece_type()), to);
            }
            Move::PC_KNIGHT | Move::PC_BISHOP | Move::PC_ROOK | Move::PC_QUEEN => {
                state.remove_piece(from);
                state.remove_piece(to);
                state.set_piece_at(make_piece(state.side, mv.piece_type()), to);
            }
            Move::CAPTURE => {
                state.half_move_clock = 0;
                state.move_piece(from, to);
            }
            _ => {}
        }

        state.side = flip(state.side);
        state.zobrist ^= ZOBRIST.side;
        state.finish();
        state
    }

    /// Applies a legal move given in UCI notation.
    pub fn do_move_uci(&self, uci: &str) -> Option<BoardState> {
        self.find_legal_move(uci).map(|mv| self.do_move(mv))
    }

    pub fn find_legal_move(&self, uci: &str) -> Option<Move> {
        self.generate_legal_moves().iter().find(|m| m.uci() == uci)
    }

    pub fn do_null_move(&self) -> BoardState {
        let mut state = self.clone();
        state.null_branch = true;
        state.clear_en_passant();
        state.side = flip(state.side);
        state.zobrist ^= ZOBRIST.side;
        state.finish();
        state
    }

    /// Squares are given for white and mirrored to the 8th rank for black.
    fn castle(&mut self, king_to: usize, rook_from: usize, rook_to: usize) {
        let rank = if self.side == WHITE { 0 } else { A8 };
        self.move_piece_quiet(rank + E1, rank + king_to);
        self.move_piece_quiet(rank + rook_from, rank + rook_to);
    }

    #[inline(always)]
    pub fn side_to_play(&self) -> usize {
        self.side
    }

    /// Kings are not counted as attackers, they cannot give check.
    #[inline(always)]
    pub fn is_king_attacked(&self) -> bool {
        self.checkers != 0
    }

    pub fn has_non_pawn_material(&self, side: usize) -> bool {
        (self.all_pieces_of(side) & !self.bitboard_of_side(side, PAWN) & !self.bitboard_of_side(side, KING)) != 0
    }

    pub fn generate_legal_moves(&self) -> MoveList {
        let mut moves = MoveList::new();
        self.generate(0, false, u64::MAX, &mut moves);
        moves
    }

    /// Quiescence mode skips quiet moves of all pieces but the king.
    pub fn generate_legal_quiescence(&self) -> MoveList {
        let mut moves = MoveList::new();
        self.generate(3, false, u64::MAX, &mut moves);
        moves
    }

    pub fn generate_legal_moves_into(&self, moves: &mut MoveList) {
        moves.clear();
        self.generate(0, false, u64::MAX, moves);
    }

    pub fn generate_legal_tactical(&self, moves: &mut MoveList) {
        moves.clear();
        self.generate(1, false, u64::MAX, moves);
    }

    pub fn generate_legal_quiets(&self, moves: &mut MoveList) {
        moves.clear();
        self.generate(2, false, u64::MAX, moves);
    }

    pub fn has_legal_move(&self) -> bool {
        let mut moves = MoveList::new();
        self.generate(0, true, u64::MAX, &mut moves);
        !moves.is_empty()
    }

    pub fn is_legal_move(&self, mv: Move) -> bool {
        if mv.is_null() {
            return false;
        }
        let flags = mv.flags();
        if flags == Move::OO || flags == Move::OOO || flags == Move::EN_PASSANT {
            return self.is_generated_move(mv);
        }
        let us = self.side;
        let from = mv.from();
        let to = mv.to();
        let piece = self.piece_at(from);
        if piece == NONE || piece >> 3 != us {
            return false;
        }
        let target = self.piece_at(to);
        let capture = flags == Move::CAPTURE || (flags & 0xC000) == 0xC000;
        let quiet_promotion = (flags & 0xC000) == 0x8000;
        if !capture && !quiet_promotion && flags != Move::QUIET && flags != Move::DOUBLE_PUSH {
            return false;
        }
        let bad_target = if capture {
            target == NONE || target >> 3 == us || type_of(target) == KING
        } else {
            target != NONE
        };
        if bad_target {
            return false;
        }
        let piece_type = type_of(piece);
        let to_bit = 1u64 << to;
        let all = self.all_pieces();
        if piece_type == PAWN {
            if mv.is_promotion() != ((to_bit & PAWN_FINAL_RANKS) != 0) {
                return false;
            }
            let up = direction(FORWARD, us);
            if capture {
                if pawn_attacks(from, us) & to_bit == 0 {
                    return false;
                }
            } else if flags == Move::DOUBLE_PUSH {
                if to as i32 != from as i32 + 2 * up
                    || push(1 << from, us) & PAWN_DOUBLE_PUSH_LINES[us] == 0
                    || self.piece_at((from as i32 + up) as usize) != NONE
                {
                    return false;
                }
            } else if to as i32 != from as i32 + up {
                return false;
            }
        } else if mv.is_promotion() || flags == Move::DOUBLE_PUSH || attacks(piece_type, from, all) & to_bit == 0 {
            return false;
        }
        let them = 1 - us;
        if piece_type == KING {
            return self.attackers_from_including_kings(to, all ^ 1 << from, them) & !to_bit == 0;
        }
        if self.checkers != 0 {
            return self.is_generated_move(mv);
        }
        let king = lsb(self.bitboard_of_side(us, KING));
        self.pinned & 1 << from == 0 || line(king, from) & to_bit != 0
    }

    fn is_generated_move(&self, mv: Move) -> bool {
        let mut candidates = MoveList::new();
        self.generate(0, false, 1 << mv.from(), &mut candidates);
        candidates.contains(mv)
    }

    fn compute_pins(&mut self) {
        let us = self.side;
        let them = 1 - us;
        let king = lsb(self.bitboard_of_side(us, KING));
        if king >= 64 {
            // positions without a king (only from malformed FEN input)
            self.checkers = 0;
            self.pinned = 0;
            return;
        }
        let us_bb = self.all_pieces_of(us);
        let them_bb = self.all_pieces_of(them);
        let mut checkers = knight_attacks(king) & self.bitboard_of_side(them, KNIGHT) | pawn_attacks(king, us) & self.bitboard_of_side(them, PAWN);
        let mut pinned = 0;
        // seen through our pieces, a slider with no piece of ours in between checks, with one it pins
        let mut b = rook_attacks(king, them_bb) & self.orthogonal_sliders(them) | bishop_attacks(king, them_bb) & self.diagonal_sliders(them);
        while b != 0 {
            let sq = lsb(b);
            let ours = between(king, sq) & us_bb;
            if ours == 0 {
                checkers |= 1 << sq;
            } else if ours & (ours - 1) == 0 {
                pinned |= ours;
            }
            b &= b - 1;
        }
        self.checkers = checkers;
        self.pinned = pinned;
    }

    #[inline(always)]
    pub fn checkers(&self) -> u64 {
        self.checkers
    }

    fn king_danger(&self) -> u64 {
        self.attacked_squares(1 - self.side, self.all_pieces() ^ self.bitboard_of_side(self.side, KING))
    }

    /// Modes: 0 all moves, 1 tactical (captures and promotions), 2 quiets, 3 quiescence (captures and king quiets).
    /// The move order is relied upon by search tie-breaks and must stay stable.
    fn generate(&self, mode: u8, first_only: bool, from_mask: u64, moves: &mut MoveList) {
        let only_quiescence = mode == 1 || mode == 3;
        let captures = mode != 2;
        let us = self.side;
        let them = flip(us);

        let us_bb = self.all_pieces_of(us);
        let them_bb = self.all_pieces_of(them);
        let all = us_bb | them_bb;

        let our_king = lsb(self.bitboard_of_side(us, KING));
        let our_bishops_and_queens = self.diagonal_sliders(us);
        let their_bishops_and_queens = self.diagonal_sliders(them);
        let our_rooks_and_queens = self.orthogonal_sliders(us);
        let their_rooks_and_queens = self.orthogonal_sliders(them);

        // the squares attacked by them are only needed when our king can move or castle
        let king_allowed = our_king < 64 && from_mask & 1 << our_king != 0;
        let king_targets = if king_allowed { king_attacks(our_king) & !us_bb } else { 0 };
        let castling_candidate = king_allowed
            && !only_quiescence
            && ((self.movements & castling_pieces_kingside_mask(us) | all & castling_blockers_kingside_mask(us)) == 0
                || (self.movements & castling_pieces_queenside_mask(us) | all & castling_blockers_queenside_mask(us)) == 0);
        let under_attack = if king_targets != 0 || castling_candidate { self.king_danger() } else { 0 };
        if mode != 1 {
            moves.make_q(our_king, king_targets & !under_attack & !them_bb);
        }
        if captures {
            moves.make_c(our_king, king_targets & !under_attack & them_bb);
        }
        if first_only && !moves.is_empty() {
            return;
        }

        let checkers = self.checkers;
        let pinned = self.pinned;
        let not_pinned = !pinned;

        // capture_mask: where a non-king piece may capture; quiet_mask: where it may move quietly
        let capture_mask;
        let quiet_mask;
        match checkers.count_ones() {
            // not in check: every square is open, only pins restrict
            0 => {
                capture_mask = them_bb;
                quiet_mask = !all;

                if captures && self.en_passant != 0 {
                    let ep_square = lsb(self.en_passant);
                    let ep_victim = if us == WHITE { self.en_passant >> 8 } else { self.en_passant << 8 };
                    let ep_occupancy = all ^ self.en_passant ^ ep_victim;
                    // pinned pawns too: the capture may stay on the pin line
                    let mut b = pawn_attacks(ep_square, them) & self.bitboard_of_side(us, PAWN) & from_mask;
                    while b != 0 {
                        let from = lsb(b);
                        let occ = ep_occupancy ^ 1 << from;
                        if (rook_attacks(our_king, occ) & their_rooks_and_queens | bishop_attacks(our_king, occ) & their_bishops_and_queens) == 0 {
                            moves.add(Move::new(from, ep_square, Move::EN_PASSANT));
                        }
                        b &= b - 1;
                    }
                }

                if castling_candidate {
                    let king_home = if us == WHITE { E1 } else { E8 };
                    if (self.movements & castling_pieces_kingside_mask(us) | (all | under_attack) & castling_blockers_kingside_mask(us)) == 0 {
                        moves.add(Move::new(king_home, king_home + 2, Move::OO));
                    }
                    if (self.movements & castling_pieces_queenside_mask(us) | (all | under_attack & !ignore_ooo_danger(us)) & castling_blockers_queenside_mask(us)) == 0 {
                        moves.add(Move::new(king_home, king_home - 2, Move::OOO));
                    }
                }

                if first_only && !moves.is_empty() {
                    return;
                }

                // pinned pieces may only move along the pin line, pinned knights not at all
                let mut b = pinned & (our_bishops_and_queens | our_rooks_and_queens) & from_mask;
                while b != 0 {
                    let sq = lsb(b);
                    let targets = attacks(self.piece_type_at(sq), sq, all) & line(our_king, sq);
                    if !only_quiescence {
                        moves.make_q(sq, targets & quiet_mask);
                    }
                    if captures {
                        moves.make_c(sq, targets & capture_mask);
                    }
                    if first_only && !moves.is_empty() {
                        return;
                    }
                    b &= b - 1;
                }

                let mut b = pinned & self.bitboard_of_side(us, PAWN) & from_mask;
                while b != 0 {
                    let sq = lsb(b);
                    let pin_line = line(our_king, sq);
                    if (1 << sq) & PAWN_RANKS[us] != 0 {
                        if captures {
                            moves.make_pc(sq, pawn_attacks(sq, us) & capture_mask & pin_line);
                        }
                    } else {
                        if captures {
                            moves.make_c(sq, pawn_attacks(sq, us) & them_bb & pin_line);
                        }
                        if !only_quiescence {
                            let single_push = push(1 << sq, us) & !all & pin_line;
                            moves.make_q(sq, single_push);
                            moves.make_dp(sq, push(single_push & PAWN_DOUBLE_PUSH_LINES[us], us) & !all & pin_line);
                        }
                    }
                    if first_only && !moves.is_empty() {
                        return;
                    }
                    b &= b - 1;
                }
            }
            // single check: capture the checker, block it, or move the king
            1 => {
                let checker = lsb(checkers);
                let checker_type = self.piece_type_at(checker);
                if checker_type == PAWN || checker_type == KNIGHT {
                    // a contact check cannot be blocked, only the checker captured
                    let ep_victim = if us == WHITE { self.en_passant >> 8 } else { self.en_passant << 8 };
                    if captures && checkers == ep_victim {
                        let ep_square = lsb(self.en_passant);
                        let mut b = pawn_attacks(ep_square, them) & self.bitboard_of_side(us, PAWN) & not_pinned & from_mask;
                        while b != 0 {
                            moves.add(Move::new(lsb(b), ep_square, Move::EN_PASSANT));
                            b &= b - 1;
                        }
                    }
                    let mut b = if captures { self.attackers_from(checker, all, us) & not_pinned & from_mask } else { 0 };
                    while b != 0 {
                        let from = lsb(b);
                        if self.piece_type_at(from) == PAWN && checkers & PAWN_FINAL_RANKS != 0 {
                            add_promotions_at(moves, from, checker, Move::CAPTURE);
                        } else {
                            moves.add(Move::new(from, checker, Move::CAPTURE));
                        }
                        b &= b - 1;
                    }
                    return;
                }
                capture_mask = checkers;
                quiet_mask = between(our_king, checker);
            }
            // double check: only the king may move
            _ => return,
        }

        let mut b = self.bitboard_of_side(us, KNIGHT) & not_pinned & from_mask;
        while b != 0 {
            let sq = lsb(b);
            add_piece_moves(moves, sq, knight_attacks(sq), if captures { capture_mask } else { 0 }, quiet_mask, only_quiescence);
            if first_only && !moves.is_empty() {
                return;
            }
            b &= b - 1;
        }
        let mut b = our_bishops_and_queens & not_pinned & from_mask;
        while b != 0 {
            let sq = lsb(b);
            add_piece_moves(moves, sq, bishop_attacks(sq, all), if captures { capture_mask } else { 0 }, quiet_mask, only_quiescence);
            if first_only && !moves.is_empty() {
                return;
            }
            b &= b - 1;
        }
        let mut b = our_rooks_and_queens & not_pinned & from_mask;
        while b != 0 {
            let sq = lsb(b);
            add_piece_moves(moves, sq, rook_attacks(sq, all), if captures { capture_mask } else { 0 }, quiet_mask, only_quiescence);
            if first_only && !moves.is_empty() {
                return;
            }
            b &= b - 1;
        }

        let pawns = self.bitboard_of_side(us, PAWN) & not_pinned & from_mask;
        let promoting = pawns & PAWN_RANKS[us];
        let others = pawns & !PAWN_RANKS[us];
        let up = direction(FORWARD, us);
        let left = direction(FORWARD_LEFT, us);
        let right = direction(FORWARD_RIGHT, us);

        if !only_quiescence {
            let single_push = push(others, us) & !all;
            let double_push = push(single_push & PAWN_DOUBLE_PUSH_LINES[us], us) & quiet_mask;
            add_pawn_moves(moves, single_push & quiet_mask, up, Move::QUIET);
            add_pawn_moves(moves, double_push, 2 * up, Move::DOUBLE_PUSH);
        }
        if captures {
            add_pawn_moves(moves, left_pawn_attacks(others, us) & capture_mask, left, Move::CAPTURE);
            add_pawn_moves(moves, right_pawn_attacks(others, us) & capture_mask, right, Move::CAPTURE);
        }

        if promoting != 0 {
            if mode == 0 || mode == 1 {
                add_promotions(moves, push(promoting, us) & quiet_mask, up, Move::QUIET);
            }
            if captures {
                add_promotions(moves, left_pawn_attacks(promoting, us) & capture_mask, left, Move::CAPTURE);
                add_promotions(moves, right_pawn_attacks(promoting, us) & capture_mask, right, Move::CAPTURE);
            }
        }
    }

    /// Squares attacked by `side`, sliders seeing through the given occupancy.
    fn attacked_squares(&self, side: usize, occ: u64) -> u64 {
        let king = self.bitboard_of_side(side, KING);
        let mut attacked = pawn_attacks_bb(self.bitboard_of_side(side, PAWN), side) | if king != 0 { king_attacks(lsb(king)) } else { 0 };
        let mut b = self.bitboard_of_side(side, KNIGHT);
        while b != 0 {
            attacked |= knight_attacks(lsb(b));
            b &= b - 1;
        }
        let mut b = self.diagonal_sliders(side);
        while b != 0 {
            attacked |= bishop_attacks(lsb(b), occ);
            b &= b - 1;
        }
        let mut b = self.orthogonal_sliders(side);
        while b != 0 {
            attacked |= rook_attacks(lsb(b), occ);
            b &= b - 1;
        }
        attacked
    }

    fn clear_en_passant(&mut self) {
        if self.en_passant != 0 {
            self.zobrist ^= ZOBRIST.en_passant[lsb(self.en_passant) & 7];
            self.en_passant = 0;
        }
    }

    #[inline(always)]
    pub fn mg(&self) -> i32 {
        self.mg
    }

    #[inline(always)]
    pub fn eg(&self) -> i32 {
        self.eg
    }

    pub fn interpolated_score(&self) -> i32 {
        let phase = (self.phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE;
        (self.mg * (256 - phase) + self.eg * phase) / 256
    }

    pub fn is_in_check(&self) -> bool {
        self.is_king_attacked()
    }

    /// True for checkmate and stalemate alike.
    pub fn is_in_check_mate(&self) -> bool {
        !self.has_legal_move()
    }
}

#[inline(always)]
fn add_piece_moves(moves: &mut MoveList, from: usize, attacks: u64, capture_mask: u64, quiet_mask: u64, only_quiescence: bool) {
    moves.make_c(from, attacks & capture_mask);
    if !only_quiescence {
        moves.make_q(from, attacks & quiet_mask);
    }
}

#[inline(always)]
fn left_pawn_attacks(pawns: u64, side: usize) -> u64 {
    if side == WHITE {
        white_left_pawn_attacks(pawns)
    } else {
        black_right_pawn_attacks(pawns)
    }
}

#[inline(always)]
fn right_pawn_attacks(pawns: u64, side: usize) -> u64 {
    if side == WHITE {
        white_right_pawn_attacks(pawns)
    } else {
        black_left_pawn_attacks(pawns)
    }
}

#[inline(always)]
fn add_pawn_moves(moves: &mut MoveList, mut targets: u64, offset: i32, flags: u16) {
    while targets != 0 {
        let to = lsb(targets);
        moves.add(Move::new((to as i32 - offset) as usize, to, flags));
        targets &= targets - 1;
    }
}

fn add_promotions(moves: &mut MoveList, mut targets: u64, offset: i32, capture_flag: u16) {
    while targets != 0 {
        let to = lsb(targets);
        add_promotions_at(moves, (to as i32 - offset) as usize, to, capture_flag);
        targets &= targets - 1;
    }
}

// PC_* flags are the PR_* flags with the CAPTURE bit set
fn add_promotions_at(moves: &mut MoveList, from: usize, to: usize, capture_flag: u16) {
    moves.add(Move::new(from, to, Move::PR_QUEEN | capture_flag));
    moves.add(Move::new(from, to, Move::PR_ROOK | capture_flag));
    moves.add(Move::new(from, to, Move::PR_KNIGHT | capture_flag));
    moves.add(Move::new(from, to, Move::PR_BISHOP | capture_flag));
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(CHESSBOARD_LINE)?;
        for rank in (0..8).rev() {
            for file in 0..8 {
                write!(f, "| {} ", piece_notation(self.piece_at(rank * 8 + file)))?;
            }
            f.write_str("|\n")?;
            f.write_str(CHESSBOARD_LINE)?;
        }
        write!(f, "Fen: {}", self.to_fen())
    }
}

impl fmt::Debug for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BoardState({})", self.to_fen())
    }
}
