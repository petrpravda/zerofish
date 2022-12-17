#![allow(unused_variables, dead_code)]

use std::fmt;

use crate::bitboard::{Bitboard, BITBOARD, BitIter};
use crate::board_position::BoardPosition;
use crate::fen::{Fen, FenExport};
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

const BOARD_STATE_HISTORY_CAPACITY: usize = 40;

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
        //bitboard: &'a Bitboard,
    ) -> Self {
        if items.len() != 64 { panic!("Expected array with 64 items. Received {} items.", items.len() as u64) }
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
//            checkers: 0,
            movements,
            en_passant,
//            bitboard
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


    //
    //     public BoardState(int[] items, int sideToPlay, long movements, long enPassantMask, int halfMoveClock, int fullMoveCount, int maxSearchDepth) {
    //         for (int i = 0; i < 64; i++) {
    //             int item = items[i];
    //             if (item != Piece.NONE) {
    //                 set_piece_at(item, i);
    //             } else {
    //                 this.items[i] = Piece.NONE;
    //             }
    //         }
    //
    //         this.sideToPlay = sideToPlay;
    //
    //         if (sideToPlay == Side.BLACK)
    //             this.hash ^= Zobrist.SIDE;
    //
    //         this.enPassant = enPassantMask;
    //         if (this.enPassant != 0) {
    //             this.hash ^= Zobrist.EN_PASSANT[(int) (this.enPassant & 0b111)];
    //         }
    //
    //         this.movements = movements;
    //
    //         this.halfMoveClock = halfMoveClock;
    //         this.full_move_normalized = (fullMoveCount - 1) * 2 + (sideToPlay == Side.WHITE ? 0 : 1);
    //         this.history = new long[maxSearchDepth];
    //         this.ply = 0;
    //     }
    //
    //     public static BoardState fromFen(String fen) {
    //         return Fen.fromFen(fen, null);
    //     }
    //
    //     public static BoardState fromFen(String fen, int maxSearchDepth) {
    //         return Fen.fromFen(fen, maxSearchDepth);
    //     }
    //
    //     @Override
    //     protected BoardState clone() {
    //         try {
    //             BoardState result = (BoardState) super.clone();
    //             result.piece_bb = this.piece_bb.clone();
    //             result.items = this.items.clone();
    //             result.history = this.history.clone();
    //             return result;
    //         } catch (CloneNotSupportedException e) {
    //             throw new IllegalStateException(e);
    //         }
    //     }
    //
    //     public int pieceAt(int square){
    //         return items[square];
    //     }

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
            // hash ^= Zobrist.ZOBRIST_TABLE[piece][square];
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

    //     public long hash(){
    //         return hash;
    //     }
    //
        pub fn bitboard_of_piece(&self, piece: Piece) -> u64 {
            return self.piece_bb[piece as usize];
        }

        pub fn bitboard_of(&self, side: Side, piece_type: PieceType) -> u64 {
            self.piece_bb[make_piece(side, piece_type) as usize]
        }

    //     public long checkers(){
    //         return checkers;
    //     }
    //
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

    //     public long attackersFromIncludingKings(int square, long occ, int side){
    //         return side == Side.WHITE ? (pawn_attacks(square, Side.BLACK) & piece_bb[Piece.WHITE_PAWN]) |
    //                 (get_king_attacks(square) & piece_bb[Piece.WHITE_KING]) |
    //                 (get_knight_attacks(square) & piece_bb[Piece.WHITE_KNIGHT]) |
    //                 (getBishopAttacks(square, occ) & (piece_bb[Piece.WHITE_BISHOP] | piece_bb[Piece.WHITE_QUEEN])) |
    //                 (getRookAttacks(square, occ) & (piece_bb[Piece.WHITE_ROOK] | piece_bb[Piece.WHITE_QUEEN])) :
    //
    //                 (pawn_attacks(square, Side.WHITE) & piece_bb[Piece.BLACK_PAWN]) |
    //                 (get_king_attacks(square) & piece_bb[Piece.BLACK_KING]) |
    //                 (get_knight_attacks(square) & piece_bb[Piece.BLACK_KNIGHT]) |
    //                 (getBishopAttacks(square, occ) & (piece_bb[Piece.BLACK_BISHOP] | piece_bb[Piece.BLACK_QUEEN])) |
    //                 (getRookAttacks(square, occ) & (piece_bb[Piece.BLACK_ROOK] | piece_bb[Piece.BLACK_QUEEN]));
    //     }
    //
    //     public BoardState do_move(String uciMove) {
    //         return performMove(this.generate_legal_moves().stream().filter(m->m.toString().equals(uciMove)).findFirst().orElseThrow(), this);
    //     }

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
        // if moov.flags() == Move::NULL {
        //     panic!()
        // }


        let mut state = self.clone();
        // state.do_move_inner(moov);
        // return state;
        let zobrist = &ZOBRIST;

        state.full_move_normalized += 1;
        state.half_move_clock += 1;
        state.history[state.ply] = moov.bits;
        state.ply += 1;
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

    // pub fn do_move_inner(&mut self, moov: &Move) {
    //     let zobrist = &ZOBRIST;
    //
    //     self.full_move_normalized += 1;
    //     self.half_move_clock += 1;
    //     self.history[self.ply] = moov.bits;
    //     self.ply += 1;
    //     self.movements |= 1u64 << moov.to() | 1u64 << moov.from();
    //
    //     if type_of(self.items[moov.from() as usize]) == PAWN {
    //         self.half_move_clock = 0;
    //     }
    //
    //     self.clear_en_passant();
    //
    //     match moov.flags() {
    //         Move::QUIET => {
    //             self.move_piece_quiet(moov.from(), moov.to());
    //         }
    //         Move::DOUBLE_PUSH => {
    //             self.move_piece_quiet(moov.from(), moov.to());
    //             self.en_passant = 1u64 << (moov.from() as i8 + Square::direction(FORWARD, self.side_to_play));
    //             self.hash ^= zobrist.en_passant[(self.en_passant.trailing_zeros() & 0b111) as usize];
    //         }
    //         Move::OO => {
    //             if self.side_to_play == WHITE {
    //                 self.move_piece_quiet(Square::E1, Square::G1);
    //                 self.move_piece_quiet(Square::H1, Square::F1);
    //             }
    //             else {
    //                 self.move_piece_quiet(Square::E8, Square::G8);
    //                 self.move_piece_quiet(Square::H8, Square::F8);
    //             }
    //         }
    //         Move::OOO => {
    //             if self.side_to_play == WHITE {
    //                 self.move_piece_quiet(Square::E1, Square::C1);
    //                 self.move_piece_quiet(Square::A1, Square::D1);
    //             } else {
    //                 self.move_piece_quiet(Square::E8, Square::C8);
    //                 self.move_piece_quiet(Square::A8, Square::D8);
    //             }
    //         }
    //         Move::EN_PASSANT => {
    //             self.move_piece_quiet(moov.from(), moov.to());
    //             self.remove_piece((moov.to() as i8 + Square::direction(BACK, self.side_to_play)) as u8);
    //         }
    //         Move::PR_KNIGHT | Move::PR_BISHOP | Move::PR_ROOK | Move::PR_QUEEN=> {
    //             self.remove_piece(moov.from());
    //             self.set_piece_at(make_piece(self.side_to_play, moov.get_piece_type()), moov.to() as usize);
    //         }
    //         Move::PC_KNIGHT | Move::PC_BISHOP | Move::PC_ROOK | Move::PC_QUEEN => {
    //             self.remove_piece(moov.from());
    //             self.remove_piece(moov.to());
    //             self.set_piece_at(make_piece(self.side_to_play, moov.get_piece_type()), moov.to() as usize);
    //         }
    //         Move::CAPTURE => {
    //             self.half_move_clock = 0;
    //             self.move_piece(moov.from(), moov.to());
    //         }
    //         _ => {
    //             panic!()
    //         }
    //     }
    //     self.side_to_play = !self.side_to_play;
    //     self.hash ^= zobrist.side;
    // }

    //     public int getSideToPlay(){
    //         return sideToPlay;
    //     }
    //

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

    //
    //     /* not    side of the attacker */
    //     /**
    //      * @param side attacked side
    //      * @return attacked pieces
    //      */
    //     public long attackedPieces(int side) {
    //         BoardState workingState = this.getSideToPlay() == side ? this.do_null_move() : this;
    //         MoveList quiescence = workingState.generateLegalQuiescence();
    //         //BoardState finalWorkingState = workingState;
    //         List<Move> attackingMoves = quiescence.stream()
    //                 .filter(m -> workingState.pieceAt(m.to()) != Piece.NONE)
    //                 .toList();
    //         long result = 0L;
    //         for (Move move : attackingMoves) {
    //             result |= 1L << move.to();
    //         }
    //         return result;
    //
    //     }
    //
    //     /**
    //      * @param side attacked side
    //      * @return
    //      */
    //     public long attackedPiecesUndefended(int side) {
    //         int sideThem = Side.flip(side);
    //         let usBb = all_pieces(side);
    //         let themBb = all_pieces(sideThem);
    //         let all = usBb | themBb;
    //
    //         long attackedPieces = this.attackedPieces(side);
    //         long attackedUndefendedPieces = 0L;
    //         long work = attackedPieces;
    //         while (work != 0){
    //             int square = Long.numberOfTrailingZeros(work);
    //             long attackingPieces = attackersFromIncludingKings(square, all, sideThem);
    //             while (attackingPieces != 0) {
    //                 int attackingSquare = Long.numberOfTrailingZeros(attackingPieces);
    //                 long allWithoutAttacker = all & ~(1L << attackingSquare);
    //                 long defendingPieces = attackersFromIncludingKings(square, allWithoutAttacker, side);
    //                 if (defendingPieces == 0L) {
    //                     attackedUndefendedPieces |= 1L << square;
    //                 }
    //
    //                 attackingPieces = Bitboard.extractLsb(attackingPieces);
    //             }
    //             work = Bitboard.extractLsb(work);
    //         }
    //
    //         return attackedUndefendedPieces;
    //     }
    //
    //     public int smallestAttackerWithKing(int square, int side) {
    //         return smallestAttacker(square, side, true);
    //     }
    //
    //     public int smallestAttacker(int square, int side, boolean withAttackingKing){
    //         final int us = Side.flip(side);
    //         final int them = side;
    //
    //         long pawns = pawn_attacks(square, us) & bitboard_of(them, PieceType.PAWN);
    //         if (pawns != 0)
    //             return Long.numberOfTrailingZeros(pawns);
    //
    //         long knights = get_knight_attacks(square) & bitboard_of(them, PieceType.KNIGHT);
    //         if (knights != 0)
    //             return Long.numberOfTrailingZeros(knights);
    //
    //         let usBb = all_pieces(us);
    //         let themBb = all_pieces(them);
    //         let all = usBb | themBb;
    //
    //         let bishopAttacks = getBishopAttacks(square, all);
    //         long bishops = bishopAttacks & bitboard_of(them, PieceType.BISHOP);
    //
    //         if (bishops != 0)
    //             return Long.numberOfTrailingZeros(bishops);
    //
    //         let rookAttacks = getRookAttacks(square, all);
    //         long rooks = rookAttacks & bitboard_of(them, PieceType.ROOK);
    //         if (rooks != 0)
    //             return Long.numberOfTrailingZeros(rooks);
    //
    //         long queens = (bishopAttacks | rookAttacks) & bitboard_of(them, PieceType.QUEEN);
    //         if (queens != 0)
    //             return Long.numberOfTrailingZeros(queens);
    //
    //         if (withAttackingKing) {
    //             long kings = get_king_attacks(square) & bitboard_of(them, PieceType.KING);
    //             if (kings != 0) {
    //                 return Long.numberOfTrailingZeros(kings);
    //             }
    //         }
    //
    //         return NO_SQUARE;
    //     }
    //
    // //    public boolean isInsufficientMaterial(int color){
    // //        if ((bitboard_of(color, PieceType.PAWN) | bitboard_of(color, PieceType.ROOK) | bitboard_of(color, PieceType.QUEEN)) != 0)
    // //            return false;
    // //
    // //        long ourPieces = all_pieces(color);
    // //        long theirPieces = all_pieces(Side.flip(color));
    // //        if (bitboard_of(color, PieceType.KNIGHT) != 0)
    // //            return Long.bitCount(ourPieces) <= 2 && (theirPieces & ~bitboard_of(Side.flip(color), PieceType.KING) & ~bitboard_of(Side.flip(color), PieceType.QUEEN)) == 0;
    // //
    // //        long ourBishops = bitboard_of(color, PieceType.BISHOP);
    // //        if (ourBishops != 0){
    // //            boolean sameColor = (ourBishops & DARK_SQUARES) == 0 || (ourBishops & LIGHT_SQUARES) == 0;
    // //            return sameColor && (bitboard_of(color, PieceType.PAWN) | bitboard_of(Side.flip(color), PieceType.PAWN)) == 0
    // //                    || (bitboard_of(color, PieceType.KNIGHT) | bitboard_of(Side.flip(color), PieceType.KNIGHT)) == 0;
    // //        }
    // //        return true;
    // //    }

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

    //     public MoveList generate_legal_moves(){
    //         return this.generate_legal_moves(false);
    //     }
    //
    //     public MoveList generateLegalQuiescence(){
    //         return generate_legal_moves(true);
    //     }

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
                let b2 = Bitboard::pawn_attacks_from_square(en_passant_square as u8, them) & self.bitboard_of(us, PAWN);
                // b2 holds pawns that can do an ep capture
                for s in BitIter(b2 & not_pinned) {
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
                        moves.add(Move::new_from_flags(s as u8, en_passant_square as u8, Move::EN_PASSANT));
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

        for s in BitIter(b2) {
            moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_LEFT, us)) as u8, s as u8, Move::CAPTURE));
        }

        for s in BitIter(diagonal_attacks_2) {
            moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, s as u8, Move::CAPTURE));
        }

        let b1 = self.bitboard_of(us, PAWN) & not_pinned & Bitboard::PAWN_RANKS[us as usize];
        if b1 != 0 {
            if !only_quiescence {
                let b2 = match us { WHITE => b1 << 8, _ => b1 >> 8 } & quiet_mask;
                for s in BitIter(b2) {
                    moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD, us)) as u8, s as u8, Move::PR_QUEEN));
                    moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD, us)) as u8, s as u8, Move::PR_KNIGHT));
                    moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD, us)) as u8, s as u8, Move::PR_ROOK));
                    moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD, us)) as u8, s as u8, Move::PR_BISHOP));
                }
            }

            let diagonal_attacks_1 = (match us { WHITE => Bitboard::white_left_pawn_attacks(b1), _ => Bitboard::black_right_pawn_attacks(b1) }) & capture_mask;
            let diagonal_attacks_2 = (match us { WHITE => Bitboard::white_right_pawn_attacks(b1), _ => Bitboard::black_left_pawn_attacks(b1) }) & capture_mask;

            for s in BitIter(diagonal_attacks_1) {
                moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_LEFT, us)) as u8, s as u8, Move::PC_QUEEN));
                moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_LEFT, us)) as u8, s as u8, Move::PC_KNIGHT));
                moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_LEFT, us)) as u8, s as u8, Move::PC_ROOK));
                moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_LEFT, us)) as u8, s as u8, Move::PC_BISHOP));
            }

            for s in BitIter(diagonal_attacks_2) {
                moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, s as u8, Move::PC_QUEEN));
                moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, s as u8, Move::PC_KNIGHT));
                moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, s as u8, Move::PC_ROOK));
                moves.add(Move::new_from_flags((s as i8 - Square::direction(FORWARD_RIGHT, us)) as u8, s as u8, Move::PC_BISHOP));
            }
        }

        moves
    }

    //
    //         return moves;
    //     }
    //
    //     @Override
    //     public String toString() {
    //         StringBuilder result = new StringBuilder(CHESSBOARD_LINE);
    //         for (int i = 56; i >= 0; i -= 8){
    //             for (int j = 0; j < 8; j++){
    //                 int piece = items[i + j];
    //                 String notation = Piece.getNotation(piece);
    //                 result.append("| ").append(notation).append(' ');
    //             }
    //             result.append("|\n").append(CHESSBOARD_LINE);
    //         }
    //         result.append("Fen: ").append(Fen.toFen(this));
    //         return result.toString();
    //     }

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

    //     public String toFen() {
    //         return Fen.toFen(this);
    //     }
    //
    //     public int mg() {
    //         return mg;
    //     }
    //
    //     public int eg() {
    //         return eg;
    //     }
    //
        pub fn interpolated_score(&self) -> i16 {
            let phase= ((self.phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE) as i16;
//        println!("mg: {}, eg: {}, phase: {}", self.mg, self.eg, phase);
            return (((self.mg as i32) * (256 - phase) as i32 + (self.eg as i32) * phase as i32) / 256) as i16;
            // self.mg
        }

    pub fn parse_moves(&self, parts: &Vec<&str>) -> Vec<Move> {
        let mut state = self.clone();
        let mut moves: Vec<Move> = Vec::new();

        for i in 0..parts.len() {
            let moov = Move::from_uci_string(parts[i], &state);
            state = state.do_move(&moov);
            state.pop_history(); // board_state doesn't have long history array typically
            moves.push(moov);
        }

        moves
    }


    //     public record Params(byte[] pieces, int wKingPos, int bKingPos) {}
    //
    //     public Params toParams() {
    //         byte[] result = new byte[80]; // 8 * 5 * 2
    //         int index = 0;
    //         for (int side = Side.WHITE; side <= Side.BLACK; side++) {
    //             for (int piece = PieceType.PAWN; piece <= PieceType.QUEEN; piece++) {
    //                 long bitboard = this.bitboard_of(side, piece);
    //                 for (int i = 0; i < 8; i++) {
    //                     result[index++] = (byte)((bitboard & 0xFF00000000000000L) >> 56);
    //                     bitboard <<= 8;
    //                 }
    //             }
    //
    //         }
    //         return new Params(result,
    //             Long.numberOfTrailingZeros(bitboard_of(Side.WHITE, PieceType.KING)),
    //             Long.numberOfTrailingZeros(bitboard_of(Side.BLACK, PieceType.KING)));
    //     }
    // fn pop_history(&mut self) -> Move {
    //     self.ply -= 1;
    //     Move::new_from_bits(self.history[self.ply])
    // }
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
