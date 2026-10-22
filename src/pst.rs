//! Piece-square tables of the classical evaluation, indexed by piece and square.

use std::sync::LazyLock;

pub const BASIC_MATERIAL_VALUE: [i32; 5] = [1, 3, 3, 5, 9];

pub struct PieceSquareTables {
    pub mgs: [[i32; 64]; 16],
    pub egs: [[i32; 64]; 16],
}

pub static PST: LazyLock<PieceSquareTables> = LazyLock::new(|| parse(include_str!("../resources/piece.square.table.txt")));

const NAMES: [&str; 6] = ["PAWN", "KNIGHT", "BISHOP", "ROOK", "QUEEN", "KING"];

/// Each block is `NAME(n)` followed by 8 rows (8th rank first) of 8 middlegame then 8 endgame values, h-file first.
fn parse(text: &str) -> PieceSquareTables {
    let mut tables = PieceSquareTables { mgs: [[0; 64]; 16], egs: [[0; 64]; 16] };
    let mut tokens = text.split_whitespace().peekable();
    while let Some(token) = tokens.next() {
        let name = token.split('(').next().unwrap_or("");
        let piece = NAMES.iter().position(|n| *n == name).unwrap_or_else(|| panic!("Unknown piece table {token}"));
        for rank in (0..8).rev() {
            for file in (0..8).rev() {
                let value: i32 = tokens.next().and_then(|t| t.parse().ok()).expect("piece square value");
                tables.mgs[piece][rank * 8 + file] = value;
                tables.mgs[piece + 8][(7 - rank) * 8 + file] = -value;
            }
            for file in (0..8).rev() {
                let value: i32 = tokens.next().and_then(|t| t.parse().ok()).expect("piece square value");
                tables.egs[piece][rank * 8 + file] = value;
                tables.egs[piece + 8][(7 - rank) * 8 + file] = -value;
            }
        }
    }
    tables
}
