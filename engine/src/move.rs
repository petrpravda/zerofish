use std::borrow::Borrow;
use std::fmt;
use crate::bitboard::BitIter;
use crate::board_state::BoardState;
use crate::piece::{make_piece, PieceType};
use crate::piece_square_table::MGS;
use crate::side::Side;
use crate::square::Square;
use crate::transposition::{BaseMove, TranspositionTable, Value};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move {
    pub(crate) bits: u32,
    sort_score: i32
}

//
// +---------------+----+-------+
// | To            |  6 | 5-0   |
// | From          |  6 | 11-6  |
// | Flags         |  4 | 15-12 |
// | Score         | 32 | 63-32 |
// +---------------+----+-------+
//

impl Move {
    pub const QUIET: u8 = 0b0000;
    pub const DOUBLE_PUSH: u8 = 0b0001;
    pub const OO: u8 = 0b0010;
    pub const OOO: u8 = 0b0011;
    pub const CAPTURE: u8 = 0b0100;
    pub const EN_PASSANT: u8 = 0b0101;
    pub const PROMOTION: u8 = 0b1000;
    pub const PR_KNIGHT: u8 = 0b1000;
    pub const PR_BISHOP: u8 = 0b1001;
    pub const PR_ROOK: u8 = 0b1010;
    pub const PR_QUEEN: u8 = 0b1011;
    pub const PC_KNIGHT: u8 = 0b1100;
    pub const PC_BISHOP: u8 = 0b1101;
    pub const PC_ROOK: u8 = 0b1110;
    pub const PC_QUEEN: u8 = 0b1111;
    pub const NULL: u8 = 0b0111;

    pub const NULL_MOVE: Move = Move { bits: 0, sort_score: 0 };
    pub const BM_NULL: BaseMove = 0;

    pub fn new() -> Self { Self{bits: 0, sort_score: 0 }}
    pub fn new_from_bits(m: u32) -> Self { Self{bits: m, sort_score: 0 }}
    pub fn new_from_to(from: u8, to: u8) -> Self { Self{bits: (from as u32) << 6 | (to as u32), sort_score: 0 }}
    pub fn new_from_flags(from: u8, to: u8, flags: u8) -> Self {
        Self{bits: (flags as u32) << 12 | (from as u32) << 6 | (to as u32), sort_score: 0 }}

    pub fn to(&self) -> u8 {
        (self.bits & 0x3f) as u8
    }

    pub fn from(&self) -> u8 {
        ((self.bits >> 6) & 0x3f) as u8
    }

    #[inline(always)]
    pub fn flags(&self) -> u8 {
        ((self.bits >> 12) & 0xf) as u8
    }

    pub fn score(&self) -> i32 {
        self.sort_score
    }

    pub fn base_move(&self) -> BaseMove { self.bits as BaseMove }

    #[inline(always)] // TODO what if I use self.flags()? is it the same speed?
    pub fn is_promotion(&self) -> bool {
        ((self.bits >> 12) as u8 & Move::PROMOTION) != 0
    }

    #[inline(always)]
    pub fn is_quiet(&self) -> bool { // TODO what if I use self.flags()? is it the same speed?
        (self.bits >> 12) as u8 & Move::PROMOTION == 0
    }

    #[inline(always)]
    pub fn is_capture(&self) -> bool {
        (self.bits >> 12) as u8 & Move::CAPTURE != 0
    }

    #[inline(always)]
    pub fn is_castling(&self) -> bool {
        matches!(self.flags(), Move::OO | Move::OOO)
    }

    #[inline(always)]
    pub fn is_ep(&self) -> bool {
        self.flags() == Move::EN_PASSANT
    }

    // @Override
    // public boolean equals(Object other) {
    // if (other != null && getClass() == other.getClass())
    // return this.bits == ((Move)other).bits();
    // return false;
    // }

    pub fn get_piece_type(&self) -> PieceType {
        PieceType::from((self.flags() & 0b11) + 1)
    }

    // pub fn get_piece_type_for_side(&self, side_to_play: Side) -> PieceType {
    //     return PieceType::from(self.get_piece_type() as u8 + (side_to_play as u8 * 8));
    // }

    pub fn uci(&self) -> String {
        let promo = match self.flags() {
            Move::PC_BISHOP | Move::PR_BISHOP => "b",
            Move::PC_KNIGHT | Move::PR_KNIGHT => "n",
            Move::PC_ROOK | Move::PR_ROOK => "r",
            Move::PC_QUEEN | Move::PR_QUEEN => "q",
            _ => ""
        };
    //     String promo = switch (this.flags()) {
    //     case Move.PC_BISHOP, Move.PR_BISHOP -> "b";
    //     case Move.PC_KNIGHT, Move.PR_KNIGHT -> "n";
    //     case Move.PC_ROOK, Move.PR_ROOK -> "r";
    //     case Move.PC_QUEEN, Move.PR_QUEEN -> "q";
    //     default -> "";
    // };
    //    Square.getName(this.from()) + Square.getName(this.to()) + promo;
        format!("{}{}{}",
               Square::get_name(self.from() as usize),
               Square::get_name(self.to() as usize),
               promo)
    }

