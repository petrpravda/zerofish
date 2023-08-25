use crate::board_state::BoardState;
use crate::fen::{Fen, FenExport, START_POS};
use crate::r#move::Move;

#[derive(Debug, Clone)]
pub struct BoardPosition {
    pub state: BoardState,
    pub history: Vec<u16>,
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
            //history_index: 0,
            history: vec![],
        }
    }

    pub fn do_move(&mut self, moov: &Move) -> &BoardState {
        // if self.history.len() == 20 {
        //     println!("oifu");
        // }
        self.state = self.state.do_move(moov);
        //self.history_index += 1;
        self.history.push(moov.bits);
        self.state.ply = 0;
        return &self.state;
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

#[cfg(test)]
mod tests {
    use crate::board_position::BoardPosition;
    use crate::board_state::BoardState;
    use crate::piece::{PieceType};
    use crate::r#move::Move;
    use crate::side::Side;

    #[test]
    fn from_cute_chess_moves() {
        let position_before = BoardPosition::from_moves(&"d2d4 e7e5");
        let state = position_before.state;
        println!("{}", BoardState::bitboard_string(state.bitboard_of(Side::BLACK, PieceType::PAWN)));

        let moov = Move::from_uci_string("d4e5", &state);
        let new_state = state.do_move(&moov);
        println!("{}", BoardState::bitboard_string(new_state.bitboard_of(Side::BLACK, PieceType::PAWN)));
    }
}

