#![allow(unused_variables, dead_code)]

use std::fmt;
use crate::bitboard::{Bitboard, BitIter};
use crate::piece::{BLACK_BISHOP, BLACK_KING, BLACK_KNIGHT, BLACK_PAWN, BLACK_QUEEN, BLACK_ROOK, KING, KNIGHT, make_piece, NONE, PAWN, Piece, PIECES_COUNT, PieceType, to_piece_char, typeOf, WHITE_BISHOP, WHITE_KING, WHITE_KNIGHT, WHITE_PAWN, WHITE_QUEEN, WHITE_ROOK};
use crate::r#move::{Move, MoveList};
use crate::side::{BLACK, flip, Side, WHITE};
use crate::square::Square;

//     public static int TOTAL_PHASE = 24;
//     public static int[] PIECE_PHASES = {0, 1, 1, 2, 4, 0};

const CHESSBOARD_LINE: &'static str = "+---+---+---+---+---+---+---+---+\n";

pub struct BoardState {
    ply: usize,
    history: Vec<u64>,
    piece_bb: [u64; PIECES_COUNT],
    pub items: [Piece; 64],
    pub side_to_play: Side,
    hash: u64,
    pub full_move_normalized: usize,
    pub half_move_clock: usize,
    phase: u32,
    mg: i32,
    eg: i32,
    checkers: u64,
    pub(crate) movements: u64,
    pub en_passant: u64,

    pub(crate) bitboard: Bitboard,
}

