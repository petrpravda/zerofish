use std::borrow::Borrow;
use std::fmt;
use crate::bitboard::BitIter;
use crate::board_state::BoardState;
use crate::piece::{make_piece, NONE, Piece, PieceType};
use crate::piece_square_table::MGS;
use crate::side::Side;
use crate::square::Square;
use crate::transposition::{ TranspositionTable, Value};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Move {
    pub bits: u16,
}

//
// +---------------+----+-------+
// | To            |  6 | 5-0   |
// | From          |  6 | 11-6  |
// | Flags         |  4 | 15-12 |
// +---------------+----+-------+
//

impl Move {
    pub const QUIET:       u16 = 0b0000000000000000;
    pub const DOUBLE_PUSH: u16 = 0b0001000000000000;
    pub const OO:          u16 = 0b0010000000000000;
    pub const OOO:         u16 = 0b0011000000000000;
    pub const CAPTURE:     u16 = 0b0100000000000000;
    pub const EN_PASSANT:  u16 = 0b0101000000000000;
    pub const PROMOTION:   u16 = 0b1000000000000000;
    pub const PR_KNIGHT:   u16 = 0b1000000000000000;
    pub const PR_BISHOP:   u16 = 0b1001000000000000;
    pub const PR_ROOK:     u16 = 0b1010000000000000;
    pub const PR_QUEEN:    u16 = 0b1011000000000000;
    pub const PC_KNIGHT:   u16 = 0b1100000000000000;
    pub const PC_BISHOP:   u16 = 0b1101000000000000;
    pub const PC_ROOK:     u16 = 0b1110000000000000;
    pub const PC_QUEEN:    u16 = 0b1111000000000000;
    pub const FLAGS_MASK:  u16 = 0b1111000000000000;
    pub const NULL:        u16 = 0b0111000000000000;

    pub const NULL_MOVE: Move = Move { bits: Move::NULL };

    //pub fn new() -> Self { Self{bits: 0, sort_score: 0 }}
    pub fn new_from_bits(m: u16) -> Self { Self{bits: m }}
    pub fn new_from_to(from: u8, to: u8) -> Self { Self{bits: (from as u16) << 6 | (to as u16) }}
    pub fn new_from_flags(from: u8, to: u8, flags: u16) -> Self {
        Self{bits: flags | (from as u16) << 6 | (to as u16) }}

    pub fn to(&self) -> u8 {
        (self.bits & 0x3f) as u8
    }

    pub fn from(&self) -> u8 {
        ((self.bits >> 6) & 0x3f) as u8
    }

    #[inline(always)]
    pub fn flags(&self) -> u16 {
        self.bits & Move::FLAGS_MASK
    }

    #[inline(always)]
    pub fn is_promotion(&self) -> bool {
        self.bits & Move::PROMOTION != 0
    }

    // #[inline(always)]
    // pub fn is_quiet(&self) -> bool {
    //     self.bits & Move::PROMOTION == 0
    // }

    // #[inline(always)]
    // pub fn is_capture(&self) -> bool {
    //     self.bits & Move::CAPTURE != 0
    // }
    //
    // #[inline(always)]
    // pub fn is_castling(&self) -> bool {
    //     let flags = self.flags();
    //     flags == Move::OO || flags == Move::OOO
    // }
    //
    // #[inline(always)]
    // pub fn is_ep(&self) -> bool {
    //     self.flags() == Move::EN_PASSANT
    // }



    pub fn get_piece_type(&self) -> PieceType {
        PieceType::from((((self.flags() >> 12) & 0b11) + 1) as u8)
    }

    // pub fn make_piece_type_promotion_flags(piece_type: PieceType) -> u8 {
    //     (piece_type as u8 - 1) | Move::PROMOTION
    // }
    //
    pub fn uci(&self) -> String {
        let promo = match self.flags() {
            Move::PC_BISHOP | Move::PR_BISHOP => "b",
            Move::PC_KNIGHT | Move::PR_KNIGHT => "n",
            Move::PC_ROOK | Move::PR_ROOK => "r",
            Move::PC_QUEEN | Move::PR_QUEEN => "q",
            _ => ""
        };
        format!("{}{}{}",
               Square::get_name(self.from() as usize),
               Square::get_name(self.to() as usize),
               promo)
    }

    // pub fn addToScore(&mut self, score: i32){
    //     self.sort_score += score;
    // }