    pub fn addToScore(&mut self, score: i32){
        self.sort_score += score;
    }

    // @Override
    // public String toString() {
    // return uci();
    // }
    //
    // public static List<Move> parseUciMoves(List<String> moves) {
    // return moves.stream()
    // .map(Move::fromUciString)
    // .collect(Collectors.toList());
    // }
    //
    // public static Move fromUciString(String str) {
    // int fromSq = Square.getSquareFromName(str.substring(0, 2));
    // int toSq = Square.getSquareFromName(str.substring(2, 4));
    // String typeStr = "";
    // if (str.length() > 4)
    // typeStr = str.substring(4);
    //
    // Move move;
    //
    // if (typeStr.equals("q"))
    // move = new Move(fromSq, toSq, Move.PR_QUEEN);
    // else if (typeStr.equals("n"))
    // move = new Move(fromSq, toSq, Move.PR_KNIGHT);
    // else if (typeStr.equals("b"))
    // move = new Move(fromSq, toSq, Move.PR_BISHOP);
    // else if (typeStr.equals("r"))
    // move = new Move(fromSq, toSq, Move.PR_ROOK);
    // else
    // move = new Move(fromSq, toSq, Move.QUIET);
    //
    // return move;
    // }
    //
    // public static Move fromFirstUciSubstring(String movesDelimitedWithSpace) {
    // String[] moves = movesDelimitedWithSpace.split(" ");
    // return fromUciString(moves[0]);
    // }
    //
    // public boolean isCastling() {
    // return this.flags() == Move.OO || this.flags() == Move.OOO;
    // }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}


pub struct MoveList {
    pub moves: Vec<Move>,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: Vec::with_capacity(64)
        }
    }

    pub fn make_quiets(&mut self, from: u8, targets: u64) {
        for to in BitIter(targets) {
            self.moves.push(Move::new_from_flags(from, to as u8, Move::QUIET));
        }
    }

    pub fn make_captures(&mut self, from: u8, targets: u64) {
        for to in BitIter(targets) {
            self.moves.push(Move::new_from_flags(from, to as u8, Move::CAPTURE));
        }
    }

    pub fn make_promotion_captures(&mut self, from: u8, targets: u64) {
        for to in BitIter(targets) {
            self.moves.push(Move::new_from_flags(from, to as u8, Move::PC_QUEEN));
            self.moves.push(Move::new_from_flags(from, to as u8, Move::PC_KNIGHT));
            self.moves.push(Move::new_from_flags(from, to as u8, Move::PC_ROOK));
            self.moves.push(Move::new_from_flags(from, to as u8, Move::PC_BISHOP));
        }
    }

    pub fn make_double_pushes(&mut self, from: u8, targets: u64) {
        for to in BitIter(targets) {
            self.moves.push(Move::new_from_flags(from, to as u8, Move::DOUBLE_PUSH));
        }
    }

    pub fn add(&mut self, mowe: Move) {
        self.moves.push(mowe);
    }

    pub fn len(&self) -> usize {
        self.moves.len()
    }

    //final BoardState state, TranspTable transposition_table, int ply, MoveOrdering moveOrdering) {
    pub fn score_moves(&mut self, state: &BoardState, transposition_table: &TranspositionTable) {
        if self.moves.len() == 0 {
            return;
        }

        let tt_entry = transposition_table.probe(state);
        let hash_move = tt_entry.map(|tt| tt.best_move());

        for index in 1..self.moves.len() {
            let moov = self.moves[index];

            if hash_move.is_some() && moov.base_move() == hash_move.unwrap() {
                //moov.addToScore(MoveOrdering::HashMoveScore);
                self.moves[index].addToScore(MoveOrdering::HashMoveScore);
                continue;
            }

            // if moov.is_quiet() {
            //     // if self.is_killer(board, m, ply) {
            //     //     moves.scores[idx] += Self::KILLER_MOVE_SCORE;
            //     //     continue;
            //     // }
            //
            //     if moov.is_castling() {
            //         self.moves[index].addToScore(MoveOrdering::CASTLING_SCORE);
            //         continue;
            //     }
            //
            //     // moves.scores[idx] += Self::HISTORY_MOVE_OFFSET + self.history_score(m);
            //     continue;
            // }

            // if moov.is_capture() {
            //     if moov.is_ep() {
            //         self.moves[index].addToScore(MoveOrdering::WINNING_CAPTURES_OFFSET);
            //         continue;
            //     }
            //
            //     moves.scores[idx] += Self::mvv_lva_score(board, m);
            //
            //     if Self::see(board, m, -100) {
            //         moves.scores[idx] += Self::WINNING_CAPTURES_OFFSET;
            //     } else {
            //         moves.scores[idx] += Self::LOSING_CAPTURES_OFFSET;
            //     }
            // }
            //
            // moves.scores[idx] += match m.promotion() {
            //     PieceType::Knight => Self::KNIGHT_PROMOTION_SCORE,
            //     PieceType::Bishop => Self::BISHOP_PROMOTION_SCORE,
            //     PieceType::Rook => Self::ROOK_PROMOTION_SCORE,
            //     PieceType::Queen => Self::QUEEN_PROMOTION_SCORE,
            //     PieceType::None => 0,
            //     _ => unreachable!(),
            // };

//            if (moveOrdering.isKiller(state, move, ply)) {
//                move.addToScore(MoveOrdering.KillerMoveScore);
//            }

            let piece = state.items[moov.from() as usize];
//
            match moov.flags() {
                Move::PC_BISHOP | Move::PC_KNIGHT | Move::PC_ROOK | Move::PC_QUEEN => {
                    let score = (MGS[make_piece(state.side_to_play, moov.get_piece_type()) as usize][moov.to() as usize]
                        - MGS[piece as usize][moov.from() as usize]
                        - MGS[state.items[moov.to() as usize] as usize][moov.to() as usize])
                        * state.side_to_play.multiplicator() as i32;
                    self.moves[index].addToScore(score);
                },
                Move::PR_BISHOP | Move::PR_KNIGHT | Move::PR_ROOK | Move::PR_QUEEN => {
                    let score = (MGS[make_piece(state.side_to_play, moov.get_piece_type()) as usize][moov.to() as usize]
                        - MGS[piece as usize][moov.from() as usize])
                        * state.side_to_play.multiplicator() as i32;
                    self.moves[index].addToScore(score);
                },
                Move::CAPTURE => {
                    let score = (MGS[piece as usize][moov.to() as usize]
                        - MGS[piece as usize][moov.from() as usize]
                        - MGS[state.items[moov.to() as usize] as usize][moov.to() as usize])
                        * state.side_to_play.multiplicator() as i32;
                    self.moves[index].addToScore(score);
                },
                Move::QUIET | Move::EN_PASSANT | Move::DOUBLE_PUSH | Move::OO | Move::OOO => {
                    let score = (MGS[piece as usize][moov.to() as usize]
                        - MGS[piece as usize][moov.from() as usize])
                        * state.side_to_play.multiplicator() as i32;
                    self.moves[index].addToScore(score);
                },
                _ => unreachable!(),
            }
        }
    }

    pub fn pick_next_best_move(&mut self, cur_index: usize){
        let size = self.moves.len();
        let mut max = i32::MAX;
        let mut max_index = 0;
        let mut i = cur_index;
        while i < size {
            if self.moves[i].score() > max {
                max = self.moves[i].score();
                max_index = i;
            }
            i += 1;
        }
        self.moves.swap(cur_index, max_index);
    }

    pub fn to_string(&self) -> String {
        format!("length: {}", self.moves.len())
    }

    pub fn over_sorted<'a>(&'a mut self, state: &'a BoardState, transposition_state: &'a TranspositionTable) -> SortedMovesIter<'a> {
        &self.score_moves(state, transposition_state);
        SortedMovesIter {
            state, // TODO check if it is really needed to keep the references
            transposition_state,
            move_list: self,
            index: 0,
        }
    }
}

