use crate::square::Square;

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

const MULTIPLIER: u64 = 0x356f678ebfe40a00;
const INCREMENT: u64 = 0x8d811b3c80fbe984;

pub struct RandomGenerator {
    state: u64,
}

impl RandomGenerator {
    pub fn new() -> Self {
        RandomGenerator::new_with_seed(0xc012a570e622f831)
    }

    pub fn new_with_seed(seed: u64) -> Self {
        RandomGenerator {
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

    pub fn rand64(&mut self) -> u64 {
        ((self.rand32() as u64) << 32) | (self.rand32() as u64)
    }
}


