use lazy_static::lazy_static;
use crate::square::Square;

lazy_static! {
    pub static ref ZOBRIST: Zobrist = Zobrist::new();
}

pub struct Zobrist {
    pub pieces: [[u64; 64]; 14],   // TODO compare access performance with flat structure
    pub en_passant: [u64; 8],
    pub side: u64,
}

impl Zobrist {
    pub fn new() -> Self {
        let mut rng = RandomGenerator::new();

        let mut pieces = [[0u64; 64]; 14];
        let mut en_passant = [0u64; 8];

        for piece in 0..14 {
            for sq in Square::A1 as usize..=Square::H8 as usize {
                pieces[piece][sq] = rng.rand64();
            }
        }

        for file in 0..=7 {
            en_passant[file] = rng.rand64();
        }

        Self {
            pieces,
            en_passant,
            side: rng.rand64()
        }
    }
}

const MULTIPLIER: u64 = 0x5851f42d4c957f2d;
const INCREMENT: u64 = 0x14057b7ef767814f;

pub struct RandomGenerator {
    state: u64,
}

impl RandomGenerator {
    pub fn new() -> Self {
        RandomGenerator::new_with_seed(0xc012a570e622f831)
    }

    pub fn new_with_seed(seed: u64) -> Self {
        Self {
            state: seed.wrapping_mul(seed)
        }
    }

    pub fn rand32(&mut self) -> u32 {
        let mut x = self.state;
        let count = (x >> 59) as u32;
        self.state = x.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
        x ^= x >> 18;

        ((x >> 27) as u32).rotate_right(count)
    }

    #[inline]
    pub fn rand64(&mut self) -> u64 {
        ((self.rand32() as u64) << 32) | (self.rand32() as u64)
    }

    #[inline]
    pub fn rand128(&mut self) -> u128 {
        ((self.rand64() as u128) << 64) | (self.rand64() as u128)
    }

}


#[cfg(test)]
mod tests {
    use crate::fen::{from_fen_default, START_POS};
    use crate::piece::{BLACK_ROOK, WHITE_ROOK};
    use crate::r#move::Move;
    use crate::zobrist::ZOBRIST;

    #[test]
    fn hashing_changes() {
        let mut state = from_fen_default(START_POS);
        let hash_starting = state.hash;
        state = state.do_move(&Move::from_uci_string("b1a3", &state));
        let hash_1 = state.hash;
        assert_ne!(hash_starting, hash_1);
        state = state.do_move(&Move::from_uci_string("d7d5", &state));
        let hash_2 = state.hash;
        assert_ne!(hash_1, hash_2);
    }

    #[test]
    fn unique_hashes() {
        let zobrist_pieces = &ZOBRIST.pieces;
        assert_ne!(zobrist_pieces[8][51], zobrist_pieces[8][35]);
    }

    // TODO check uniqueness of random numbers in pieces, distribution???
}
