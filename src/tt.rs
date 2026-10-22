//! Shared transposition table. Lazy SMP threads access it without locks; with `lockless` the key word is
//! XOR-ed with the data word, so a torn write from another thread fails the key check.

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering::Relaxed};

use crate::moves::Move;

pub const EXACT: i32 = 0;
pub const LOWER_BOUND: i32 = 1;
pub const UPPER_BOUND: i32 = 2;
pub const EVAL_ONLY: i32 = 3;
pub const UNKNOWN_EVAL: i32 = i32::MIN;
pub const BYTES_PER_ENTRY: usize = 16;

// Each entry is two words: key bits 12..63 with flag+1 (bits 0..2) and generation (bits 3..10), then
// score (bits 40..63), move (24..39), static eval (8..23) and depth (0..7). Four entries share one bucket.
const KEY_MASK: u64 = !0xFFF;
const NO_EVAL: i64 = i16::MIN as i64;

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub score: i32,
    pub depth: i32,
    pub flag: i32,
    pub mv: Move,
    pub static_eval: i32,
}

pub struct TranspositionTable {
    table: Vec<AtomicU64>,
    count: usize,
    generation: AtomicU32,
    lockless: AtomicBool,
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new(16).expect("default hash size")
    }
}

impl TranspositionTable {
    pub fn new(mebibytes: usize) -> Result<Self, String> {
        if !(1..=512).contains(&mebibytes) {
            return Err("Hash must be 1..512 MiB".to_string());
        }
        let count = 1usize << (mebibytes * 1024 * 1024 / BYTES_PER_ENTRY).ilog2();
        let table = (0..count * 2).map(|_| AtomicU64::new(0)).collect();
        Ok(TranspositionTable { table, count, generation: AtomicU32::new(0), lockless: AtomicBool::new(false) })
    }

    pub fn capacity(&self) -> usize {
        self.count
    }

    pub fn is_lockless(&self) -> bool {
        self.lockless.load(Relaxed)
    }

    /// Switching modes clears the table, as existing key words would no longer verify.
    pub fn set_lockless(&self, lockless: bool) {
        if self.lockless.swap(lockless, Relaxed) != lockless {
            self.clear_slots();
        }
    }

    #[inline(always)]
    fn load(&self, slot: usize) -> u64 {
        // SAFETY: slots come from `bucket`, which stays below `count * 2 - 8`
        unsafe { self.table.get_unchecked(slot).load(Relaxed) }
    }

    #[inline(always)]
    fn store(&self, slot: usize, value: u64) {
        // SAFETY: as in `load`
        unsafe { self.table.get_unchecked(slot).store(value, Relaxed) }
    }

    #[inline(always)]
    fn key_word(&self, slot: usize, data: u64, lockless: bool) -> u64 {
        if lockless {
            self.load(slot) ^ data
        } else {
            self.load(slot)
        }
    }

    #[inline(always)]
    fn write(&self, slot: usize, key_word: u64, data: u64, lockless: bool) {
        self.store(slot + 1, data);
        self.store(slot, if lockless { key_word ^ data } else { key_word });
    }

    pub fn new_search(&self) {
        self.generation.fetch_add(1, Relaxed);
    }

    #[inline(always)]
    fn bucket(&self, key: u64) -> usize {
        ((key as u32 as usize) & (self.count - 4)) << 1
    }

    #[inline(always)]
    fn matches(stored: u64, key: u64) -> bool {
        stored != 0 && (stored ^ key) & KEY_MASK == 0
    }

    #[inline(always)]
    fn flag_of(stored: u64) -> i32 {
        (stored & 7) as i32 - 1
    }

    #[inline(always)]
    fn generation_of(stored: u64) -> u32 {
        ((stored >> 3) & 0xFF) as u32
    }

    #[inline(always)]
    fn depth_of(data: u64) -> i32 {
        (data & 0xFF) as i32
    }

    #[inline(always)]
    fn eval_of(data: u64) -> i32 {
        let eval = (data >> 8) as i16 as i64;
        if eval == NO_EVAL {
            UNKNOWN_EVAL
        } else {
            eval as i32
        }
    }

