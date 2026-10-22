/// `java.util.Random`, so magic numbers and Zobrist keys match the Java engine bit for bit.
pub struct JavaRandom {
    seed: u64,
}

const MULTIPLIER: u64 = 0x5DEE_CE66D;
const MASK: u64 = (1 << 48) - 1;

impl JavaRandom {
    pub fn new(seed: i64) -> Self {
        JavaRandom { seed: (seed as u64 ^ MULTIPLIER) & MASK }
    }

    fn next(&mut self, bits: u32) -> i32 {
        self.seed = (self.seed.wrapping_mul(MULTIPLIER).wrapping_add(0xB)) & MASK;
        (self.seed >> (48 - bits)) as i32
    }

    pub fn next_long(&mut self) -> i64 {
        ((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64)
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        let mut r = self.next(31);
        let m = bound - 1;
        if bound & m == 0 {
            return ((bound as i64 * r as i64) >> 31) as i32;
        }
        let mut u = r;
        loop {
            r = u % bound;
            if u.wrapping_sub(r).wrapping_add(m) >= 0 {
                return r;
            }
            u = self.next(31);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_java_sequence() {
        // new java.util.Random(42).nextLong() / nextInt(100)
        let mut random = JavaRandom::new(42);
        assert_eq!(random.next_long(), -5025562857975149833);
        let mut random = JavaRandom::new(42);
        assert_eq!(random.next_int(100), 30);
        assert_eq!(random.next_int(100), 63);
        assert_eq!(random.next_int(7), 6);
    }
}
