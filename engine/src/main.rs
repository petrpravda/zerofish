extern crate core;

use std::{env, panic};
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


fn main() {
    let result = panic::catch_unwind(|| {
        env::set_var("RUST_BACKTRACE", "1");
        uci::start_uci_loop(&engine_thread::spawn_engine_thread().0);
    });

    match result {
        Ok(_) => {},
        Err(e) => println!("The code panicked with: {:?}", e),
    }


    // let mut search = Search::new();     //println!("{}", MGS[1][3]);
    // let position = BoardPosition::from_fen(START_POS);
    // let result = search.it_deep(&position, 10);
    // println!("{:?}", result);
    // uci::start_uci_loop(&engine_thread::spawn_engine_thread());
    // let bitboard = Bitboard::new();
    // let mut state = from_fen_default(START_POS, &bitboard);
    // let moves = state.generate_legal_moves();
    // println!("{}", moves);
}