    #[inline(always)]
    fn with_eval(data: u64, eval: i32) -> u64 {
        let packed = if eval == UNKNOWN_EVAL { NO_EVAL } else { eval.clamp(-32767, 32767) as i64 };
        data & !0xFF_FF00 | ((packed as u64) & 0xFFFF) << 8
    }

    fn age(&self, stored: u64) -> u32 {
        self.generation.load(Relaxed).wrapping_sub(Self::generation_of(stored)) & 0xFF
    }

    pub fn set_static_eval(&self, key: u64, evaluation: i32) {
        let lockless = self.is_lockless();
        let base = self.bucket(key);
        for slot in (base..base + 8).step_by(2) {
            let data = self.load(slot + 1);
            let stored = self.key_word(slot, data, lockless);
            if Self::matches(stored, key) {
                self.write(slot, stored, Self::with_eval(data, evaluation), lockless);
                return;
            }
        }
        self.set(key, 0, 0, EVAL_ONLY, Move::NULL_MOVE, evaluation);
    }

    pub fn set(&self, key: u64, score: i32, depth: i32, flag: i32, best_move: Move, mut static_eval: i32) {
        let lockless = self.is_lockless();
        let base = self.bucket(key);
        let mut index = base;
        let mut weakest = i32::MAX;
        let current_generation = self.generation.load(Relaxed) & 0xFF;
        for slot in (base..base + 8).step_by(2) {
            let mut data = self.load(slot + 1);
            let stored = self.key_word(slot, data, lockless);
            if stored == 0 {
                index = slot;
                break;
            }
            if Self::matches(stored, key) {
                if static_eval != UNKNOWN_EVAL {
                    data = Self::with_eval(data, static_eval);
                    self.write(slot, stored, data, lockless);
                }
                if Self::generation_of(stored) == current_generation && depth + 2 < Self::depth_of(data) && flag != EXACT {
                    return;
                }
                if static_eval == UNKNOWN_EVAL {
                    static_eval = Self::eval_of(data);
                }
                index = slot;
                break;
            }
            let exact_bonus = if Self::flag_of(stored) == EXACT { 4 } else { 0 };
            let quality = Self::depth_of(data) + exact_bonus - self.age(stored).min(32) as i32 * 8;
            if quality < weakest {
                weakest = quality;
                index = slot;
            }
        }
        let previous = self.load(index + 1);
        let stored = self.key_word(index, previous, lockless);
        if stored != 0 && !Self::matches(stored, key) && Self::generation_of(stored) == current_generation && Self::depth_of(previous) > depth + 4 {
            return;
        }
        let data = ((score as i64) << 40) as u64 | (best_move.bits() as u64) << 24 | depth.clamp(0, 127) as u64;
        self.write(index, key & KEY_MASK | (current_generation as u64) << 3 | (flag + 1) as u64, Self::with_eval(data, static_eval), lockless);
    }

    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        let lockless = self.is_lockless();
        let base = self.bucket(key);
        for slot in (base..base + 8).step_by(2) {
            let data = self.load(slot + 1);
            let stored = self.key_word(slot, data, lockless);
            if Self::matches(stored, key) {
                return Some(TTEntry {
                    score: (data as i64 >> 40) as i32,
                    depth: Self::depth_of(data),
                    flag: Self::flag_of(stored),
                    mv: Move((data >> 24) as u16),
                    static_eval: Self::eval_of(data),
                });
            }
        }
        None
    }

    fn clear_slots(&self) {
        for slot in &self.table {
            slot.store(0, Relaxed);
        }
    }

    pub fn reset(&self) {
        self.clear_slots();
        self.generation.store(0, Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_and_probes() {
        for lockless in [false, true] {
            let table = TranspositionTable::new(1).unwrap();
            table.set_lockless(lockless);
            let key = 0x1234_5678_9ABC_DEF0u64;
            let mv = Move::new(12, 28, Move::DOUBLE_PUSH);
            table.set(key, -345, 7, LOWER_BOUND, mv, 42);
            let entry = table.probe(key).unwrap();
            assert_eq!((entry.score, entry.depth, entry.flag, entry.mv, entry.static_eval), (-345, 7, LOWER_BOUND, mv, 42));
            table.set_static_eval(key, -17);
            assert_eq!(table.probe(key).unwrap().static_eval, -17);
            assert!(table.probe(key ^ 1 << 40).is_none());
        }
    }
}
