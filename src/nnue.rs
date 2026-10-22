//! Zerofish 1024-hidden, 8 king-bucket NNUE (768 inputs per perspective, SCReLU output).

use std::sync::LazyLock;

use crate::board::BoardState;
use crate::moves::Move;
use crate::search::MAX_PLY;
use crate::types::*;

pub const SHA256: &str = "85959e702a2147cceaf6829a6933211865e2c271d434f394ac46063b90f7a6b3";
pub const HIDDEN: usize = 1024;
const FEATURES: usize = 768;
const NETWORK_BYTES: usize = 12_589_120;
const BUCKETS: [usize; 64] = [
    0, 1, 2, 3, 3, 2, 1, 0, 4, 4, 5, 5, 5, 5, 4, 4, //
    6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, //
    6, 6, 6, 6, 6, 6, 6, 6, 7, 7, 7, 7, 7, 7, 7, 7, //
    7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7,
];

type Accumulator = [i16; HIDDEN];

struct Network {
    weights: Vec<i16>,
    biases: Box<Accumulator>,
    output: Vec<i16>,
    output_bias: i16,
}

static NETWORK: LazyLock<Network> = LazyLock::new(|| {
    let bytes: &[u8] = include_bytes!("../resources/weights-original.nnue");
    assert_eq!(bytes.len(), NETWORK_BYTES, "NNUE resource size mismatch (expected sha256 {SHA256})");
    let mut values = bytes.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]));
    let weights: Vec<i16> = values.by_ref().take(8 * FEATURES * HIDDEN).collect();
    let mut biases = Box::new([0i16; HIDDEN]);
    for value in biases.iter_mut() {
        *value = values.next().unwrap();
    }
    let output: Vec<i16> = values.by_ref().take(2 * HIDDEN).collect();
    let output_bias = values.next().unwrap();
    Network { weights, biases, output, output_bias }
});

/// Forces loading the network, so the first search does not pay for it.
pub fn init() {
    LazyLock::force(&NETWORK);
}

#[inline(always)]
fn weights(offset: usize) -> &'static Accumulator {
    NETWORK.weights[offset..offset + HIDDEN].try_into().unwrap()
}

/// Incrementally updated accumulators, one pair per search ply.
pub struct Nnue {
    stack: Vec<[Accumulator; 2]>,
    // piece placement each stack entry was computed for
    positions: Vec<Option<[u64; 16]>>,
}

impl Default for Nnue {
    fn default() -> Self {
        Self::new()
    }
}

impl Nnue {
    pub fn new() -> Self {
        Nnue {
            stack: vec![[[0; HIDDEN]; 2]; MAX_PLY + 2],
            positions: vec![None; MAX_PLY + 2],
        }
    }

    pub fn reset(&mut self, state: &BoardState) {
        self.positions.fill(None);
        self.refresh(state, 0);
    }

    pub fn refresh(&mut self, state: &BoardState, ply: usize) {
        for perspective in 0..2 {
            self.refresh_perspective(state, ply, perspective);
        }
        self.positions[ply] = Some(*state.bitboards());
    }

    fn refresh_perspective(&mut self, state: &BoardState, ply: usize, perspective: usize) {
        let king = state.bitboard_of_side(perspective, KING).trailing_zeros() as usize;
        let bucket = bucket(king, perspective);
        let accumulator = &mut self.stack[ply][perspective];
        *accumulator = *NETWORK.biases;
        for side in 0..2 {
            for piece_type in PAWN..=KING {
                let piece = make_piece(side, piece_type);
                let mut pieces = state.bitboard_of(piece);
                while pieces != 0 {
                    let offset = (bucket * FEATURES + feature(piece, pieces.trailing_zeros() as usize, perspective, king)) * HIDDEN;
                    for (value, weight) in accumulator.iter_mut().zip(weights(offset)) {
                        *value = value.wrapping_add(*weight);
                    }
                    pieces &= pieces - 1;
                }
            }
        }
    }

    /// Without incremental updates every child is evaluated from a full refresh.
    pub fn push(&mut self, _before: &BoardState, after: &BoardState, ply: usize) {
        self.refresh(after, ply);
    }

    pub fn push_move(&mut self, _before: &BoardState, after: &BoardState, _mv: Move, ply: usize) {
        self.refresh(after, ply);
    }

    pub fn accumulator(&mut self, state: &BoardState, ply: usize, perspective: usize) -> Vec<i16> {
        if self.positions[ply].as_ref() != Some(state.bitboards()) {
            self.refresh(state, ply);
        }
        self.stack[ply][perspective].to_vec()
    }

    pub fn raw(&mut self, state: &BoardState, ply: usize) -> i32 {
        if self.positions[ply].as_ref() != Some(state.bitboards()) {
            self.refresh(state, ply);
        }
        let side = state.side_to_play();
        forward(&self.stack[ply][side], &self.stack[ply][1 - side])
    }

    pub fn evaluate(&mut self, state: &BoardState, ply: usize) -> i32 {
        self.evaluate_position(state, ply) * (200 - state.half_move_clock) / 200
    }

    /// Material-scaled output without the 50-move scaling, so it depends only on the position.
    pub fn evaluate_position(&mut self, state: &BoardState, ply: usize) -> i32 {
        let both = |piece_type| (state.bitboard_of_side(0, piece_type) | state.bitboard_of_side(1, piece_type)).count_ones() as i32;
        let phase = 3 * both(KNIGHT) + 3 * both(BISHOP) + 5 * both(ROOK) + 10 * both(QUEEN);
        self.raw(state, ply) * (22400 + phase) / 32768
    }
}

/// SCReLU output layer; the i16 product deliberately wraps like the reference implementation.
pub fn forward(us: &Accumulator, them: &Accumulator) -> i32 {
    let network = &*NETWORK;
    let (out_us, out_them): (&Accumulator, &Accumulator) =
        (network.output[..HIDDEN].try_into().unwrap(), network.output[HIDDEN..].try_into().unwrap());
    let mut sum = 0i32;
    for i in 0..HIDDEN {
        let ours = us[i].clamp(0, 255);
        let theirs = them[i].clamp(0, 255);
        sum = sum
            .wrapping_add((ours as i32).wrapping_mul(ours.wrapping_mul(out_us[i]) as i32))
            .wrapping_add((theirs as i32).wrapping_mul(theirs.wrapping_mul(out_them[i]) as i32));
    }
    sum /= 255;
    sum += network.output_bias as i32;
    sum *= 400;
    sum / (255 * 64)
}

pub fn feature(piece: usize, square: usize, perspective: usize, king: usize) -> usize {
    let transformed = square ^ (if perspective == 1 { 56 } else { 0 }) ^ (if (king & 7) > 3 { 7 } else { 0 });
    (if piece >> 3 == perspective { 0 } else { 384 }) + type_of(piece) * 64 + transformed
}

#[inline(always)]
fn bucket(king: usize, perspective: usize) -> usize {
    BUCKETS[king ^ (if perspective == 1 { 56 } else { 0 })]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::START_POS;
    use crate::random::JavaRandom;

    fn compare(incremental: &mut Nnue, fresh: &mut Nnue, state: &BoardState, ply: usize) {
        fresh.refresh(state, 0);
        assert_eq!(fresh.raw(state, 0), incremental.raw(state, ply));
        assert_eq!(fresh.evaluate(state, 0), incremental.evaluate(state, ply));
        for perspective in 0..2 {
            assert_eq!(fresh.accumulator(state, 0, perspective), incremental.accumulator(state, ply, perspective));
        }
    }

}
