//! Zerofish 1024-hidden, 8 king-bucket NNUE (768 inputs per perspective, SCReLU output).

use std::sync::LazyLock;

use crate::board::BoardState;
use crate::moves::Move;
use crate::search::MAX_PLY;
use crate::types::*;

pub const SHA256: &str = "85959e702a2147cceaf6829a6933211865e2c271d434f394ac46063b90f7a6b3";
pub const RULE50_DENOMINATOR: i32 = 200;
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
    // accumulator refresh cache indexed by (perspective * 8 + bucket) * 2 + mirrored
    cached_accumulators: Vec<Accumulator>,
    cached_boards: Vec<[u64; 16]>,
}

impl Default for Nnue {
    fn default() -> Self {
        Self::new()
    }
}

impl Nnue {
    pub fn new() -> Self {
        let network = &*NETWORK;
        Nnue {
            stack: vec![[[0; HIDDEN]; 2]; MAX_PLY + 2],
            positions: vec![None; MAX_PLY + 2],
            cached_accumulators: vec![*network.biases; 32],
            cached_boards: vec![[0; 16]; 32],
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

    #[inline(always)]
    fn refresh_perspective(&mut self, state: &BoardState, ply: usize, perspective: usize) {
        let king = state.bitboard_of_side(perspective, KING).trailing_zeros() as usize;
        let bucket = bucket(king, perspective);
        let slot = (perspective * 8 + bucket) * 2 + usize::from((king & 7) > 3);
        let cached = &mut self.cached_accumulators[slot];
        let boards = &mut self.cached_boards[slot];
        for side in 0..2 {
            for piece_type in PAWN..=KING {
                let piece = make_piece(side, piece_type);
                let now = state.bitboard_of(piece);
                let mut added = now & !boards[piece];
                while added != 0 {
                    let offset = (bucket * FEATURES + feature(piece, added.trailing_zeros() as usize, perspective, king)) * HIDDEN;
                    for (value, weight) in cached.iter_mut().zip(weights(offset)) {
                        *value = value.wrapping_add(*weight);
                    }
                    added &= added - 1;
                }
                let mut removed = boards[piece] & !now;
                while removed != 0 {
                    let offset = (bucket * FEATURES + feature(piece, removed.trailing_zeros() as usize, perspective, king)) * HIDDEN;
                    for (value, weight) in cached.iter_mut().zip(weights(offset)) {
                        *value = value.wrapping_sub(*weight);
                    }
                    removed &= removed - 1;
                }
                boards[piece] = now;
            }
        }
        self.stack[ply][perspective] = *cached;
    }

    /// For a null move or any transition whose changed squares are not known.
    pub fn push(&mut self, before: &BoardState, after: &BoardState, ply: usize) {
        let mut changed = 0;
        for side in 0..2 {
            for piece_type in 0..6 {
                changed |= before.bitboard_of_side(side, piece_type) ^ after.bitboard_of_side(side, piece_type);
            }
        }
        self.push_changed(before, after, ply, changed);
    }

    pub fn push_move(&mut self, before: &BoardState, after: &BoardState, mv: Move, ply: usize) {
        let mut changed = 1u64 << mv.from() | 1u64 << mv.to();
        if mv.flags() == Move::EN_PASSANT {
            changed |= 1u64 << (mv.to() as i32 + if before.side_to_play() == 0 { -8 } else { 8 });
        }
        if mv.is_castling() {
            let rank = if before.side_to_play() == 0 { 0 } else { 56 };
            changed |= if before.is_chess960() {
                let kingside = mv.flags() == Move::OO;
                1u64 << before.castling_rook_square(before.side_to_play(), kingside) | if kingside { 1u64 << (rank + 6) | 1u64 << (rank + 5) } else { 1u64 << (rank + 2) | 1u64 << (rank + 3) }
            } else if mv.flags() == Move::OO {
                1u64 << (rank + 7) | 1u64 << (rank + 5)
            } else {
                1u64 << rank | 1u64 << (rank + 3)
            };
        }
        self.push_changed(before, after, ply, changed);
    }

    fn push_changed(&mut self, before: &BoardState, after: &BoardState, ply: usize, changed: u64) {
        #[cfg(target_arch = "x86_64")]
        if is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 support was just checked.
            return unsafe { self.push_changed_avx2(before, after, ply, changed) };
        }
        self.push_changed_impl(before, after, ply, changed)
    }

    // same code, auto-vectorized with 256-bit registers
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    fn push_changed_avx2(&mut self, before: &BoardState, after: &BoardState, ply: usize, changed: u64) {
        self.push_changed_impl(before, after, ply, changed)
    }

    #[inline(always)]
    fn push_changed_impl(&mut self, before: &BoardState, after: &BoardState, ply: usize, mut changed: u64) {
        if self.positions[ply - 1].as_ref() != Some(before.bitboards()) {
            self.refresh(before, ply - 1);
        }
        let mut added = [(0usize, 0usize); 4];
        let mut removed = [(0usize, 0usize); 4];
        let mut adds = 0;
        let mut subs = 0;
        while changed != 0 {
            let square = changed.trailing_zeros() as usize;
            changed &= changed - 1;
            let old_piece = before.piece_at(square);
            let new_piece = after.piece_at(square);
            if old_piece == new_piece {
                continue;
            }
            if old_piece != NONE {
                removed[subs] = (old_piece, square);
                subs += 1;
            }
            if new_piece != NONE {
                added[adds] = (new_piece, square);
                adds += 1;
            }
        }
        for perspective in 0..2 {
            let king = after.bitboard_of_side(perspective, KING).trailing_zeros() as usize;
            let previous_king = before.bitboard_of_side(perspective, KING).trailing_zeros() as usize;
            if bucket(king, perspective) != bucket(previous_king, perspective) || ((king & 7) > 3) != ((previous_king & 7) > 3) {
                self.refresh_perspective(after, ply, perspective);
                continue;
            }
            let bucket = bucket(king, perspective);
            let (lower, upper) = self.stack.split_at_mut(ply);
            let source = &lower[ply - 1][perspective];
            let destination = &mut upper[0][perspective];
            let offset = |(piece, square): (usize, usize)| (bucket * FEATURES + feature(piece, square, perspective, king)) * HIDDEN;
            match (adds, subs) {
                (0, 0) => *destination = *source,
                (1, 1) => {
                    let (add, sub) = (weights(offset(added[0])), weights(offset(removed[0])));
                    for i in 0..HIDDEN {
                        destination[i] = source[i].wrapping_sub(sub[i]).wrapping_add(add[i]);
                    }
                }
                (1, 2) => {
                    let (add, sub1, sub2) = (weights(offset(added[0])), weights(offset(removed[0])), weights(offset(removed[1])));
                    for i in 0..HIDDEN {
                        destination[i] = source[i].wrapping_sub(sub1[i]).wrapping_sub(sub2[i]).wrapping_add(add[i]);
                    }
                }
                (2, 2) => {
                    let (add1, add2) = (weights(offset(added[0])), weights(offset(added[1])));
                    let (sub1, sub2) = (weights(offset(removed[0])), weights(offset(removed[1])));
                    for i in 0..HIDDEN {
                        destination[i] = source[i].wrapping_sub(sub1[i]).wrapping_sub(sub2[i]).wrapping_add(add1[i]).wrapping_add(add2[i]);
                    }
                }
                _ => {
                    *destination = *source;
                    for &change in &removed[..subs] {
                        for (value, weight) in destination.iter_mut().zip(weights(offset(change))) {
                            *value = value.wrapping_sub(*weight);
                        }
                    }
                    for &change in &added[..adds] {
                        for (value, weight) in destination.iter_mut().zip(weights(offset(change))) {
                            *value = value.wrapping_add(*weight);
                        }
                    }
                }
            }
        }
        self.positions[ply] = Some(*after.bitboards());
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

    pub fn clocked(state: &BoardState, score: i32) -> i32 { score * (200 - state.half_move_clock) / 200 }

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
    let mut sum = screlu_dot(us, out_us).wrapping_add(screlu_dot(them, out_them));
    sum /= 255;
    sum += network.output_bias as i32;
    sum *= 400;
    sum / (255 * 64)
}

/// Sum of clamp(v)^2 * w as clamp(v) * (clamp(v) * w wrapping in i16), wrapping in i32.
#[inline(always)]
fn screlu_dot(accumulator: &Accumulator, weights: &Accumulator) -> i32 {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: AVX2 support was just checked.
            return unsafe { simd::screlu_dot_avx2(accumulator, weights) };
        }
        simd::screlu_dot_sse2(accumulator, weights)
    }
    #[cfg(not(target_arch = "x86_64"))]
    screlu_dot_scalar(accumulator, weights)
}

