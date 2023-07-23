#![allow(unused_variables, dead_code)]

use std::fmt;

use crate::bitboard::{Bitboard, BITBOARD, BitIter};
use crate::board_position::BoardPosition;
use crate::fen::{Fen, FenExport};
use crate::pgn::Pgn;
use crate::piece::{BLACK_BISHOP, BLACK_KING, BLACK_KNIGHT, BLACK_PAWN, BLACK_QUEEN, BLACK_ROOK, make_piece, NONE, Piece, PIECES_COUNT, PieceType, to_piece_char, type_of, WHITE_BISHOP, WHITE_KING, WHITE_KNIGHT, WHITE_PAWN, WHITE_QUEEN, WHITE_ROOK};
use crate::piece::PieceType::{KING, KNIGHT, PAWN};
use crate::piece_square_table::{EGS, MGS};
use crate::r#move::{Move, MoveList};
use crate::side::Side;
use crate::side::Side::{BLACK, WHITE};
use crate::square::{BACK, DOUBLE_FORWARD, FORWARD, FORWARD_LEFT, FORWARD_RIGHT, Square};
use crate::transposition::Depth;
use crate::zobrist::ZOBRIST;

const TOTAL_PHASE: i32 = 24i32;
const PIECE_PHASES: [i32; 6] = [0, 1, 1, 2, 4, 0];

const CHESSBOARD_LINE: &'static str = "+---+---+---+---+---+---+---+---+\n";

pub const BOARD_STATE_HISTORY_CAPACITY: usize = 40;

//#[derive(Copy, Clone)]
#[derive(Clone, Debug)]
pub struct BoardState {
    pub(crate) ply: usize,
    history: [u16;BOARD_STATE_HISTORY_CAPACITY],
    piece_bb: [u64; PIECES_COUNT],
    pub items: [Piece; 64],
    pub side_to_play: Side,
    pub hash: u64,
    pub full_move_normalized: usize,
    pub half_move_clock: usize,
    phase: i32,
    mg: i16,
    eg: i16,
//    checkers: u64,
    pub(crate) movements: u64,
    pub en_passant: u64,

//    pub(crate) bitboard: &'a Bitboard,
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl FenExport for BoardState {
    fn to_fen(&self) -> String {
        Fen::compute_fen(&self)
    }
}

impl BoardState {
    pub fn new(
        items: &[Piece; 64],
        side_to_play: Side,
        movements: u64,
        en_passant: u64,
        half_move_clock: usize,
        full_move_count: usize,
        max_search_depth: usize,
    ) -> Self {
        // if items.len() != 64 { panic!("Expected array with 64 items. Received {} items.", items.len() as u64) }
        let mut board_state = BoardState {
            ply: 0,
            history: [0u16;BOARD_STATE_HISTORY_CAPACITY],
            piece_bb: [0; PIECES_COUNT],
            items: [0; 64], //(*items).clone(),
            side_to_play,
            hash: 0,
            full_move_normalized: 0,
            half_move_clock,
            phase: TOTAL_PHASE,
            mg: 0,
            eg: 0,
            movements,
            en_passant,
        };

        for i in 0..64 {
            let item = items[i];
            if item != NONE {
                board_state.set_piece_at(item, i);
            } else {
                board_state.items[i] = NONE;
            }
        }

        if side_to_play == BLACK {
            board_state.hash ^= ZOBRIST.side;
        }

        board_state.en_passant = en_passant;
        if board_state.en_passant != 0 {
            board_state.hash ^= ZOBRIST.en_passant[(en_passant.trailing_zeros() & 0b111) as usize];
        }

        board_state
    }

    pub fn to_string(&self) -> String  {
        let mut result = String::new();
        result.push_str(CHESSBOARD_LINE);

        for i in (0..=56).step_by(8).collect::<Vec<usize>>().into_iter().rev().collect::<Vec<usize>>() {
            for j in 0..8 {
                let piece: u8 = self.items[i + j];
                result.push_str("| ");
                result.push(to_piece_char(piece));
                result.push(' ');
            }

            result.push_str("|\n");
            result.push_str(&CHESSBOARD_LINE);
        }

        result
    }