impl fmt::Display for BoardState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
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
        max_search_depth: usize
    ) -> Self {
        if items.len() != 64 { panic!("Expected array with 64 items. Received {} items.", items.len()) }
        let mut board_state = BoardState {
            ply: 0,
            history: vec![],
            piece_bb: [0; PIECES_COUNT],
            items: [0; 64], //(*items).clone(),
            side_to_play,
            hash: 0,
            full_move_normalized: 0,
            half_move_clock,
            phase: 0,
            mg: 0,
            eg: 0,
            checkers: 0,
            movements,
            en_passant,
            bitboard: Bitboard::new()
        };

        for i in 0..64 {
            let item = items[i];
            if item != NONE {
                board_state.setPieceAt(item, i);
            } else {
                board_state.items[i] = NONE;
            }
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


    //
    //     public BoardState(int[] items, int sideToPlay, long movements, long enPassantMask, int halfMoveClock, int fullMoveCount, int maxSearchDepth) {
    //         for (int i = 0; i < 64; i++) {
    //             int item = items[i];
    //             if (item != Piece.NONE) {
    //                 setPieceAt(item, i);
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

        pub fn pieceTypeAt(&self, square: u8) -> PieceType {
            return typeOf(self.items[square as usize]);
        }

        pub fn setPieceAt(&mut self, piece: Piece, square: usize) {

            // //update incremental evaluation terms
            // phase -= PIECE_PHASES[Piece.typeOf(piece)];
            // mg += MGS[piece][square];
            // eg += EGS[piece][square];
            // // materialScore += materialValue(piece);

            //set piece on board
            self.items[square] = piece;
            self.piece_bb[piece as usize] |= 1u64 << square;

            // //update hashes
            // hash ^= Zobrist.ZOBRIST_TABLE[piece][square];
        }

    //     public void removePiece(int square){
    //         int piece = items[square];
    //         phase += PIECE_PHASES[Piece.typeOf(piece)];
    //         mg -= MGS[piece][square]; // EConstants.PIECE_TABLES[piece][square];
    //         eg -= EGS[piece][square];
    //
    //         //update hash tables
    //         hash ^= Zobrist.ZOBRIST_TABLE[items[square]][square];
    //
    //         //update board
    //         piece_bb[items[square]] &= ~(1L << square);
    //         items[square] = Piece.NONE;
    //     }
    //
    //     public void movePieceQuiet(int fromSq, int toSq){
    //         //update incremental evaluation terms
    //         int piece = items[fromSq];
    //         mg += MGS[piece][toSq] - MGS[piece][fromSq];
    //         eg += EGS[piece][toSq] - EGS[piece][fromSq];
    //
    //         //update hashes
    //         hash ^= Zobrist.ZOBRIST_TABLE[piece][fromSq] ^ Zobrist.ZOBRIST_TABLE[piece][toSq];
    //         //materialHash ^= Zobrist.ZOBRIST_TABLE[piece][fromSq] ^ Zobrist.ZOBRIST_TABLE[piece][toSq];
    //
    //         //update board
    //         piece_bb[piece] ^= (1L << fromSq | 1L << toSq);
    //         items[toSq] = piece;
    //         items[fromSq] = Piece.NONE;
    //     }
    //
    //     public void movePiece(int fromSq, int toSq){
    //         removePiece(toSq);
    //         movePieceQuiet(fromSq, toSq);
    //     }
    //
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

        pub fn attackersFrom(&self, square: u8, occ: u64, side: Side) -> u64 {
            match side {
                WHITE => {
                    (Bitboard::pawnAttacks(square as u64, BLACK) & self.piece_bb[WHITE_PAWN as usize]) |
                        (self.bitboard.getKnightAttacks(square as usize) & self.piece_bb[WHITE_KNIGHT as usize]) |
                        (self.bitboard.get_bishop_attacks(square as usize, occ) & (self.piece_bb[WHITE_BISHOP as usize] | self.piece_bb[WHITE_QUEEN as usize])) |
                        (self.bitboard.get_rook_attacks(square as usize, occ) & (self.piece_bb[WHITE_ROOK as usize] | self.piece_bb[WHITE_QUEEN as usize]))
                }
                _ => {
                    (Bitboard::pawnAttacks(square as u64, WHITE) & self.piece_bb[BLACK_PAWN as usize]) |
                        (self.bitboard.getKnightAttacks(square as usize) & self.piece_bb[BLACK_KNIGHT as usize]) |
                        (self.bitboard.get_bishop_attacks(square as usize, occ) & (self.piece_bb[BLACK_BISHOP as usize] | self.piece_bb[BLACK_QUEEN as usize])) |
                        (self.bitboard.get_rook_attacks(square as usize, occ) & (self.piece_bb[BLACK_ROOK as usize] | self.piece_bb[BLACK_QUEEN as usize]))
                }
            }
        }

    //     public long attackersFromIncludingKings(int square, long occ, int side){
    //         return side == Side.WHITE ? (pawnAttacks(square, Side.BLACK) & piece_bb[Piece.WHITE_PAWN]) |
    //                 (getKingAttacks(square) & piece_bb[Piece.WHITE_KING]) |
    //                 (getKnightAttacks(square) & piece_bb[Piece.WHITE_KNIGHT]) |
    //                 (getBishopAttacks(square, occ) & (piece_bb[Piece.WHITE_BISHOP] | piece_bb[Piece.WHITE_QUEEN])) |
    //                 (getRookAttacks(square, occ) & (piece_bb[Piece.WHITE_ROOK] | piece_bb[Piece.WHITE_QUEEN])) :
    //
    //                 (pawnAttacks(square, Side.WHITE) & piece_bb[Piece.BLACK_PAWN]) |
    //                 (getKingAttacks(square) & piece_bb[Piece.BLACK_KING]) |
    //                 (getKnightAttacks(square) & piece_bb[Piece.BLACK_KNIGHT]) |
    //                 (getBishopAttacks(square, occ) & (piece_bb[Piece.BLACK_BISHOP] | piece_bb[Piece.BLACK_QUEEN])) |
    //                 (getRookAttacks(square, occ) & (piece_bb[Piece.BLACK_ROOK] | piece_bb[Piece.BLACK_QUEEN]));
    //     }
    //
    //     public BoardState doMove(Move move) {
    //         return performMove(move, this);
    //     }
    //
    //     public BoardState doMove(String uciMove) {
    //         return performMove(this.generate_legal_moves().stream().filter(m->m.toString().equals(uciMove)).findFirst().orElseThrow(), this);
    //     }
    //
    //     public BoardState doNullMove() {
    //         return performNullMove(this);
    //     }
    //
    //     private BoardState performNullMove(BoardState oldBoardState) {
    //         BoardState state = oldBoardState.clone();
    //
    //         state.halfMoveClock += 1;
    //         state.clearEnPassant();
    //         state.sideToPlay = Side.flip(state.sideToPlay);
    //         state.hash ^= Zobrist.SIDE;
    //         return state;
    //     }
    //
    //
    //     public static BoardState performMove(Move move, BoardState oldBoardState) {
    //         BoardState state = oldBoardState.clone();
    //
    //         state.full_move_normalized += 1;
    //         state.halfMoveClock += 1;
    //         state.history[state.ply++] = move.bits();
    //         state.movements |= (1L << move.to() | 1L << move.from());
    //
    //         if (Piece.typeOf(state.items[move.from()]) == PieceType.PAWN)
    //             state.halfMoveClock = 0;
    //
    //         state.clearEnPassant();
    //
    //         switch (move.flags()){
    //             case Move.QUIET:
    //                 state.movePieceQuiet(move.from(), move.to());
    //                 break;
    //             case Move.DOUBLE_PUSH:
    //                 state.movePieceQuiet(move.from(), move.to());
    //                 state.enPassant = 1L << (move.from() + Square.direction(FORWARD, state.sideToPlay));
    //                 state.hash ^= Zobrist.EN_PASSANT[(int) (state.enPassant & 0b111)];
    //                 break;
    //             case Move.OO:
    //                 if (state.sideToPlay == Side.WHITE){
    //                     state.movePieceQuiet(E1, G1);
    //                     state.movePieceQuiet(H1, F1);
    //                 }
    //                 else {
    //                     state.movePieceQuiet(E8, G8);
    //                     state.movePieceQuiet(H8, F8);
    //                 }
    //                 break;
    //             case Move.OOO:
    //                 if (state.sideToPlay == Side.WHITE){
    //                     state.movePieceQuiet(E1, C1);
    //                     state.movePieceQuiet(A1, D1);
    //                 }
    //                 else {
    //                     state.movePieceQuiet(E8, C8);
    //                     state.movePieceQuiet(A8, D8);
    //                 }
    //                 break;
    //             case Move.EN_PASSANT:
    //                 state.movePieceQuiet(move.from(), move.to());
    //                 state.removePiece(move.to() + Square.direction(BACK, state.sideToPlay));
    //                 break;
    //             case Move.PR_KNIGHT:
    //                 state.removePiece(move.from());
    //                 state.setPieceAt(Piece.make_piece(state.sideToPlay, PieceType.KNIGHT), move.to());
    //                 break;
    //             case Move.PR_BISHOP:
    //                 state.removePiece(move.from());
    //                 state.setPieceAt(Piece.make_piece(state.sideToPlay, PieceType.BISHOP), move.to());
    //                 break;
    //             case Move.PR_ROOK:
    //                 state.removePiece(move.from());
    //                 state.setPieceAt(Piece.make_piece(state.sideToPlay, PieceType.ROOK), move.to());
    //                 break;
    //             case Move.PR_QUEEN:
    //                 state.removePiece(move.from());
    //                 state.setPieceAt(Piece.make_piece(state.sideToPlay, PieceType.QUEEN), move.to());
    //                 break;
    //             case Move.PC_KNIGHT:
    //                 state.removePiece(move.from());
    //                 state.removePiece(move.to());
    //                 state.setPieceAt(Piece.make_piece(state.sideToPlay, PieceType.KNIGHT), move.to());
    //                 break;
    //             case Move.PC_BISHOP:
    //                 state.removePiece(move.from());
    //                 state.removePiece(move.to());
    //                 state.setPieceAt(Piece.make_piece(state.sideToPlay, PieceType.BISHOP), move.to());
    //                 break;
    //             case Move.PC_ROOK:
    //                 state.removePiece(move.from());
    //                 state.removePiece(move.to());
    //                 state.setPieceAt(Piece.make_piece(state.sideToPlay, PieceType.ROOK), move.to());
    //                 break;
    //             case Move.PC_QUEEN:
    //                 state.removePiece(move.from());
    //                 state.removePiece(move.to());
    //                 state.setPieceAt(Piece.make_piece(state.sideToPlay, PieceType.QUEEN), move.to());
    //                 break;
    //             case Move.CAPTURE:
    //                 state.halfMoveClock = 0;
    //                 state.movePiece(move.from(), move.to());
    //                 break;
    //         }
    //         state.sideToPlay = Side.flip(state.sideToPlay);
    //         state.hash ^= Zobrist.SIDE;
    //
    //         return state;
    //     }
    //
    //     public int getSideToPlay(){
    //         return sideToPlay;
    //     }
    //
    //     public boolean kingAttacked(){
    //         final int us = sideToPlay;
    //         final int them = Side.flip(sideToPlay);
    //         final int ourKing = Long.numberOfTrailingZeros(bitboard_of(us, PieceType.KING));
    //
    //         if ((pawnAttacks(ourKing, us) & bitboard_of(them, PieceType.PAWN)) != 0)
    //             return true;
    //
    //         if ((getKnightAttacks(ourKing) & bitboard_of(them, PieceType.KNIGHT)) != 0)
    //             return true;
    //
    //         let usBb = all_pieces(us);
    //         let themBb = all_pieces(them);
    //         let all = usBb | themBb;
    //
    //         let theirDiagSliders = diagonal_sliders(them);
    //         let theirOrthSliders = orthogonal_sliders(them);
    //
    //         if ((getRookAttacks(ourKing, all) & theirOrthSliders) != 0)
    //             return true;
    //
    //         return (getBishopAttacks(ourKing, all) & theirDiagSliders) != 0;
    //     }
    //
    //
    //     /* not    side of the attacker */
    //     /**
    //      * @param side attacked side
    //      * @return attacked pieces
    //      */
    //     public long attackedPieces(int side) {
    //         BoardState workingState = this.getSideToPlay() == side ? this.doNullMove() : this;
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
    //         long pawns = pawnAttacks(square, us) & bitboard_of(them, PieceType.PAWN);
    //         if (pawns != 0)
    //             return Long.numberOfTrailingZeros(pawns);
    //
    //         long knights = getKnightAttacks(square) & bitboard_of(them, PieceType.KNIGHT);
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
    //             long kings = getKingAttacks(square) & bitboard_of(them, PieceType.KING);
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
    // //
    //     public boolean isRepetitionOrFifty(BoardPosition position){
    //         let lastMoveBits = this.ply > 0 ? this.history[this.ply - 1] : position.history[position.historyIndex - 1];
    //         int count = 0;
    //         int index = this.ply - 1;
    //         while (index >= 0) {
    //             if (this.history[index--] == lastMoveBits) {
    //                 count++;
    //             }
    //         }
    //         index = position.historyIndex - 1;
    //         while (index >= 0) {
    //             if (position.history[index--] == lastMoveBits) {
    //                 count++;
    //             }
    //         }
    //         return count > 2 || this.halfMoveClock >= 100;
    //     }
    //
    //     public boolean hasNonPawnMaterial(int side) {
    //         int start = Piece.make_piece(side, PieceType.KNIGHT);
    //         int end = Piece.make_piece(side, PieceType.QUEEN);
    //         for (int piece = start; piece <= end; piece++){
    //             if (bitboard_of(piece) != 0)
    //                 return true;
    //         }
    //         return false;
    //     }
    //
    //     public MoveList generate_legal_moves(){
    //         return this.generate_legal_moves(false);
    //     }
    //
    //     public MoveList generateLegalQuiescence(){
    //         return generate_legal_moves(true);
    //     }

    pub fn generate_legal_moves(&mut self, only_quiescence: bool) -> MoveList {
        let mut moves = MoveList::new();
        let us = self.side_to_play;
        let them = flip(self.side_to_play);

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

        // General purpose to keep down initialized primitives
        let mut b1: u64 = 0;
        let mut b2: u64 = 0;
        let mut b3: u64 = 0;

        // Squares that the king can't move to
        let mut underAttack: u64 = 0;
        underAttack |= Bitboard::pawnAttacks(self.bitboard_of(them, PAWN), them) | self.bitboard.getKingAttacks(their_king);

        for b1 in BitIter(self.bitboard_of(them, KNIGHT)) {
            underAttack |= self.bitboard.getKnightAttacks(b1 as usize);
        }

        for b1 in BitIter(their_bishops_and_queens) {
            underAttack |= self.bitboard.get_bishop_attacks(b1.trailing_zeros() as usize, all ^ (1u64 << our_king as u8));
        }

        for b1 in BitIter(their_rooks_and_queens) {
            underAttack |= self.bitboard.get_rook_attacks(b1.trailing_zeros() as usize, all ^ (1u64 << our_king as u8));
        }

        b1 = self.bitboard.getKingAttacks(our_king) & !(us_bb | underAttack);

        moves.makeQuiets(our_king as u8, b1 & !them_bb);
        moves.makeCaptures(our_king as u8, b1 & them_bb);

        //captureMask contains destinations where there is an enemy piece that is checking the king and must be captured
        //quietMask contains squares where pieces must be moved to block an incoming attack on the king
        let mut captureMask: u64;
        let mut quietMask: u64;
        //let mut s: u8;

        // checker moves from opposite knights and pawns
        self.checkers = (self.bitboard.getKnightAttacks(our_king) & self.bitboard_of(them, KNIGHT))
                | (Bitboard::pawnAttacksFromSquare(our_king as u8, us) & self.bitboard_of(them, PAWN));

        // ray candidates to our king
        let mut candidates = (self.bitboard.get_rook_attacks(our_king, them_bb) & their_rooks_and_queens)
                | (self.bitboard.get_bishop_attacks(our_king, them_bb) & their_bishops_and_queens);

        let mut pinned: u64 = 0;

        for ray_candidate in BitIter(candidates) {
            // squares obstructed by our pieces
            let squares_between = self.bitboard.between(our_king as u8, ray_candidate as u8) & us_bb;

            // king is not guarded by any of our pieces
            if squares_between == 0 {
                self.checkers ^= 1u64 << ray_candidate;
            // when there's only one piece between king and a sliding piece, the piece is pinned
            } else if squares_between.count_ones() == 1 {
                pinned ^= squares_between;
            }
        }


        let notPinned = !pinned;
        if self.checkers.count_ones() == 2 {
            // our king is in check mate
            return moves;
        } else if self.checkers.count_ones() == 1 {
            // our king is checked
            let checkerSquare: usize = self.checkers.trailing_zeros() as usize;
            let checker_piece_type = typeOf(self.items[checkerSquare] as Piece);
            // for checking sliding pieces
            if checker_piece_type != PAWN && checker_piece_type != KNIGHT {
                // we have to capture them
                captureMask = self.checkers;
                // ...or block 'em
                quietMask = self.bitboard.between(our_king as u8, checkerSquare as u8);
            } else {
                // for checking en-passants
                if checker_piece_type == PAWN && self.checkers == (if us == WHITE { self.en_passant >> 8 } else { self.en_passant << 8 }) {
                    // we have to consider taking the pawn en passant
                    let enPassantSquare = self.en_passant.trailing_zeros();
                    for b1 in BitIter(Bitboard::pawnAttacksFromSquare(enPassantSquare as u8, them) & self.bitboard_of(us, PAWN) & notPinned) {
                        moves.add(Move::newFromFlags(b1.trailing_zeros() as u8, enPassantSquare as u8, Move::EN_PASSANT));
                    }
                }

                // capture the checking piece
                for sq in BitIter(self.attackersFrom(checkerSquare as u8, all, us) & notPinned) {
                    if self.pieceTypeAt(sq as u8) == PAWN && (1u64 << sq & Bitboard::PAWN_FINAL_RANKS) != 0u64 {
                        moves.add(Move::newFromFlags(sq as u8, checkerSquare as u8, Move::PC_QUEEN));
                        moves.add(Move::newFromFlags(sq as u8, checkerSquare as u8, Move::PC_ROOK));
                        moves.add(Move::newFromFlags(sq as u8, checkerSquare as u8, Move::PC_KNIGHT));
                        moves.add(Move::newFromFlags(sq as u8, checkerSquare as u8, Move::PC_BISHOP));
                    }
                    else {
                        moves.add(Move::newFromFlags(sq as u8, checkerSquare as u8, Move::CAPTURE));
                    }
                }
                return moves;
            }
        // our king is not checked
        } else {
            captureMask = them_bb;

            quietMask = !all;

            if self.en_passant != 0u64 {
                let enPassantSquare = self.en_passant.trailing_zeros();
                b2 = Bitboard::pawnAttacksFromSquare(enPassantSquare as u8, them) & self.bitboard_of(us, PAWN);
                // b2 holds pawns that can do an ep capture
                for s in BitIter(b2 & notPinned) {
                    // s hold square from which pawn attack to epsq can be done
                    // s = Long.numberOfTrailingZeros(b1);
                    // b1 = Bitboard.extractLsb(b1);

    //                        long attacks = Attacks.slidingAttacks(ourKing,
    //                                all ^ 1L << s) ^ Bitboard.shift(1L << this.epsq), Square.relative_dir(Square.SOUTH, us)),
    //                                Rank.getBb(Square.getRank(ourKing)));

                    // Bitboard.shift(1L << this.epsq), Square.relative_dir(Square.SOUTH, us)) holds pawn which can be en-passant taken
                    let qqq = them_bb ^ (if us == WHITE { self.en_passant >> 8 } else { self.en_passant << 8 });
                    candidates = (self.bitboard.get_rook_attacks(our_king, qqq | us_bb) & their_rooks_and_queens)
                            | (self.bitboard.get_bishop_attacks(our_king, qqq | us_bb) & their_bishops_and_queens);

                    if candidates == 0 {
                        moves.add(Move::newFromFlags(s as u8, enPassantSquare as u8, Move::EN_PASSANT));
                    }
                }
            }

            if !only_quiescence {
                if 0 == ((self.movements & Bitboard::castling_pieces_kingside_mask(us)) | ((all | underAttack) & Bitboard::castlingBlockersKingsideMask(us))) {
                    moves.add(if us == WHITE { Move::newFromFlags(Square::E1, Square::G1, Move::OO) }
                              else { Move::newFromFlags(Square::E8, Square::G8, Move::OO) });
                }

                if 0 == ((self.movements & Bitboard::castling_pieces_queenside_mask(us)) |
                        ((all | (underAttack & !Bitboard::ignoreOOODanger(us))) & Bitboard::castlingBlockersQueensideMask(us))) {
                    moves.add(if us == WHITE { Move::newFromFlags(Square::E1, Square::C1, Move::OOO) }
                              else { Move::newFromFlags(Square::E8, Square::C8, Move::OOO)});
                }
            }

            // all pinned sliding pieces can only eliminate the threat or move while staying pinned
            b1 = !(notPinned | self.bitboard_of(us, KNIGHT));
            for s in BitIter(b1) {
                b2 = self.bitboard.attacks(self.pieceTypeAt(s as u8), s as u8, all) & self.bitboard.line(our_king as u8, s as u8);
                if !only_quiescence {
                    moves.makeQuiets(s as u8, b2 & quietMask);
                }
                moves.makeCaptures(s as u8, b2 & captureMask);
            }

            // for each pinned pawn
            b1 = !notPinned & self.bitboard_of(us, PAWN);
            for s in BitIter(b1) {
                if ((1u64 << s) & Bitboard::PAWN_FINAL_RANKS) != 0 {
                    b2 = Bitboard::pawnAttacksFromSquare(s as u8, us) & captureMask & self.bitboard.line(our_king as u8, s as u8);
                    moves.makePromotionCaptures(s as u8, b2);
                } else {
                    b2 = Bitboard::pawnAttacksFromSquare(s as u8, us) & them_bb & self.bitboard.line(s as u8, our_king as u8);
                    moves.makeCaptures(s as u8, b2);

                    if !only_quiescence {
                        //single pawn pushes
                        b2 = Bitboard::push(1u64 << s, us) & !all & self.bitboard.line(our_king as u8, s as u8);
                        b3 = Bitboard::push(b2 & Bitboard::PAWN_DOUBLE_PUSH_LINES[us as usize], us) & !all & self.bitboard.line(our_king as u8, s as u8);

                        moves.makeQuiets(s as u8, b2);
                        moves.makeDoublePushes(s as u8, b3);
                    }
                }
            }

            // pinned knights cannot move anyway, so let them stay
        }


        moves
    }

    //
    //         //non-pinned knight moves.
    //         b1 = bitboard_of(us, PieceType.KNIGHT) & notPinned;
    //         while (b1 != 0){
    //             s = Long.numberOfTrailingZeros(b1);
    //             b1 = Bitboard.extractLsb(b1);
    //             b2 = getKnightAttacks(s);
    //             moves.makeC(s, b2 & captureMask);
    //             if (!onlyQuiescence) {
    //                 moves.makeQ(s, b2 & quietMask);
    //             }
    //         }
    //
    //         b1 = ourBishopsAndQueens & notPinned;
    //         while (b1 != 0){
    //             s = Long.numberOfTrailingZeros(b1);
    //             b1 = Bitboard.extractLsb(b1);
    //             b2 = getBishopAttacks(s, all);
    //             moves.makeC(s, b2 & captureMask);
    //             if (!onlyQuiescence) {
    //                 moves.makeQ(s, b2 & quietMask);
    //             }
    //         }
    //
    //         b1 = ourRooksAndQueens & notPinned;
    //         while(b1 != 0){
    //             s = Long.numberOfTrailingZeros(b1);
    //             b1 = Bitboard.extractLsb(b1);
    //             b2 = getRookAttacks(s, all);
    //             moves.makeC(s, b2 & captureMask);
    //             if (!onlyQuiescence) {
    //                 moves.makeQ(s, b2 & quietMask);
    //             }
    //         }
    //
    //         b1 = bitboard_of(us, PieceType.PAWN) & notPinned & ~PAWN_RANKS[us];
    //
    //         if (!onlyQuiescence) {
    //             // single pawn pushes
    //             b2 = (us == Side.WHITE ? b1 << 8 : b1 >>> 8) & ~all;
    //
    //             //double pawn pushes
    //             b3 = Bitboard.push(b2 & PAWN_DOUBLE_PUSH_LINES[us], us) & quietMask;
    //
    //             b2 &= quietMask;
    //
    //             while (b2 != 0) {
    //                 s = Long.numberOfTrailingZeros(b2);
    //                 b2 = Bitboard.extractLsb(b2);
    //                 moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.QUIET));
    //             }
    //
    //             while (b3 != 0) {
    //                 s = Long.numberOfTrailingZeros(b3);
    //                 b3 = Bitboard.extractLsb(b3);
    //                 moves.add(new Move(s - Square.direction(DOUBLE_FORWARD, us), s, Move.DOUBLE_PUSH));
    //             }
    //         }
    //
    //         b2 = (us == Side.WHITE ? whiteLeftPawnAttacks(b1) : blackRightPawnAttacks(b1)) & captureMask;
    //         b3 = (us == Side.WHITE ? whiteRightPawnAttacks(b1) : blackLeftPawnAttacks(b1)) & captureMask;
    //
    //
    //         while (b2 != 0){
    //             s = Long.numberOfTrailingZeros(b2);
    //             b2 = Bitboard.extractLsb(b2);
    //             moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.CAPTURE));
    //         }
    //
    //         while (b3 != 0){
    //             s = Long.numberOfTrailingZeros(b3);
    //             b3 = Bitboard.extractLsb(b3);
    //             moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.CAPTURE));
    //         }
    //
    //         b1 = bitboard_of(us, PieceType.PAWN) & notPinned & PAWN_RANKS[us];
    //         if (b1 != 0){
    //             if (!onlyQuiescence) {
    //                 b2 = (us == Side.WHITE ? b1 << 8 : b1 >>> 8) & quietMask;
    //                 while (b2 != 0) {
    //                     s = Long.numberOfTrailingZeros(b2);
    //                     b2 = Bitboard.extractLsb(b2);
    //
    //                     moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.PR_QUEEN));
    //                     moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.PR_ROOK));
    //                     moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.PR_KNIGHT));
    //                     moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.PR_BISHOP));
    //                 }
    //             }
    //
    //             b2 = (us == Side.WHITE ? whiteLeftPawnAttacks(b1) : blackRightPawnAttacks(b1)) & captureMask;
    //             b3 = (us == Side.WHITE ? whiteRightPawnAttacks(b1) : blackLeftPawnAttacks(b1)) & captureMask;
    //
    //             while (b2 != 0){
    //                 s = Long.numberOfTrailingZeros(b2);
    //                 b2 = Bitboard.extractLsb(b2);
    //
    //                 moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_QUEEN));
    //                 moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_ROOK));
    //                 moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_KNIGHT));
    //                 moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_BISHOP));
    //             }
    //
    //             while (b3 != 0){
    //                 s = Long.numberOfTrailingZeros(b3);
    //                 b3 = Bitboard.extractLsb(b3);
    //
    //                 moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_QUEEN));
    //                 moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_ROOK));
    //                 moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_KNIGHT));
    //                 moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_BISHOP));
    //             }
    //         }
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
    //
    //     private void clearEnPassant() {
    //         // TODO zjednodusit
    //         long previous_state = this.enPassant;
    //
    //         if (previous_state != 0L) {
    //             this.enPassant = 0L;
    //             this.hash ^= Zobrist.EN_PASSANT[(int) (previous_state & 0b111)];
    //         }
    //     }
    //
    //     public BoardState forSearchDepth(int searchDepth) {
    //         BoardState result = this.clone();
    //         result.history = new long[searchDepth];
    //         result.ply = 0;
    //         return result;
    //     }
    //
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
    //     public int interpolatedScore() {
    //         int phase = (this.phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE;
    //         return (this.mg() * (256 - phase) + this.eg() * phase) / 256;
    //     }
    //
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
}


#[cfg(test)]
mod tests {
    use crate::fen::{from_fen_default, to_fen};

    #[test]
    fn from_fen_startpos() {
        let mut state = from_fen_default("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        let moves = state.generate_legal_moves(false);
        // assert_eq!(state.to_string(), );
    }
}
