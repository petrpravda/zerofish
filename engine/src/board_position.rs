// use std::borrow::{Borrow, BorrowMut};
// use std::result;
//
// use crate::bitboard::Bitboard;
// use crate::board_state::BoardState;
// use crate::fen::from_fen_default;
// use crate::r#move::Move;

use crate::board_state::BoardState;
use crate::fen::{Fen, FenExport, START_POS};
use crate::r#move::Move;
use crate::transposition::Depth;

#[derive(Debug)]
pub struct BoardPosition {
    pub state: BoardState,
    pub history_index: usize,
    pub history: Vec<u16>,
    //public long[] history = new long[MAX_GAME_HISTORY_DEPTH];

}

impl FenExport for BoardPosition {
    fn to_fen(&self) -> String {
        self.state.to_fen()
    }
}

impl BoardPosition {

    pub fn from_fen(fen: &str) -> BoardPosition {
        BoardPosition {
            state: Fen::from_fen_default(fen),
            history_index: 0,
            history: vec![],
        }
    }

    pub fn do_move(&mut self, moov: &Move) -> &BoardState {
        self.state = self.state.do_move(moov);
        self.history_index += 1;
        self.history.push(moov.bits);
        self.state.ply = 0;
        return &self.state;
    }

    //     public BoardState doMove(String uciMove) {
    //         Move move = this.state.generateLegalMoves().stream().filter(m -> m.toString().equals(uciMove)).findFirst()
    //                 .orElseThrow();
    //         return doMove(move);
    //     }
    //
    pub fn for_search_depth(&self, search_depth: Depth) -> BoardPosition {
        BoardPosition {
            state: self.state.for_search_depth(search_depth),
            history_index: self.history_index,
            history: self.history.clone(),
        }
    }

    pub fn from_moves(moves_string: &str) -> Self {
        let mut position = BoardPosition::from_fen(START_POS);
        let moves: Vec<&str> = moves_string.split_whitespace().collect();
        let parsed_moves = position.state.parse_moves(&moves);
        for moov in parsed_moves {
            position.do_move(&moov);
        }
        position
    }
}

// position startpos moves d2d4 e7e5 d4e5 f7f6 c1f4

#[cfg(test)]
mod tests {
    use crate::board_position::BoardPosition;
    use crate::board_state::BoardState;
    use crate::piece::{PieceType};
    use crate::r#move::Move;
    use crate::side::Side;

    #[test]
    fn from_cute_chess_moves() {
        // let from = Square::get_square_from_name("e5"); // 36
        // let to = Square::get_square_from_name("f4");   // 29
        let position_before = BoardPosition::from_moves(&"d2d4 e7e5");
        let state = position_before.state;
        println!("{}", BoardState::bitboard_string(state.bitboard_of(Side::BLACK, PieceType::PAWN)));

        let moov = Move::from_uci_string("d4e5", &state);
        let new_state = state.do_move(&moov);
        println!("{}", BoardState::bitboard_string(new_state.bitboard_of(Side::BLACK, PieceType::PAWN)));

        // let position = BoardPosition::from_moves(&"d2d4 e7e5 d4e5 f7f6 c1f4");
        // println!("{}", position.state.to_fen());
        // let bb = position.state.bitboard_of(Side::BLACK, PieceType::PAWN);
        // println!("{}", BoardState::to_bitboard_string(bb));
        // let moves = position.state.generate_legal_moves();
        // println!("{}", moves);
        // assert_eq!(state.to_string(), );
    }
}

