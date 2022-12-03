use std::sync::atomic::{AtomicU64, Ordering};
use crate::board_state::BoardState;
use crate::r#move::Move;
use crate::search::Bound;

pub type Depth = u8;
pub type Hash = u64;
pub type BaseMove = u16;
// pub type MoveInt = u16;
// pub type Ply = usize;
// pub type SortValue = i16;
// pub type Time = u64;
// pub type ScoreValueTT = i16;
pub type Value = i32;

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct TTEntry {
    value: Value,
    best_move: BaseMove,
    depth: Depth,
    flag: Bound,
}

impl TTEntry {
    pub fn new(value: Value, best_move: BaseMove, depth: Depth, flag: Bound) -> Self {
        TTEntry {
            best_move,
            depth,
            value,
            flag,
        }
    }

    #[inline(always)]
    pub fn best_move(&self) -> BaseMove {
        self.best_move
    }

    #[inline(always)]
    pub fn depth(&self) -> Depth {
        self.depth
    }

    #[inline(always)]
    pub fn value(&self) -> Value {
        self.value
    }

    #[inline(always)]
    pub fn flag(&self) -> Bound {
        self.flag
    }
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            best_move: Move::BM_NULL,
            depth: 0,
            value: 0,
            flag: Bound::Exact,
        }
    }
}

impl From<Hash> for TTEntry {
    fn from(value: Hash) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

impl From<TTEntry> for Hash {
    fn from(value: TTEntry) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

///////////////////////////////////////////////////////////////////
// Transposition Table
///////////////////////////////////////////////////////////////////

pub struct TranspositionTable {
    table: Vec<AtomicEntry>,
    bitmask: Hash,
}

impl TranspositionTable {
    pub fn new(mb_size: usize) -> Self {
        assert_eq!(std::mem::size_of::<TTEntry>(), 8);
        // let upper_limit = mb_size * 1024 * 1024 / std::mem::size_of::<AtomicEntry>() + 1;
        //let count = upper_limit.next_power_of_two() / 2;
        let count = 1048576;
        let mut table = Vec::with_capacity(count);

        for _ in 0..count {
            table.push(AtomicEntry::default());
        }

        TranspositionTable {
            table,
            bitmask: count as Hash - 1,
        }
    }

    #[inline(always)]
    pub fn insert(
        &self,
        state: &BoardState,  // TODO pass hash instead
        depth: Depth,
        value: Value,
        best_move: BaseMove,
        flag: Bound,
    ) {
        self.table[self.index(state)]
            .write(state.hash, TTEntry::new(value, best_move, depth, flag))
    }

    #[inline(always)]
    pub fn probe(&self, state: &BoardState) -> Option<TTEntry> { // TODO pash hash
        self.table[self.index(state)].read(state.hash)
    }

    pub fn clear(&mut self) {
        self.table
            .iter_mut()
            .for_each(|entry| *entry = AtomicEntry::default());
    }

    #[inline(always)]
    fn index(&self, state: &BoardState) -> usize {
        (state.hash & self.bitmask) as usize
    }

    pub fn mb_size(&self) -> usize {
        self.table.len() * std::mem::size_of::<AtomicEntry>() / 1024 / 1024
    }

    pub fn hashfull(&self) -> usize {
        // Sample the first 1000 entries to estimate how full the table is.
        self.table
            .iter()
            .take(1000)
            .filter(|&entry| entry.used())
            .count()
    }
}

#[derive(Default)]
struct AtomicEntry(AtomicU64, AtomicU64);

impl AtomicEntry {
    fn read(&self, lookup_hash: Hash) -> Option<TTEntry> {
        let entry_hash = self.0.load(Ordering::Relaxed);
        let data = self.1.load(Ordering::Relaxed);
        if entry_hash ^ data == lookup_hash {
            return Some(TTEntry::from(data));
        }
        None
    }

    fn write(&self, hash: Hash, entry: TTEntry) {
        let data = Hash::from(entry);
        self.0.store(hash ^ data, Ordering::Relaxed);
        self.1.store(data, Ordering::Relaxed);
    }

    fn used(&self) -> bool {
        self.0.load(Ordering::Relaxed) != Hash::default()
    }
}


// public class TranspTable {
//     private final static HashMap<Long, TTEntry> table = new HashMap<>();
//
//     public static void set(long key, int score, int depth, int flag, Move bestMove){
//         table.put(key, new TTEntry(score, depth, flag, bestMove));
//     }
//
//     public static TTEntry probe(long key){
//         return table.get(key);
//     }
//
//     public static void reset(){
//         table.clear();
//     }
// }

// public class TTEntry {
//     public final static byte EXACT = 0, LOWER_BOUND = 1, UPPER_BOUND = 2;
//     public final static int SIZE = 10; // in bytes
//
//     private final int score;
//     private final byte depth, flag;
//     private final int bestMove;
//
//     public TTEntry(int score, int depth, int flag, Move bestMove){
//         this.score = score;
//         this.depth = (byte)depth;
//         this.flag = (byte)flag;
//         this.bestMove = bestMove.bits();
//     }
//
//     public int score(){
//         return score;
//     }
//
//     public int depth(){
//         return depth;
//     }
//
//     public int flag(){
//         return flag;
//     }
//
//     public Move move(){
//         return new Move(bestMove);
//     }
//
// }