#[cfg_attr(target_arch = "x86_64", allow(dead_code))]
fn screlu_dot_scalar(accumulator: &Accumulator, weights: &Accumulator) -> i32 {
    let mut sum = 0i32;
    for i in 0..HIDDEN {
        let value = accumulator[i].clamp(0, 255);
        sum = sum.wrapping_add((value as i32).wrapping_mul(value.wrapping_mul(weights[i]) as i32));
    }
    sum
}

// pmaddwd sums two i16 products into one i32 lane; |255 * i16| * 2 cannot overflow, and the
// i32 lane sums wrap, so the result is bit-identical to the scalar loop.
#[cfg(target_arch = "x86_64")]
mod simd {
    use super::{Accumulator, HIDDEN};
    use std::arch::x86_64::*;

    #[target_feature(enable = "avx2")]
    pub(super) fn screlu_dot_avx2(accumulator: &Accumulator, weights: &Accumulator) -> i32 {
        let zero = _mm256_setzero_si256();
        let max = _mm256_set1_epi16(255);
        let mut sum = zero;
        for i in (0..HIDDEN).step_by(16) {
            // SAFETY: i + 16 <= HIDDEN, unaligned loads.
            let (value, weight) = unsafe {
                (
                    _mm256_loadu_si256(accumulator.as_ptr().add(i).cast()),
                    _mm256_loadu_si256(weights.as_ptr().add(i).cast()),
                )
            };
            let value = _mm256_min_epi16(_mm256_max_epi16(value, zero), max);
            sum = _mm256_add_epi32(sum, _mm256_madd_epi16(value, _mm256_mullo_epi16(value, weight)));
        }
        let sum = _mm_add_epi32(_mm256_castsi256_si128(sum), _mm256_extracti128_si256::<1>(sum));
        // SAFETY: SSE2 is part of the x86_64 baseline.
        unsafe { horizontal_sum(sum) }
    }