    pub fn bitboard_string(bitboard: u64) -> String {
        let mut result = String::new();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let c = if (bitboard >> (rank * 8 + file) & 1) == 1 { 'X' } else { '.' };
                result.push(c);
                result.push(' ');
            }
            result.push('\n');
        }
        result
    }

    pub fn piece_at(&self, square: u8) -> Piece {
        self.items[square as usize]
    }

    pub fn piece_type_at(&self, square: u8) -> PieceType {
        return type_of(self.items[square as usize]);
    }

    pub fn set_piece_at(&mut self, piece: Piece, square: usize) {
        // //update incremental evaluation terms
        self.phase -= PIECE_PHASES[type_of(piece).index()];
        self.mg += MGS[piece as usize][square];
        self.eg += EGS[piece as usize][square];

        //set piece on board
        self.items[square] = piece;
        self.piece_bb[piece as usize] |= 1u64 << square;

        // //update hashes
        self.hash ^= ZOBRIST.pieces[piece as usize][square as usize];
    }

    fn remove_piece(&mut self, square: u8){
        let piece = self.items[square as usize];
        self.phase += PIECE_PHASES[type_of(piece).index()];
        self.mg -= MGS[piece as usize][square as usize];
        self.eg -= EGS[piece as usize][square as usize];

        //update hash tables
        self.hash ^= ZOBRIST.pieces[piece as usize][square as usize];

        //update board
        self.piece_bb[self.items[square as usize] as usize] &= !(1u64 << square);
        self.items[square as usize] = NONE;
    }

    fn move_piece_quiet(&mut self, from_sq: u8, to_sq: u8){
        //update incremental evaluation terms
        let piece = self.items[from_sq as usize];
        // if piece == NONE {
        //     panic!()
        // }
        self.mg += MGS[piece as usize][to_sq as usize] - MGS[piece as usize][from_sq as usize];
        self.eg += EGS[piece as usize][to_sq as usize] - EGS[piece as usize][from_sq as usize];

        //update hashes
        let zobrist = &ZOBRIST;
        self.hash ^= zobrist.pieces[piece as usize][from_sq as usize]
            ^ zobrist.pieces[piece as usize][to_sq as usize];

        //update board
        self.piece_bb[piece as usize] ^= 1u64 << from_sq | 1u64 << to_sq;
        self.items[to_sq as usize] = piece;
        self.items[from_sq as usize] = NONE;
    }

    pub fn move_piece(&mut self, from_sq: u8, to_sq: u8){
        self.remove_piece(to_sq);
        self.move_piece_quiet(from_sq, to_sq);
    }

    pub fn bitboard_of_piece(&self, piece: Piece) -> u64 {
        return self.piece_bb[piece as usize];
    }

    pub fn bitboard_of(&self, side: Side, piece_type: PieceType) -> u64 {
        self.piece_bb[make_piece(side, piece_type) as usize]
    }

    pub fn diagonal_sliders(&self, side: Side) -> u64 {
        match side {
            WHITE => self.piece_bb[WHITE_BISHOP as usize] | self.piece_bb[WHITE_QUEEN as usize],
            _ => self.piece_bb[BLACK_BISHOP as usize] | self.piece_bb[BLACK_QUEEN as usize]
        }
    }

    pub fn orthogonal_sliders(&self, side: Side) -> u64 {
        match side {
            WHITE => self.piece_bb[WHITE_ROOK as usize] | self.piece_bb[WHITE_QUEEN as usize],
            _ => self.piece_bb[BLACK_ROOK as usize] | self.piece_bb[BLACK_QUEEN as usize]
        }
    }

    pub fn all_pieces_for_side(&self, side: Side) -> u64 {
        return match side {
            WHITE => self.piece_bb[WHITE_PAWN as usize] | self.piece_bb[WHITE_KNIGHT as usize] |
                self.piece_bb[WHITE_BISHOP as usize] | self.piece_bb[WHITE_ROOK as usize] |
                self.piece_bb[WHITE_QUEEN as usize] | self.piece_bb[WHITE_KING as usize],
            _ =>
                self.piece_bb[BLACK_PAWN as usize] | self.piece_bb[BLACK_KNIGHT as usize] |
                self.piece_bb[BLACK_BISHOP as usize] | self.piece_bb[BLACK_ROOK as usize] |
                self.piece_bb[BLACK_QUEEN as usize] | self.piece_bb[BLACK_KING as usize]
        }
    }

    pub fn all_pieces(&self) -> u64 {
        self.all_pieces_for_side(WHITE) | self.all_pieces_for_side(BLACK)
    }

    pub fn attackers_from(&self, square: u8, occ: u64, side: Side) -> u64 {
        match side {
            WHITE => {
                (Bitboard::pawn_attacks_from_square(square as u8, BLACK) & self.piece_bb[WHITE_PAWN as usize]) |
                    (BITBOARD.get_knight_attacks(square as usize) & self.piece_bb[WHITE_KNIGHT as usize]) |
                    (BITBOARD.get_bishop_attacks(square as usize, occ) & (self.piece_bb[WHITE_BISHOP as usize] | self.piece_bb[WHITE_QUEEN as usize])) |
                    (BITBOARD.get_rook_attacks(square as usize, occ) & (self.piece_bb[WHITE_ROOK as usize] | self.piece_bb[WHITE_QUEEN as usize]))
            }
            _ => {
                (Bitboard::pawn_attacks_from_square(square as u8, WHITE) & self.piece_bb[BLACK_PAWN as usize]) |
                    (BITBOARD.get_knight_attacks(square as usize) & self.piece_bb[BLACK_KNIGHT as usize]) |
                    (BITBOARD.get_bishop_attacks(square as usize, occ) & (self.piece_bb[BLACK_BISHOP as usize] | self.piece_bb[BLACK_QUEEN as usize])) |
                    (BITBOARD.get_rook_attacks(square as usize, occ) & (self.piece_bb[BLACK_ROOK as usize] | self.piece_bb[BLACK_QUEEN as usize]))
            }
        }
    }

    pub fn do_null_move(&self) -> BoardState {
        BoardState::perform_null_move(self)
    }

    fn perform_null_move(old_board_state: &BoardState) -> BoardState {
        let mut state = old_board_state.clone();

        state.half_move_clock += 1;
        state.clear_en_passant();
        state.side_to_play = !state.side_to_play;
        state.hash ^= ZOBRIST.side;
        state
    }

    pub fn do_move(&self, moov: &Move) -> BoardState {
        self.do_move_param(moov, false)
    }

    pub fn do_move_string(&self, uci_move: &str) -> BoardState {
        let legal_moves = self.generate_legal_moves();
        let moov = legal_moves
            .moves
            .into_iter()
            .find(|m| m.to_string() == uci_move)
            .expect("Move not found");

        self.do_move(&moov)
    }

    pub fn do_move_no_history(&self, moov: &Move) -> BoardState {
        self.do_move_param(moov, true)
    }

    #[inline(always)]
    pub fn do_move_param(&self, moov: &Move, ignore_history: bool) -> BoardState {
        let mut state = self.clone();
        let zobrist = &ZOBRIST;

        state.full_move_normalized += 1;
        state.half_move_clock += 1;
        if !ignore_history {
            state.history[state.ply] = moov.bits;
            state.ply += 1;
        }
        state.movements |= 1u64 << moov.to() | 1u64 << moov.from();

        if type_of(state.items[moov.from() as usize]) == PAWN {
            state.half_move_clock = 0;
        }

        state.clear_en_passant();

        match moov.flags() {
            Move::QUIET => {
                state.move_piece_quiet(moov.from(), moov.to());
            }
            Move::DOUBLE_PUSH => {
                state.move_piece_quiet(moov.from(), moov.to());
                state.en_passant = 1u64 << (moov.from() as i8 + Square::direction(FORWARD, state.side_to_play));
                state.hash ^= zobrist.en_passant[(state.en_passant.trailing_zeros() & 0b111) as usize];
            }
            Move::OO => {
                if state.side_to_play == WHITE {
                    state.move_piece_quiet(Square::E1, Square::G1);
                    state.move_piece_quiet(Square::H1, Square::F1);
                }
                else {
                    state.move_piece_quiet(Square::E8, Square::G8);
                    state.move_piece_quiet(Square::H8, Square::F8);
                }
            }
            Move::OOO => {
                if state.side_to_play == WHITE {
                    state.move_piece_quiet(Square::E1, Square::C1);
                    state.move_piece_quiet(Square::A1, Square::D1);
                } else {
                    state.move_piece_quiet(Square::E8, Square::C8);
                    state.move_piece_quiet(Square::A8, Square::D8);
                }
            }
            Move::EN_PASSANT => {
                state.move_piece_quiet(moov.from(), moov.to());
                state.remove_piece((moov.to() as i8 + Square::direction(BACK, state.side_to_play)) as u8);
            }
            Move::PR_KNIGHT | Move::PR_BISHOP | Move::PR_ROOK | Move::PR_QUEEN=> {
                state.remove_piece(moov.from());
                state.set_piece_at(make_piece(state.side_to_play, moov.get_piece_type()), moov.to() as usize);
            }
            Move::PC_KNIGHT | Move::PC_BISHOP | Move::PC_ROOK | Move::PC_QUEEN => {
                state.remove_piece(moov.from());
                state.remove_piece(moov.to());
                state.set_piece_at(make_piece(state.side_to_play, moov.get_piece_type()), moov.to() as usize);
            }
            Move::CAPTURE => {
                state.half_move_clock = 0;
                state.move_piece(moov.from(), moov.to());
            }
            _ => {
                panic!()
            }
        }
        state.side_to_play = !state.side_to_play;
        state.hash ^= zobrist.side;

        state
    }

    pub fn is_king_attacked(&self) -> bool {
        let us = self.side_to_play;
        let them = !us;
        let our_king = self.bitboard_of(us, KING).trailing_zeros();

        if (Bitboard::pawn_attacks_from_square(our_king as u8, us) & self.bitboard_of(them, PAWN)) != 0 {
            return true;
        }

        if (BITBOARD.get_knight_attacks(our_king as usize) & self.bitboard_of(them, KNIGHT)) != 0 {
            return true;
        }

        let us_bb = self.all_pieces_for_side(us);
        let them_bb = self.all_pieces_for_side(them);
        let all = us_bb | them_bb;

        let their_diagonal_sliders = self.diagonal_sliders(them);
        let their_orthogonal_sliders = self.orthogonal_sliders(them);

        if (BITBOARD.get_rook_attacks(our_king as usize, all) & their_orthogonal_sliders) != 0 {
            return true;
        }

        return (BITBOARD.get_bishop_attacks(our_king as usize, all) & their_diagonal_sliders) != 0;
    }

    pub fn is_repetition_or_fifty(&self, position: &BoardPosition) -> bool {
        let last_move_bits = if self.ply > 0 { self.history[self.ply - 1] } else { *position.history.last().unwrap_or(&0) };
        let mut count = 0;
        let mut index: i32 = (self.ply - 1) as i32;
        while index >= 0 {
            if self.history[index as usize] == last_move_bits {
                count += 1;
            }
            index -= 1;
        }
        index = position.history.len() as i32 - 1;
        while index >= 0 {
            if position.history[index as usize] == last_move_bits {
                count += 1;
            }
            index -= 1;
        }
        return count > 2 || self.half_move_clock >= 100;
    }

    pub fn has_non_pawn_material(&self, side: Side) -> bool {
        for piece in make_piece(side, KNIGHT)..=make_piece(side, PieceType::QUEEN) {
            if self.bitboard_of_piece(piece) != 0 {
                return true;
            }
        }
        return false;
    }

    pub fn generate_legal_moves(&self) -> MoveList {
        self.generate_legal_moves_wo(false)
    }

    pub fn generate_legal_moves_wo(&self, only_quiescence: bool) -> MoveList {
        let mut moves = MoveList::new();
        let us = self.side_to_play;
        let them = !self.side_to_play;

        let us_bb = self.all_pieces_for_side(us);
        let them_bb = self.all_pieces_for_side(them);
        let all = us_bb | them_bb;

        let our_king_bb = self.bitboard_of(us, KING);
        let our_king = our_king_bb.trailing_zeros() as usize;
        let their_king = self.bitboard_of(them, KING).trailing_zeros() as usize;

        let our_bishops_and_queens = self.diagonal_sliders(us);
        let their_bishops_and_queens = self.diagonal_sliders(them);
        let our_rooks_and_queens = self.orthogonal_sliders(us);
        let their_rooks_and_queens = self.orthogonal_sliders(them);

        // Squares that the king can't move to
        let mut under_attack: u64 = 0;
        under_attack |= Bitboard::pawn_attacks(self.bitboard_of(them, PAWN), them) | BITBOARD.get_king_attacks(their_king);

        for b1 in BitIter(self.bitboard_of(them, KNIGHT)) {
            under_attack |= BITBOARD.get_knight_attacks(b1 as usize);
        }

        for b1 in BitIter(their_bishops_and_queens) {
            under_attack |= BITBOARD.get_bishop_attacks(b1 as usize, all ^ (1u64 << our_king as u8));
        }

        for b1 in BitIter(their_rooks_and_queens) {
            under_attack |= BITBOARD.get_rook_attacks(b1 as usize, all ^ (1u64 << our_king as u8));
        }

        let b1 = BITBOARD.get_king_attacks(our_king) & !(us_bb | under_attack);

        moves.make_quiets(our_king as u8, b1 & !them_bb);
        moves.make_captures(our_king as u8, b1 & them_bb);

        //capture_mask contains destinations where there is an enemy piece that is checking the king and must be captured
        //quiet_mask contains squares where pieces must be moved to block an incoming attack on the king
        let capture_mask: u64;
        let quiet_mask: u64;
        //let mut s: u8;

        // checker moves from opposite knights and pawns
        let mut checkers = (BITBOARD.get_knight_attacks(our_king) & self.bitboard_of(them, KNIGHT))
                | (Bitboard::pawn_attacks_from_square(our_king as u8, us) & self.bitboard_of(them, PAWN));

        // ray candidates to our king
        let mut candidates = (BITBOARD.get_rook_attacks(our_king, them_bb) & their_rooks_and_queens)
                | (BITBOARD.get_bishop_attacks(our_king, them_bb) & their_bishops_and_queens);

        let mut pinned: u64 = 0;

        for ray_candidate in BitIter(candidates) {
            // squares obstructed by our pieces
            let squares_between = BITBOARD.between(our_king as u8, ray_candidate as u8) & us_bb;

            // king is not guarded by any of our pieces
            if squares_between == 0 {
                checkers ^= 1u64 << ray_candidate;
            // when there's only one piece between king and a sliding piece, the piece is pinned
            } else if squares_between.count_ones() == 1 {
                pinned ^= squares_between;
            }
        }


        let not_pinned = !pinned;
        if checkers.count_ones() == 2 {
            // our king is in check mate
            return moves;
        } else if checkers.count_ones() == 1 {
            // our king is checked
            let checker_square: usize = checkers.trailing_zeros() as usize;
            let checker_piece_type = type_of(self.items[checker_square] as Piece);
            // for checking sliding pieces
            if checker_piece_type != PAWN && checker_piece_type != KNIGHT {
                // we have to capture them
                capture_mask = checkers;
                // ...or block 'em
                quiet_mask = BITBOARD.between(our_king as u8, checker_square as u8);
            } else {
                // for checking en-passants
                if checker_piece_type == PAWN && checkers == (if us == WHITE { self.en_passant >> 8 } else { self.en_passant << 8 }) {
                    // we have to consider taking the pawn en passant
                    let en_passant_square = self.en_passant.trailing_zeros();
                    // let bubbi1 = Bitboard::pawn_attacks_from_square(en_passant_square as u8, them);
                    // let bubbi2 = self.bitboard_of(us, PAWN);
                    for b1 in BitIter(Bitboard::pawn_attacks_from_square(en_passant_square as u8, them) & self.bitboard_of(us, PAWN) & not_pinned) {
                        moves.add(Move::new_from_flags(b1 as u8, en_passant_square as u8, Move::EN_PASSANT));
                    }
                }

                // capture the checking piece
                //for sq in BitIter(self.attackers_from(checker_square as u8, all, us) & not_pinned) {
                let attackers_from = self.attackers_from(checker_square as u8, all, us);
                let masked = attackers_from & not_pinned;
                for sq in BitIter(masked) {
                    if self.piece_type_at(sq as u8) == PAWN && (1u64 << sq & Bitboard::PAWN_FINAL_RANKS) != 0u64 {
                        moves.add(Move::new_from_flags(sq as u8, checker_square as u8, Move::PC_QUEEN));
                        moves.add(Move::new_from_flags(sq as u8, checker_square as u8, Move::PC_ROOK));
                        moves.add(Move::new_from_flags(sq as u8, checker_square as u8, Move::PC_KNIGHT));
                        moves.add(Move::new_from_flags(sq as u8, checker_square as u8, Move::PC_BISHOP));
                    }
                    else {
                        moves.add(Move::new_from_flags(sq as u8, checker_square as u8, Move::CAPTURE));
                    }
                }
                return moves;
            }
        // our king is not checked
        } else {
            capture_mask = them_bb;

            quiet_mask = !all;

            if self.en_passant != 0u64 {
                let en_passant_square = self.en_passant.trailing_zeros();
                let pawn_attacks = Bitboard::pawn_attacks_from_square(en_passant_square as u8, them) & self.bitboard_of(us, PAWN);
                for square in BitIter(pawn_attacks & not_pinned) {
                    // s hold square from which pawn attack to epsq can be done
                    // s = Long.numberOfTrailingZeros(b1);
                    // b1 = Bitboard.extractLsb(b1);

    //                        long attacks = Attacks.slidingAttacks(ourKing,
    //                                all ^ 1L << s) ^ Bitboard.shift(1L << this.epsq), Square.relative_dir(Square.SOUTH, us)),
    //                                Rank.getBb(Square.getRank(ourKing)));

                    // Bitboard.shift(1L << this.epsq), Square.relative_dir(Square.SOUTH, us)) holds pawn which can be en-passant taken
                    let qqq = them_bb ^ (if us == WHITE { self.en_passant >> 8 } else { self.en_passant << 8 });
                    candidates = (BITBOARD.get_rook_attacks(our_king, qqq | us_bb) & their_rooks_and_queens)
                            | (BITBOARD.get_bishop_attacks(our_king, qqq | us_bb) & their_bishops_and_queens);

                    if candidates == 0 {
                        moves.add(Move::new_from_flags(square as u8, en_passant_square as u8, Move::EN_PASSANT));
                    }
                }
            }

            if !only_quiescence {
                if 0 == ((self.movements & Bitboard::castling_pieces_kingside_mask(us)) | ((all | under_attack) & Bitboard::castling_blockers_kingside_mask(us))) {
                    moves.add(if us == WHITE { Move::new_from_flags(Square::E1, Square::G1, Move::OO) }
                              else { Move::new_from_flags(Square::E8, Square::G8, Move::OO) });
                }

                if 0 == ((self.movements & Bitboard::castling_pieces_queenside_mask(us)) |
                        ((all | (under_attack & !Bitboard::ignore_ooo_danger(us))) & Bitboard::castling_blockers_queenside_mask(us))) {
                    moves.add(if us == WHITE { Move::new_from_flags(Square::E1, Square::C1, Move::OOO) }
                              else { Move::new_from_flags(Square::E8, Square::C8, Move::OOO)});
                }
            }

            // all pinned sliding pieces can only eliminate the threat or move while staying pinned
            let b1 = !(not_pinned | self.bitboard_of(us, KNIGHT));
            for s in BitIter(b1) {
                let b2 = BITBOARD.attacks(self.piece_type_at(s as u8), s as u8, all) & BITBOARD.line(our_king as u8, s as u8);
                if !only_quiescence {
                    moves.make_quiets(s as u8, b2 & quiet_mask);
                }
                moves.make_captures(s as u8, b2 & capture_mask);
            }

            // for each pinned pawn
            let b1 = !not_pinned & self.bitboard_of(us, PAWN);
            for s in BitIter(b1) {
                if ((1u64 << s) & Bitboard::PAWN_FINAL_RANKS) != 0 {
                    let b2 = Bitboard::pawn_attacks_from_square(s as u8, us) & capture_mask & BITBOARD.line(our_king as u8, s as u8);
                    moves.make_promotion_captures(s as u8, b2);
                } else {
                    let b2 = Bitboard::pawn_attacks_from_square(s as u8, us) & them_bb & BITBOARD.line(s as u8, our_king as u8);
                    moves.make_captures(s as u8, b2);

                    if !only_quiescence {
                        //single pawn pushes
                        let b2 = Bitboard::push(1u64 << s, us) & !all & BITBOARD.line(our_king as u8, s as u8);
                        let b3 = Bitboard::push(b2 & Bitboard::PAWN_DOUBLE_PUSH_LINES[us as usize], us) & !all & BITBOARD.line(our_king as u8, s as u8);

                        moves.make_quiets(s as u8, b2);
                        moves.make_double_pushes(s as u8, b3);
                    }
                }
            }

            // pinned knights cannot move anyway, so let them stay
        }

        //non-pinned knight moves.
        let b1 = self.bitboard_of(us, KNIGHT) & not_pinned;
        for s in BitIter(b1) {
            let b2 = BITBOARD.get_knight_attacks(s as usize);
            moves.make_captures(s as u8, b2 & capture_mask);
            if !only_quiescence {
                moves.make_quiets(s as u8, b2 & quiet_mask);
            }
        }

        let b1 = our_bishops_and_queens & not_pinned;
        for s in BitIter(b1) {
            let b2 = BITBOARD.get_bishop_attacks(s as usize, all);
            moves.make_captures(s as u8, b2 & capture_mask);
            if !only_quiescence {
                moves.make_quiets(s as u8, b2 & quiet_mask);
            }
        }

        let b1 = our_rooks_and_queens & not_pinned;
        for s in BitIter(b1) {
            let b2 = BITBOARD.get_rook_attacks(s as usize, all);
            moves.make_captures(s as u8, b2 & capture_mask);
            if !only_quiescence {
                moves.make_quiets(s as u8, b2 & quiet_mask);
            }
        }

        // let pawni = self.bitboard_of(us, PAWN);
        // let filter = !Bitboard::PAWN_RANKS[us as usize];
        let b1 = self.bitboard_of(us, PAWN) & not_pinned & !Bitboard::PAWN_RANKS[us as usize];

        if !only_quiescence {
            // single pawn pushes
            let mut b2 = match us { WHITE => b1 << 8, _ => b1 >> 8} & !all;

            //double pawn pushes
            let double_pawn_pushes = Bitboard::push(b2 & Bitboard::PAWN_DOUBLE_PUSH_LINES[us as usize], us) & quiet_mask;

            b2 &= quiet_mask;

            for s in BitIter(b2) {
                let direct = Square::direction(FORWARD, us);
                moves.add(Move::new_from_flags((s as i8 - direct) as u8, s as u8, Move::QUIET));
            }

            for s in BitIter(double_pawn_pushes) {
                moves.add(Move::new_from_flags((s as i8 - Square::direction(DOUBLE_FORWARD, us)) as u8, s as u8, Move::DOUBLE_PUSH));
            }
        }

        let b2 = (match us { WHITE => Bitboard::white_left_pawn_attacks(b1), _ => Bitboard::black_right_pawn_attacks(b1) }) & capture_mask;
        let diagonal_attacks_2 = (match us { WHITE => Bitboard::white_right_pawn_attacks(b1), _ => Bitboard::black_left_pawn_attacks(b1) }) & capture_mask;

        for square in BitIter(b2) {
            moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_LEFT, us)) as u8, square as u8, Move::CAPTURE));
        }

        for square in BitIter(diagonal_attacks_2) {
            moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, square as u8, Move::CAPTURE));
        }

        let b1 = self.bitboard_of(us, PAWN) & not_pinned & Bitboard::PAWN_RANKS[us as usize];
        if b1 != 0 {
            if !only_quiescence {
                let b2 = match us { WHITE => b1 << 8, _ => b1 >> 8 } & quiet_mask;
                for square in BitIter(b2) {
                    moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD, us)) as u8, square as u8, Move::PR_QUEEN));
                    moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD, us)) as u8, square as u8, Move::PR_KNIGHT));
                    moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD, us)) as u8, square as u8, Move::PR_ROOK));
                    moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD, us)) as u8, square as u8, Move::PR_BISHOP));
                }
            }

            let diagonal_attacks_1 = (match us { WHITE => Bitboard::white_left_pawn_attacks(b1), _ => Bitboard::black_right_pawn_attacks(b1) }) & capture_mask;
            let diagonal_attacks_2 = (match us { WHITE => Bitboard::white_right_pawn_attacks(b1), _ => Bitboard::black_left_pawn_attacks(b1) }) & capture_mask;

            for square in BitIter(diagonal_attacks_1) {
                moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_LEFT, us)) as u8, square as u8, Move::PC_QUEEN));
                moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_LEFT, us)) as u8, square as u8, Move::PC_KNIGHT));
                moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_LEFT, us)) as u8, square as u8, Move::PC_ROOK));
                moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_LEFT, us)) as u8, square as u8, Move::PC_BISHOP));
            }

            for square in BitIter(diagonal_attacks_2) {
                moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, square as u8, Move::PC_QUEEN));
                moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, square as u8, Move::PC_KNIGHT));
                moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, square as u8, Move::PC_ROOK));
                moves.add(Move::new_from_flags((square as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, square as u8, Move::PC_BISHOP));
            }
        }

        moves
    }

    fn clear_en_passant(&mut self) {
        let previous_state = self.en_passant;

        if previous_state != 0u64 {
            self.en_passant = 0;
            self.hash ^= ZOBRIST.en_passant[(previous_state.trailing_zeros() & 0b111) as usize];
            //this.hash ^= Zobrist.EN_PASSANT[(int) (previous_state & 0b111)];
        }
    }

    pub fn for_search_depth(&self, search_depth: Depth) -> BoardState {
        let mut result = self.clone();
        //result.history = new long[search_depth];
        result.ply = 0;
        return result;
    }

    pub fn interpolated_score(&self) -> i16 {
        let phase= ((self.phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE) as i16;
//        println!("mg: {}, eg: {}, phase: {}", self.mg, self.eg, phase);
        return (((self.mg as i32) * (256 - phase) as i32 + (self.eg as i32) * phase as i32) / 256) as i16;
        // self.mg
    }

    ///
    ///
    /// # Arguments
    ///
    /// * `parts`:
    ///
    /// returns: Vec<Move, Global>
    ///
    /// # Examples
    ///
    /// ```
    ///
    /// ```
    pub fn parse_moves(&self, parts: &Vec<&str>) -> Vec<Move> {
        let mut state = self.clone();
        let mut moves: Vec<Move> = Vec::new();

        for i in 0..parts.len() {
            let moov = Move::from_uci_string(parts[i], &state);
            state = state.do_move_no_history(&moov);
            moves.push(moov);
        }

        moves
    }

    pub fn parse_pgn_moves(&self, pgn_moves: &str) -> Vec<String> {
        let mut state = self.clone();
        let mut move_vec= Vec::new();
        let moves = pgn_moves.split_whitespace();
        for moov in moves {
            let uci = Pgn::one_san_to_uci(moov, &state);
            let parsed_move = Move::from_uci_string(&uci, &state);
            state = state.do_move_no_history(&parsed_move);
            move_vec.push(uci);
        }
        move_vec
    }

    pub fn is_in_check(&self) -> bool {
        self.checkers() != 0
    }

    pub fn is_in_checkmate(&self) -> bool {
        self.generate_legal_moves().is_empty()
    }

    pub fn is_capture(&self, move_str: &str) -> bool {
        let parsed_move = Move::from_uci_string(move_str, self);
        return self.piece_at(parsed_move.to()) != NONE;
    }

    pub fn checkers(&self) -> u64 {
        // TODO simplify? inline???

        let us = self.side_to_play;
        let them = !self.side_to_play;

        let us_bb = self.all_pieces_for_side(us);
        let them_bb = self.all_pieces_for_side(them);
        let all = us_bb | them_bb;

        let our_king_bb = self.bitboard_of(us, KING);
        let our_king = our_king_bb.trailing_zeros() as usize;
        let their_king = self.bitboard_of(them, KING).trailing_zeros() as usize;

        let our_bishops_and_queens = self.diagonal_sliders(us);
        let their_bishops_and_queens = self.diagonal_sliders(them);
        let our_rooks_and_queens = self.orthogonal_sliders(us);
        let their_rooks_and_queens = self.orthogonal_sliders(them);

        // Squares that the king can't move to
        let mut under_attack: u64 = 0;
        under_attack |= Bitboard::pawn_attacks(self.bitboard_of(them, PAWN), them) | BITBOARD.get_king_attacks(their_king);

        for b1 in BitIter(self.bitboard_of(them, KNIGHT)) {
            under_attack |= BITBOARD.get_knight_attacks(b1 as usize);
        }

        for b1 in BitIter(their_bishops_and_queens) {
            under_attack |= BITBOARD.get_bishop_attacks(b1 as usize, all ^ (1u64 << our_king as u8));
        }

        for b1 in BitIter(their_rooks_and_queens) {
            under_attack |= BITBOARD.get_rook_attacks(b1 as usize, all ^ (1u64 << our_king as u8));
        }

        let b1 = BITBOARD.get_king_attacks(our_king) & !(us_bb | under_attack);

        // moves.make_quiets(our_king as u8, b1 & !them_bb);
        // moves.make_captures(our_king as u8, b1 & them_bb);

        //capture_mask contains destinations where there is an enemy piece that is checking the king and must be captured
        //quiet_mask contains squares where pieces must be moved to block an incoming attack on the king
        //let capture_mask: u64;
        //let quiet_mask: u64;
        //let mut s: u8;

        // checker moves from opposite knights and pawns
        let mut checkers = (BITBOARD.get_knight_attacks(our_king) & self.bitboard_of(them, KNIGHT))
            | (Bitboard::pawn_attacks_from_square(our_king as u8, us) & self.bitboard_of(them, PAWN));

        // ray candidates to our king
        let candidates = (BITBOARD.get_rook_attacks(our_king, them_bb) & their_rooks_and_queens)
            | (BITBOARD.get_bishop_attacks(our_king, them_bb) & their_bishops_and_queens);
        //
        // let mut pinned: u64 = 0;

        for ray_candidate in BitIter(candidates) {
            // squares obstructed by our pieces
            let squares_between = BITBOARD.between(our_king as u8, ray_candidate as u8) & us_bb;

            // king is not guarded by any of our pieces
            if squares_between == 0 {
                checkers ^= 1u64 << ray_candidate;
                // when there's only one piece between king and a sliding piece, the piece is pinned
            }
        }

        checkers
    }

    /**
     * @param side attacked side
     * @return attacked pieces
     */
    pub fn attacked_pieces(&self, side: Side) -> u64 {
        let working_state = if self.side_to_play == side {
            self.do_null_move()
        } else {
            self.clone()
        };

        let quiescence = working_state.generate_legal_moves_wo(false);
        let attacking_moves: Vec<Move> = quiescence
            .moves
            .iter()
            .filter(|&m| working_state.piece_at(m.to()) != NONE)
            .cloned()
            .collect();

        let mut result: u64 = 0;
        for m in attacking_moves {
            result |= 1 << m.to();
        }
        result
    }
}


