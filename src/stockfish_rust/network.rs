// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2004-2026 The Stockfish developers.
// Rust adaptation of the pinned NNUE contracts; see native/UPSTREAM.json.
use super::{features::DIMENSIONS, simd::Kernel};
use crate::{board::BoardState, types::*};
use std::sync::LazyLock;
pub const HIDDEN: usize = 1024;
pub type Accumulator = [i16; HIDDEN];
const FT_HASH: u32 = {
    let hash = 0x2e6b9d04u32.rotate_left(1) ^ 0x86f2b1ddu32;
    (hash.rotate_left(1) ^ 0x32b5e284u32) ^ 2048
};
const fn affine_hash(previous: u32, outputs: u32) -> u32 {
    0xCC03DAE4u32.wrapping_add(outputs) ^ previous.rotate_right(1)
}
const ARCH_HASH: u32 = affine_hash(
    affine_hash(
        affine_hash(0xEC42E90D ^ 2048, 32).wrapping_add(0x538D24C7),
        32,
    )
    .wrapping_add(0x538D24C7),
    1,
);
const BYTES: &[u8] = include_bytes!(env!("ZEROFISH_STOCKFISH_NETWORK"));
pub static NETWORK: LazyLock<Network> =
    LazyLock::new(|| Network::read(BYTES).expect("invalid pinned Stockfish network"));

#[repr(C, align(64))]
struct CacheLine([i8; 64]);
pub struct AlignedBytes(Vec<CacheLine>);
impl AlignedBytes {
    fn new(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len() % 64, 0);
        let count = bytes.len() / 64;
        let mut storage = Vec::<CacheLine>::with_capacity(count);
        // The allocation is large enough, aligned, and disjoint from the source.
        // All bit patterns of CacheLine are valid; every byte is initialized here.
        unsafe {
            std::ptr::copy_nonoverlapping(
                bytes.as_ptr(),
                storage.as_mut_ptr().cast::<u8>(),
                bytes.len(),
            );
            storage.set_len(count);
        }
        Self(storage)
    }
}
impl std::ops::Deref for AlignedBytes {
    type Target = [i8];
    fn deref(&self) -> &[i8] {
        // CacheLine has exactly 64 bytes, contains no padding, and owns initialized i8s.
        unsafe { std::slice::from_raw_parts(self.0.as_ptr().cast::<i8>(), self.0.len() * 64) }
    }
}