    pub(super) fn screlu_dot_sse2(accumulator: &Accumulator, weights: &Accumulator) -> i32 {
        // SAFETY: SSE2 is part of the x86_64 baseline; i + 8 <= HIDDEN, unaligned loads.
        unsafe {
            let zero = _mm_setzero_si128();
            let max = _mm_set1_epi16(255);
            let mut sum = zero;
            for i in (0..HIDDEN).step_by(8) {
                let value = _mm_loadu_si128(accumulator.as_ptr().add(i).cast());
                let weight = _mm_loadu_si128(weights.as_ptr().add(i).cast());
                let value = _mm_min_epi16(_mm_max_epi16(value, zero), max);
                sum = _mm_add_epi32(sum, _mm_madd_epi16(value, _mm_mullo_epi16(value, weight)));
            }
            horizontal_sum(sum)
        }
    }

    #[inline(always)]
    unsafe fn horizontal_sum(sum: __m128i) -> i32 {
        let sum = _mm_add_epi32(sum, _mm_shuffle_epi32::<0b01_00_11_10>(sum));
        let sum = _mm_add_epi32(sum, _mm_shuffle_epi32::<0b10_11_00_01>(sum));
        _mm_cvtsi128_si32(sum)
    }
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

    #[test]
    fn screlu_dot_matches_scalar() {
        let mut random = JavaRandom::new(11);
        let mut values = Box::new([0i16; HIDDEN]);
        let mut weights = Box::new([0i16; HIDDEN]);
        for round in 0..64 {
            for (value, weight) in values.iter_mut().zip(weights.iter_mut()) {
                *value = random.next_int(65536) as i16;
                *weight = if round % 2 == 0 { random.next_int(65536) as i16 } else { [i16::MIN, i16::MAX][random.next_int(2) as usize] };
            }
            if round % 4 == 1 {
                values.fill(255);
            }
            assert_eq!(screlu_dot(&values, &weights), screlu_dot_scalar(&values, &weights));
            #[cfg(target_arch = "x86_64")]
            assert_eq!(simd::screlu_dot_sse2(&values, &weights), screlu_dot_scalar(&values, &weights));
        }
    }

