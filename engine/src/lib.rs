pub mod uci;
pub mod engine;
pub mod engine_thread;
pub mod fen;
pub mod bitboard;
mod piece;
mod side;
mod square;
mod r#move;
pub mod board_state;
mod perft;


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
