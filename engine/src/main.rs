// use zerofish::bitboard::Bitboard;
// use zerofish::fen::{from_fen_default, START_POS};
use zerofish::{engine_thread, uci};
//use zerofish::piece_square_table::MGS;

// mod uci;
// mod engine;
// mod engine_thread;
// mod fen;
// mod bitboard;
// mod piece;
// mod side;
// mod square;
// mod r#move;
// mod lib;
// mod board_state;
// mod board_state;
//use zerofish::{engine_thread, uci};

// #[derive(Debug)]
// pub struct MyMove {
//     id: u32,
//     code: String,
// }
//
// impl MyMove {
//     fn new(code: &str, id: u32) -> Self {
//         Self {
//             code: code.to_string(),
//             id
//         }
//     }
// }
//
// pub struct SortedMovesIter<'a> {
//     move_list: Vec<MyMove>,
//     index: usize,
// }
//
// impl<'a> Iterator for SortedMovesIter<'a> {
//     type Item = &'a MyMove;
//
//     fn next(&mut self) -> Option<&'a MyMove> {
//         if self.index == self.move_list.len() {
//             return None;
//         }
//         let item = &self.move_list[self.index];
//         self.index += 1;
//         Some(item)
//     }
// }

fn main() {
    // let moves = vec![
    //     MyMove::new("d2d4", 0),
    //     MyMove::new("e7e5", 1),
    //     MyMove::new("d4e5", 2)];
    //
    // for moov in (SortedMovesIter { move_list: moves.to_vec(), index: 0 }) {
    //     println!("{:?}", moov);
    // }
    uci::start_uci_loop(&engine_thread::spawn_engine_thread());

    //println!("{}", MGS[1][3]);
    //uci::start_uci_loop(&engine_thread::spawn_engine_thread());
    // let bitboard = Bitboard::new();
    // let mut state = from_fen_default(START_POS, &bitboard);
    // let moves = state.generate_legal_moves();
    // println!("{}", moves);
}
