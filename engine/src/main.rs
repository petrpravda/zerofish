mod uci;
mod engine;
mod engine_thread;
mod fen;
mod bitboard;
mod board_state;
mod piece;
mod side;
mod square;

fn main() {
    uci::start_uci_loop(&engine_thread::spawn_engine_thread());
}
