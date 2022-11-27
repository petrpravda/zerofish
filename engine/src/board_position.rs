// use std::borrow::{Borrow, BorrowMut};
// use std::result;
//
// use crate::bitboard::Bitboard;
// use crate::board_state::BoardState;
// use crate::fen::from_fen_default;
// use crate::r#move::Move;

// pub struct BoardPosition {
//     state: <&'a>,
//     historyIndex: usize,
//     history: Vec<u32>,
//     //public long[] history = new long[MAX_GAME_HISTORY_DEPTH];
//
// }
//
// impl BoardPosition {
//
//     pub fn from_fen<'b>(fen: &str, bitboard: &'b Bitboard) -> BoardPosition<'b> {
//         let result = BoardPosition {
//             state: from_fen_default(fen, bitboard),
//             historyIndex: 0,
//             history: vec![],
//         };
//         result
//     }
//
//     pub fn do_move(&mut self, moov: &Move) -> &'a BoardState {
//         let bbb = self.state.do_move(moov);
//         self.state = (move || bbb)();
//         self.historyIndex += 1;
//         self.history.push(moov.bits);
//         // this.state.ply = 0;
//         return &self.state;
//     }
//
//     //     public BoardState doMove(Move move) {
//     //         this.state = this.state.doMove(move);
//     //         this.history[this.historyIndex++] = move.bits();
//     //         this.state.ply = 0;
//     //         return this.state;
//     //     }
//     //
//     //     public BoardState doMove(String uciMove) {
//     //         Move move = this.state.generateLegalMoves().stream().filter(m -> m.toString().equals(uciMove)).findFirst()
//     //                 .orElseThrow();
//     //         return doMove(move);
//     //     }
//     //
//     //     public BoardPosition forSearchDepth(int searchDepth) {
//     //         BoardPosition result = new BoardPosition();
//     //         result.state = this.state.forSearchDepth(searchDepth);
//     //         result.historyIndex = this.historyIndex;
//     //         result.history = this.history.clone();
//     //         return result;
//     //     }
// }
