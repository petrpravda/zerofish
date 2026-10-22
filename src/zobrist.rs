use std::sync::LazyLock;

use crate::random::JavaRandom;

pub struct Zobrist {
    pub pieces: [[u64; 64]; 14],
    pub en_passant: [u64; 8],
    pub castling: [u64; 4],
    pub chess960_castling: [[u64; 8]; 4],
    pub side: u64,
}

/// Same seed and draw order as the Java engine, so position keys are identical.
pub static ZOBRIST: LazyLock<Zobrist> = LazyLock::new(|| {
    let mut random = JavaRandom::new(0x4a61_7661_6669_7368);
    let side = random.next_long() as u64;
    let mut pieces = [[0u64; 64]; 14];
    for row in pieces.iter_mut() {
        for key in row.iter_mut() {
            *key = random.next_long() as u64;
        }
    }
    let mut en_passant = [0u64; 8];
    for key in en_passant.iter_mut() {
        *key = random.next_long() as u64;
    }
    let mut castling = [0u64; 4];
    for key in castling.iter_mut() {
        *key = random.next_long() as u64;
    }
    let mut chess960_castling = [[0u64; 8]; 4];
    for row in chess960_castling.iter_mut() {
        for key in row.iter_mut() {
            *key = random.next_long() as u64;
        }
    }
    Zobrist { pieces, en_passant, castling, chess960_castling, side }
});