    pub fn from_uci_string(uci: &str, state: &BoardState) -> Move {
        let bytes = uci.as_bytes();
        if bytes.len() < 4 {
            panic!("Invalid uci move notation: {}", uci);
        }

        let start_col = bytes[0] - b'a';
        let start_row = b'8' - bytes[1];
        let from_sq = (start_row * 8 + start_col) as u8;

        let end_col = bytes[2] - b'a';
        let end_row = b'8' - bytes[3];
        let to_sq = (end_row * 8 + end_col) as u8;

        let capture = state.piece_at(to_sq) != NONE;

        let promotion = if bytes.len() == 5 {
            Some(match bytes[4] {
                b'q' => Move::PR_QUEEN,
                b'r' => Move::PR_ROOK,
                b'b' => Move::PR_BISHOP,
                b'n' => Move::PR_KNIGHT,
                _ => {
                    panic!("Invalid promotion piece in UCI notation: {}", uci);
                }
            })
        } else {
            None
        };

        let flags = promotion.map(|p| p | p | if capture { Move::CAPTURE } else { Move::QUIET }).unwrap_or(0u16);
        Move::new_from_flags(from_sq as u8, to_sq as u8, flags)
    }
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

    pub fn clone(&self) -> Self {
        Self {
            moves: self.moves.to_vec()
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
    pub fn score_moves(&mut self, state: &BoardState, transposition_table: &TranspositionTable) -> Vec<ScoredMove> {
        if self.moves.len() == 0 {
            return Vec::new()
        }

        let tt_entry = transposition_table.probe(state);
        let hash_move = tt_entry.map(|tt| tt.best_move());

        let result: Vec<ScoredMove> = self.moves.iter().map(|moov| {
            let move_score = if hash_move.is_some() && hash_move.unwrap().bits == moov.bits
                    { MoveOrdering::HashMoveScore } else { 0 };
            let piece = state.items[moov.from() as usize];
//
            let pieces_score: Value = match moov.flags() {
                Move::PC_BISHOP | Move::PC_KNIGHT | Move::PC_ROOK | Move::PC_QUEEN => {
                    MGS[make_piece(state.side_to_play, moov.get_piece_type()) as usize][moov.to() as usize]
                        - MGS[piece as usize][moov.from() as usize]
                        - MGS[state.items[moov.to() as usize] as usize][moov.to() as usize]
                },
                Move::PR_BISHOP | Move::PR_KNIGHT | Move::PR_ROOK | Move::PR_QUEEN => {
                    MGS[make_piece(state.side_to_play, moov.get_piece_type()) as usize][moov.to() as usize]
                        - MGS[piece as usize][moov.from() as usize]
                },
                Move::CAPTURE => {
                    MGS[piece as usize][moov.to() as usize]
                        - MGS[piece as usize][moov.from() as usize]
                        - MGS[state.items[moov.to() as usize] as usize][moov.to() as usize]
                },
                Move::QUIET | Move::EN_PASSANT | Move::DOUBLE_PUSH | Move::OO | Move::OOO => {
                    MGS[piece as usize][moov.to() as usize]
                        - MGS[piece as usize][moov.from() as usize]
                },
                _ => unreachable!(),
            };

            let total_score = move_score + pieces_score * state.side_to_play.multiplicator() as i16;
            ScoredMove { moov: moov.clone(), score: total_score }
        }).collect();


        result
    }

    pub fn pick_next_best_move(&mut self, cur_index: usize){
        let size = self.moves.len();
        let mut max = i32::MIN;
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


        //         //         int max = Integer.MIN_VALUE;
        //         //         int maxIndex = -1;
        //         //         for (int i = curIndex; i < this.size(); i++){
        //         //             if (this.get(i).score() > max){
        //         //                 max = this.get(i).score();
        //         //                 maxIndex = i;
        //         //             }
        //         //         }
        //         //         Collections.swap(this, curIndex, maxIndex);
    }

    pub fn to_string(&self) -> String {
        let uci_moves = self.moves.iter().map(|m| m.uci()).collect::<Vec<String>>().join(" ");
        format!("[{}] {}", self.moves.len(), uci_moves)
    }

    pub fn over_sorted<'a>(&'a mut self, state: &'a BoardState, transposition_state: &'a TranspositionTable) -> SortedMovesIter<'a> {
        let scored_moves = self.score_moves(state, transposition_state);
        SortedMovesIter {
            state, // TODO check if it is really needed to keep the references
            transposition_state,
            move_list: scored_moves,
            index: 0,
        }
    }
}

pub struct ScoredMove {
    pub moov: Move,
    pub score: i16,
}

// pub struct IndexedMove<'a> {
//     moov: &'a Move,
//     index: usize,
// }
//
pub struct SortedMovesIter<'a> {
    state: &'a BoardState,
    transposition_state: &'a TranspositionTable,
    move_list: Vec<ScoredMove>,
    index: usize,
}

impl<'a> Iterator for SortedMovesIter<'a> {
    type Item = IndexedMove;
    fn next(&mut self) -> Option<IndexedMove> {
        if self.index == self.move_list.len() {
            return None;
        }

        self.move_list.pick_next_best_move(self.index);
        let moov: &Move = &self.move_list.moves[self.index];
        let result = Some(IndexedMove { moov: moov.clone(), index: 0 } );
        self.index += 1;
        result
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
