// extern crate core;

extern crate core;

pub mod uci;
pub mod engine;
pub mod engine_thread;
pub mod fen;
pub mod bitboard;
pub mod piece;
pub mod side;
pub mod square;
pub mod r#move;
pub mod board_state;
mod perft;
pub mod zobrist;
pub mod piece_square_table;
pub mod search;
pub mod board_position;
pub mod time;
mod transposition;
mod statistics;
mod evaluation;
pub mod util;
pub mod pgn;


pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
