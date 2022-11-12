mod uci;
mod engine;
mod engine_thread;
mod fen;
mod bitboard;

fn main() {
    uci::start_uci_loop(&engine_thread::spawn_engine_thread());
}