#[cfg(test)]
mod tests {
    use crate::fen::{Fen, START_POS};
    use crate::piece::{BLACK_ROOK, WHITE_ROOK};
    use crate::square::Square;
    use crate::transposition::TranspositionTable;

    #[test]
    fn from_fen_startpos() {
        let state = Fen::from_fen_default(START_POS);
        let moves = state.generate_legal_moves();
        println!("{}", moves);
        // assert_eq!(state.to_string(), );
    }

    #[test]
    fn from_failing_sts() {
        let state = Fen::from_fen_default("2r5/p3k1p1/1p5p/4Pp2/1PPnK3/PB1R2P1/7P/8 w - f6 0 4");
        let moves = state.generate_legal_moves();
        println!("{}", moves);
    }

    #[test]
    fn failing_cute_chess() {
        let state = Fen::from_fen_default("rnbqkbnr/pppp2pp/5p2/8/5P2/8/PPP1PPPP/RN1QKBNR w KQkq - 0 4");
        let moves = state.generate_legal_moves();
        let tt = TranspositionTable::new(1);
        for moov in moves.over_sorted(&state, &tt) {
            println!("{}", moov.uci());
        }
        println!("{}", moves);
    }

    #[test]
    fn mg_value_test() {
        let mut state = Fen::from_fen_default("8/8/8/8/8/8/8/8 w KQkq - 0 1");
        assert_eq!(state.mg, 0);
        state.set_piece_at(WHITE_ROOK, Square::get_square_from_name("d5") as usize);
        assert_eq!(state.mg, 482);
        state.set_piece_at(BLACK_ROOK, Square::get_square_from_name("d2") as usize);
        assert_eq!(state.mg, -20);
    }
}