    #[test]
    fn incremental_special_moves_and_walks_match_refresh() {
        let mut incremental = Nnue::new();
        let mut fresh = Nnue::new();
        for fen in [
            START_POS,
            "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1",
            "4k3/P7/8/3pP3/8/8/8/4K3 w - d6 0 1",
            "1r2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
            "4k3/8/8/8/8/3K4/8/8 w - - 99 1",
            "4k3/8/8/8/3Pp3/8/p7/1R2K3 b - d3 0 1",
        ] {
            let root = BoardState::from_fen(fen).unwrap();
            incremental.reset(&root);
            for mv in root.generate_legal_moves().iter() {
                let child = root.do_move(mv);
                incremental.push_move(&root, &child, mv, 1);
                compare(&mut incremental, &mut fresh, &child, 1);
                compare(&mut incremental, &mut fresh, &root, 0);
            }
            let null_state = root.do_null_move();
            incremental.push(&root, &null_state, 1);
            compare(&mut incremental, &mut fresh, &null_state, 1);
        }
        let mut random = JavaRandom::new(73);
        let mut state = BoardState::from_fen(START_POS).unwrap();
        incremental.reset(&state);
        for ply in 1..=100 {
            let moves = state.generate_legal_moves();
            if moves.is_empty() {
                break;
            }
            let mv = moves.get(random.next_int(moves.len() as i32) as usize);
            let child = state.do_move(mv);
            incremental.push_move(&state, &child, mv, ply);
            compare(&mut incremental, &mut fresh, &child, ply);
            state = child;
        }
    }

    #[test]
    fn chess960_incremental_castling_matches_refresh() {
        let mut incremental = Nnue::new();
        let mut fresh = Nnue::new();
        for side in [WHITE, BLACK] {
            for king in 1..=6 {
                for rook in 0..8 {
                    if king == rook {
                        continue;
                    }
                    let mut rank = String::new();
                    let mut empty = 0;
                    for file in 0..8 {
                        if file == king || file == rook {
                            if empty != 0 {
                                rank.push_str(&empty.to_string());
                                empty = 0;
                            }
                            rank.push(if file == king { 'K' } else { 'R' });
                        } else {
                            empty += 1;
                        }
                    }
                    if empty != 0 {
                        rank.push_str(&empty.to_string());
                    }
                    let fen = if side == WHITE {
                        format!("8/7k/8/8/8/8/8/{rank} w {} - 0 1", (b'A' + rook) as char)
                    } else {
                        format!("{}/8/8/8/8/8/7K/8 b {} - 0 1", rank.to_ascii_lowercase(), (b'a' + rook) as char)
                    };
                    let before = BoardState::from_fen(&fen).unwrap();
                    let castle = before.generate_legal_moves().iter().find(|mv| mv.is_castling()).unwrap();
                    incremental.reset(&before);
                    let after = before.do_move(castle);
                    incremental.push_move(&before, &after, castle, 1);
                    compare(&mut incremental, &mut fresh, &after, 1);
                    compare(&mut incremental, &mut fresh, &before, 0);
                }
            }
        }
    }
}
