//! Static evaluation correction from pawn-structure and position keyed histories.

use std::sync::atomic::{AtomicI32, Ordering::Relaxed};

use crate::board::BoardState;
use crate::search::is_score_checkmate;
use crate::types::PAWN;

const SIZE: usize = 16384;

pub struct CorrectionHistory {
    pawn: Vec<AtomicI32>,
    placement: Vec<AtomicI32>,
}

impl Default for CorrectionHistory {
    fn default() -> Self {
        Self::new()
    }
}

fn index(mut key: u64) -> usize {
    key ^= key >> 33;
    key = key.wrapping_mul(0xff51_afd7_ed55_8ccd);
    key ^= key >> 33;
    key as usize & (SIZE - 1)
}

fn pawn_index(state: &BoardState) -> usize {
    index(state.bitboard_of_side(0, PAWN) ^ state.bitboard_of_side(1, PAWN).rotate_left(23))
}

impl CorrectionHistory {
    pub fn new() -> Self {
        let table = || (0..2 * SIZE).map(|_| AtomicI32::new(0)).collect();
        CorrectionHistory { pawn: table(), placement: table() }
    }

    pub fn reset(&self) {
        for value in self.pawn.iter().chain(&self.placement) {
            value.store(0, Relaxed);
        }
    }

    pub fn corrected(&self, state: &BoardState, raw: i32) -> i32 {
        let side = state.side_to_play() * SIZE;
        let adjustment = (self.pawn[side + pawn_index(state)].load(Relaxed) + self.placement[side + index(state.hash())].load(Relaxed)) / 1024;
        raw + adjustment.clamp(-128, 128)
    }

    pub fn update(&self, state: &BoardState, depth: i32, raw: i32, score: i32) {
        if is_score_checkmate(raw) || is_score_checkmate(score) {
            return;
        }
        let target = (score - raw).clamp(-256, 256) * 256;
        let weight = (depth + 1).clamp(1, 16);
        let side = state.side_to_play() * SIZE;
        for entry in [&self.pawn[side + pawn_index(state)], &self.placement[side + index(state.hash())]] {
            entry.store((entry.load(Relaxed) * (256 - weight) + target * weight) / 256, Relaxed);
        }
    }
}
