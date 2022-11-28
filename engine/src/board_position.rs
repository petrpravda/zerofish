// use std::borrow::{Borrow, BorrowMut};
// use std::result;
//
// use crate::bitboard::Bitboard;
// use crate::board_state::BoardState;
// use crate::fen::from_fen_default;
// use crate::r#move::Move;

use crate::board_state::BoardState;
use crate::fen::from_fen_default;
use crate::r#move::Move;

pub struct BoardPosition {
    pub(crate) state: BoardState,
    historyIndex: usize,
    history: Vec<u32>,
    //public long[] history = new long[MAX_GAME_HISTORY_DEPTH];

}

impl BoardPosition {

    pub fn from_fen(fen: &str) -> BoardPosition {
        let result = BoardPosition {
            state: from_fen_default(fen),
            historyIndex: 0,
            history: vec![],
        };
        result
    }

    pub fn do_move(&mut self, moov: &Move) -> &BoardState {
        self.state = self.state.do_move(moov);
        self.historyIndex += 1;
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
    pub fn for_search_depth(&self, search_depth: u16) -> BoardPosition {
        BoardPosition {
            state: self.state.for_search_depth(search_depth),
            historyIndex: self.historyIndex,
            history: self.history.clone(),
        }
    }
}