pub struct Layer {
    pub inputs: usize,
    pub outputs: usize,
    pub biases: Vec<i32>,
    // Four consecutive inputs, then output channel; matches VNNI's u8*i8 dot groups.
    pub weights: AlignedBytes,
}
impl Layer {
    fn read(reader: &mut Reader, inputs: usize, outputs: usize) -> Result<Self, String> {
        let biases = (0..outputs)
            .map(|_| reader.i32())
            .collect::<Result<_, _>>()?;
        let serialized = reader.take(inputs * outputs)?;
        let mut weights = vec![0; inputs * outputs];
        for o in 0..outputs {
            for i in 0..inputs {
                weights[(i / 4) * outputs * 4 + o * 4 + i % 4] = serialized[o * inputs + i] as i8;
            }
        }
        let bytes =
            unsafe { std::slice::from_raw_parts(weights.as_ptr().cast::<u8>(), weights.len()) };
        Ok(Self {
            inputs,
            outputs,
            biases,
            weights: AlignedBytes::new(bytes),
        })
    }
}
pub struct Stack {
    pub first: Layer,
    pub second: Layer,
    pub last: Layer,
}
pub struct Network {
    pub biases: Accumulator,
    pub weights: AlignedBytes,
    pub stacks: Vec<Stack>,
}
struct Reader {
    bytes: &'static [u8],
    position: usize,
}
impl Reader {
    fn take(&mut self, count: usize) -> Result<&'static [u8], String> {
        let end = self
            .position
            .checked_add(count)
            .ok_or("network length overflow")?;
        let result = self
            .bytes
            .get(self.position..end)
            .ok_or("truncated network")?;
        self.position = end;
        Ok(result)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, String> {
        Ok(self.u32()? as i32)
    }
    fn expect(&mut self, value: u32) -> Result<(), String> {
        let actual = self.u32()?;
        if actual == value {
            Ok(())
        } else {
            Err(format!(
                "network header mismatch: {actual:#x} != {value:#x}"
            ))
        }
    }
}
impl Network {
    pub fn read(bytes: &'static [u8]) -> Result<Self, String> {
        let mut reader = Reader { bytes, position: 0 };
        reader.expect(0x6A448AFA)?;
        reader.expect(FT_HASH ^ ARCH_HASH)?;
        let description_length = reader.u32()? as usize;
        reader.take(description_length)?;
        reader.expect(FT_HASH)?;
        let mut biases = [0; HIDDEN];
        for bias in &mut biases {
            *bias = i16::from_le_bytes(reader.take(2)?.try_into().unwrap());
        }
        let raw = reader.take(DIMENSIONS * HIDDEN)?;
        let weights = AlignedBytes::new(raw);
        let mut stacks = Vec::with_capacity(32);
        for _ in 0..32 {
            reader.expect(ARCH_HASH)?;
            stacks.push(Stack {
                first: Layer::read(&mut reader, 1024, 32)?,
                second: Layer::read(&mut reader, 64, 32)?,
                last: Layer::read(&mut reader, 128, 1)?,
            });
        }
        if reader.position != bytes.len() {
            return Err("trailing network data".into());
        }
        Ok(Self {
            biases,
            weights,
            stacks,
        })
    }
    pub fn forward(
        &self,
        state: &BoardState,
        accumulators: &[Accumulator; 2],
        kernel: Kernel,
    ) -> i32 {
        let stm = state.side_to_play();
        let queen_bucket = usize::from(state.bitboard_of_side(stm, QUEEN) != 0) * 2
            + usize::from(state.bitboard_of_side(1 - stm, QUEEN) != 0);
        let bucket = (state.all_pieces().count_ones() as usize - 1) / 4 * 4 + queen_bucket;
        let stack = &self.stacks[bucket];
        let mut input = [0; HIDDEN];
        kernel.transform(&accumulators[stm], &mut input[..512]);
        kernel.transform(&accumulators[1 - stm], &mut input[512..]);
        let mut first = [0; 32];
        kernel.affine(&stack.first, &input, &mut first);
        let mut concat = [0; 128];
        kernel.activate(&first, &mut concat[..64], 7);
        let mut second = [0; 32];
        kernel.affine(&stack.second, &concat[..64], &mut second);
        kernel.activate(&second, &mut concat[64..], 6);
        let mut last = [0];
        kernel.affine(&stack.last, &concat, &mut last);
        let out = last[0] + first[30] - first[31];
        ((out as i64 * 9600) / 16384) as i32 / 16
    }
}

fn counts(state: &BoardState) -> [i32; 5] {
    std::array::from_fn(|pt| {
        (state.bitboard_of_side(WHITE, pt) | state.bitboard_of_side(BLACK, pt)).count_ones() as i32
    })
}
pub fn scaled(state: &BoardState, raw: i32, clock: i32) -> i32 {
    let values = [208, 781, 825, 1276, 2538];
    let side = state.side_to_play();
    let mut simple = 0;
    let mut non_pawn = 0;
    for pt in PAWN..=QUEEN {
        let us = state.bitboard_of_side(side, pt).count_ones() as i32;
        let them = state.bitboard_of_side(1 - side, pt).count_ones() as i32;
        simple += values[pt] * (us - them);
        if pt != PAWN {
            non_pawn += values[pt] * (us + them);
        }
    }
    let se_norm = simple * 1024 / (simple.abs() + 1024);
    let nnue_norm = raw * 1024 / (raw.abs() + 1024);
    let correlation = se_norm * nnue_norm / 512;
    let mut score = raw + raw * correlation / 65536;
    let material = 521 * counts(state)[PAWN] + non_pawn;
    score += (material as i64 * score as i64 / 90649) as i32;
    score -= score * clock / 189;
    score.clamp(-31506, 31506)
}
pub fn centipawns(state: &BoardState, score: i32) -> i32 {
    let counts = counts(state);
    let material = counts[PAWN]
        + 3 * counts[KNIGHT]
        + 3 * counts[BISHOP]
        + 5 * counts[ROOK]
        + 9 * counts[QUEEN];
    let m = material.clamp(17, 78) as f64 / 58.0;
    let a = ((-142.72052667 * m + 372.35176398) * m - 340.71073572) * m + 415.23490212;
    (100.0 * score as f64 / a).round() as i32
}
