use zerofish::fen::from_fen_default;

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
    //uci::start_uci_loop(&engine_thread::spawn_engine_thread());
    let mut state = from_fen_default("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let moves = state.generate_legal_moves(false);
    println!("{}", moves);

}
