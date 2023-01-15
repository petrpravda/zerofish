extern crate core;

use env::args;
use std::{env};
use zerofish::{engine_thread, uci};
use zerofish::engine::{EngineOptions};
use zerofish::util::extract_parameter;

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
    let args_strings = args().collect::<Vec<String>>();
    let args: Vec<&str> = args_strings.iter().map(|s| s.as_ref()).collect();
    let log_filename = extract_parameter(&args, "--log");
    let engine_options = EngineOptions { log_filename };
    env::set_var("RUST_BACKTRACE", "full");
    uci::start_uci_loop(&engine_thread::spawn_engine_thread(&engine_options).0);



    // let mut engine = Engine::new(EngineOptions { log_filename:  None});
    // engine.do_pgn_moves("d4 Nf6 c4 e6 Nc3 Bb4 e3 O-O Bd3 c5");
    // println!("{}", engine.position.to_fen());

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
