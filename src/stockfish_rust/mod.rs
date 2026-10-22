// SPDX-License-Identifier: GPL-3.0-or-later
//! Direct Rust Stockfish NNUE: encoders, network reader, accumulator stack and SIMD.
//! No C/C++ calls or shared-library dependencies. See native/UPSTREAM.json for provenance.
mod features;
mod network;
mod simd;
use crate::{board::BoardState, moves::Move, search::MAX_PLY};
pub use network::HIDDEN;
use network::{Accumulator, NETWORK};
pub use simd::Kernel;
pub const SHA256: &str = "c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73";
pub const RULE50_DENOMINATOR: i32 = 189;
pub fn init() {
    std::sync::LazyLock::force(&NETWORK);
}

#[repr(align(64))]
struct Frame {
    accumulator: [Accumulator; 2],
    key: Option<u64>,
}
impl Frame {
    fn new() -> Self {
        Self {
            accumulator: [[0; HIDDEN]; 2],
            key: None,
        }
    }
}
pub struct Nnue {
    frames: Vec<Frame>,
    kernel: Kernel,
    old_features: [Vec<u32>; 2],
    new_features: [Vec<u32>; 2],
    removed: Vec<u32>,
    added: Vec<u32>,
}
impl Default for Nnue {
    fn default() -> Self {
        Self::new()
    }
}
impl Nnue {
    pub fn new() -> Self {
        Self::with_kernel(Kernel::best())
    }
    pub fn with_kernel(kernel: Kernel) -> Self {
        assert!(
            kernel != Kernel::Avx512Vnni || Kernel::avx512_available(),
            "AVX-512BW/VNNI unavailable on this host"
        );
        init();
        Self {
            frames: (0..MAX_PLY + 2).map(|_| Frame::new()).collect(),
            kernel,
            old_features: std::array::from_fn(|_| Vec::with_capacity(512)),
            new_features: std::array::from_fn(|_| Vec::with_capacity(512)),
            removed: Vec::with_capacity(512),
            added: Vec::with_capacity(512),
        }
    }
    pub fn kernel(&self) -> Kernel {
        self.kernel
    }
    pub fn reset(&mut self, state: &BoardState) {
        for frame in &mut self.frames {
            frame.key = None;
        }
        self.refresh(state, 0);
    }
    pub fn refresh(&mut self, state: &BoardState, ply: usize) {
        let network = &*NETWORK;
        let frame = &mut self.frames[ply];
        features::active(state, &mut self.new_features);
        for perspective in 0..2 {
            frame.accumulator[perspective] = network.biases;
            self.kernel.update(
                &mut frame.accumulator[perspective],
                &network.weights,
                &[],
                &self.new_features[perspective],
            );
        }
        frame.key = Some(state.hash());
    }
    fn ensure(&mut self, state: &BoardState, ply: usize) {
        if self.frames[ply].key != Some(state.hash()) {
            self.refresh(state, ply);
        }
    }
    pub fn push(&mut self, before: &BoardState, after: &BoardState, ply: usize) {
        self.ensure(before, ply - 1);
        let (parents, children) = self.frames.split_at_mut(ply);
        let parent = &parents[ply - 1];
        let child = &mut children[0];
        child.accumulator = parent.accumulator;
        features::changed(
            before,
            after,
            &mut self.old_features,
            &mut self.new_features,
        );
        for perspective in 0..2 {
            self.removed.clear();
            self.added.clear();
            let old = &self.old_features[perspective];
            let new = &self.new_features[perspective];
            let (mut i, mut j) = (0, 0);
            while i < old.len() && j < new.len() {
                match old[i].cmp(&new[j]) {
                    std::cmp::Ordering::Less => {
                        self.removed.push(old[i]);
                        i += 1;
                    }
                    std::cmp::Ordering::Greater => {
                        self.added.push(new[j]);
                        j += 1;
                    }
                    std::cmp::Ordering::Equal => {
                        i += 1;
                        j += 1;
                    }
                }
            }
            self.removed.extend_from_slice(&old[i..]);
            self.added.extend_from_slice(&new[j..]);
            self.kernel.update(
                &mut child.accumulator[perspective],
                &NETWORK.weights,
                &self.removed,
                &self.added,
            );
        }
        child.key = Some(after.hash());
    }
    pub fn push_move(&mut self, before: &BoardState, after: &BoardState, _mv: Move, ply: usize) {
        self.push(before, after, ply);
    }
    pub fn accumulator(&mut self, state: &BoardState, ply: usize, perspective: usize) -> Vec<i16> {
        self.ensure(state, ply);
        self.frames[ply].accumulator[perspective].to_vec()
    }
    pub fn raw(&mut self, state: &BoardState, ply: usize) -> i32 {
        self.ensure(state, ply);
        NETWORK.forward(state, &self.frames[ply].accumulator, self.kernel)
    }
    pub fn full_internal(&mut self, state: &BoardState, ply: usize) -> i32 {
        let raw = self.raw(state, ply);
        network::scaled(state, raw, state.half_move_clock)
    }
    pub fn evaluate_position(&mut self, state: &BoardState, ply: usize) -> i32 {
        let raw = self.raw(state, ply);
        network::centipawns(state, network::scaled(state, raw, 0))
    }
    pub fn evaluate(&mut self, state: &BoardState, ply: usize) -> i32 {
        let score = self.evaluate_position(state, ply);
        Self::clocked(state, score)
    }
    pub fn clocked(state: &BoardState, score: i32) -> i32 {
        score - score * state.half_move_clock / RULE50_DENOMINATOR
    }
}

#[cfg(test)]
mod tests;