pub struct SortedMovesIter<'a> {
    state: &'a BoardState,
    transposition_state: &'a TranspositionTable,
    move_list: &'a mut MoveList,
    index: usize,
}
impl<'a> Iterator for SortedMovesIter<'a> {
    type Item = &'a Move;
    //fn next(&mut self) -> Option<&Move> {
    //fn next<'a>(&'a mut self) -> Option<&'a Move> {
    //fn next<'b>(&'a mut self) -> Option<&'a Move> where 'a: 'b {
    fn next(&'a mut self) -> Option<&'a Move> {
        if self.index == self.move_list.len() {
            return None;
        }

        self.move_list.pick_next_best_move(self.index);
        self.index += 1;
        let moov: &'a Move = &self.move_list.moves[self.index];
        Some(moov)
    }
}

impl fmt::Display for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ucis = self.moves.iter().map(|m| m.uci()).collect::<Vec<String>>().join(" ");
        write!(f, "Move list: {}", ucis)
    }
}

pub struct MoveOrdering {

}

impl MoveOrdering {
    pub const HashMoveScore: Value = 10000; // 25000?
    //const N_KILLERS: usize = 3;
    const QUEEN_PROMOTION_SCORE: Value = 8000;
    const ROOK_PROMOTION_SCORE: Value = 7000;
    const BISHOP_PROMOTION_SCORE: Value = 6000;
    const KNIGHT_PROMOTION_SCORE: Value = 5000;
    const WINNING_CAPTURES_OFFSET: Value = 10;
    const KILLER_MOVE_SCORE: Value = 2;
    const CASTLING_SCORE: Value = 1;
    const HISTORY_MOVE_OFFSET: Value = -30000;
    const LOSING_CAPTURES_OFFSET: Value = -30001;
}
//    private final int[][][] killerMoves = new int[2][1000][1];
