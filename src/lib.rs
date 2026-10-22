//! zerofish: a UCI chess engine with magic bitboards, NNUE evaluation and Lazy SMP search.

pub mod bitboard;
pub mod board;
pub mod correction;
pub mod evaluation;
pub mod fen;
pub mod limits;
pub mod moves;
#[cfg(not(feature = "stockfish-nnue"))]
pub mod nnue;
#[cfg(feature = "stockfish-nnue")]
#[path = "stockfish_nnue.rs"]
pub mod nnue;
pub mod ordering;
pub mod parallel;
pub mod perft;
pub mod pgn;
pub mod position;
pub mod pst;
pub mod random;
pub mod search;
pub mod see;
pub mod tt;
pub mod types;
pub mod uci;
pub mod zobrist;
