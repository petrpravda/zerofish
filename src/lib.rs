//! zerofish: a chess engine, rebuilt one feature at a time.

// Search code comes from the final engine with later features removed; their scaffolding stays.
#![allow(unused)]

pub mod bitboard;
pub mod board;
pub mod evaluation;
pub mod fen;
pub mod limits;
pub mod moves;
pub mod ordering;
pub mod perft;
pub mod position;
pub mod pst;
pub mod random;
pub mod search;
pub mod see;
pub mod tt;
pub mod types;
pub mod zobrist;
