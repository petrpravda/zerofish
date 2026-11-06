// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2004-2026 The Stockfish developers.
// Rust adaptation of the pinned NNUE contracts; see native/UPSTREAM.json.
use super::{features::DIMENSIONS, simd::Kernel};
use crate::{board::BoardState, types::*};
use std::sync::OnceLock;
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
static NETWORK: OnceLock<Network> = OnceLock::new();
#[cfg(not(target_arch = "wasm32"))]
static LOAD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
pub fn load(bytes: &[u8]) -> Result<(), String> {
    if NETWORK.get().is_some() {
        return Err("Stockfish network is already initialized".into());
    }
    let expected =
        std::env::var("ZEROFISH_STOCKFISH_NETWORK_SHA256").unwrap_or_else(|_| super::SHA256.into());
    crate::nnue::validate(bytes, bytes.len(), &expected)?;
    let network = Network::read(bytes)?;
    NETWORK
        .set(network)
        .map_err(|_| "Stockfish network is already initialized".into())
}
pub fn identity() -> Option<(usize, String)> {
    NETWORK.get().map(|n| (n.bytes, n.sha256.clone()))
}
pub fn is_loaded() -> bool {
    NETWORK.get().is_some()
}
pub fn network() -> &'static Network {
    #[cfg(not(target_arch = "wasm32"))]
    if NETWORK.get().is_none() {
        let _guard = LOAD_LOCK.lock().unwrap();
        if let Some(network) = NETWORK.get() {
            return network;
        }
        let path = std::env::var_os("ZEROFISH_STOCKFISH_NETWORK")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../stockfish/src/nn-c9e480e92425.nnue"
                ))
            });
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|e| panic!("Cannot read NNUE {}: {e}", path.display()));
        let _ = load(&bytes);
    }
    NETWORK
        .get()
        .expect("Stockfish NNUE must be loaded before evaluation")
}

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
pub struct TransformerWeights {
    serialized: Vec<u8>,
    canonical: OnceLock<AlignedBytes>,
    paired: OnceLock<AlignedBytes>,
}
impl TransformerWeights {
    fn new(serialized: &[u8]) -> Self {
        Self {
            serialized: serialized.to_vec(),
            canonical: OnceLock::new(),
            paired: OnceLock::new(),
        }
    }
    fn paired(&self) -> &[i8] {
        &self.paired.get_or_init(|| {
            let mut output = AlignedBytes::new(&self.serialized);
            // In each 64-byte row segment, unpacklo/unpackhi must produce
            // consecutive canonical i16 lanes. This is Stockfish's paired
            // widening layout, independent of its additional packus layout.
            const ORDER: [usize; 8] = [0, 4, 1, 5, 2, 6, 3, 7];
            for line in &mut output.0 {
                let source = line.0;
                for (block, &from) in ORDER.iter().enumerate() {
                    line.0[block * 8..block * 8 + 8]
                        .copy_from_slice(&source[from * 8..from * 8 + 8]);
                }
            }
            output
        })[..]
    }
    pub fn for_kernel(&self, kernel: Kernel) -> &[i8] {
        if kernel == Kernel::Avx512Stockfish {
            self.paired()
        } else {
            self
        }
    }
}
impl std::ops::Deref for TransformerWeights {
    type Target = [i8];
    fn deref(&self) -> &[i8] {
        &self
            .canonical
            .get_or_init(|| AlignedBytes::new(&self.serialized))[..]
    }
}

pub struct Stack {
    pub first: Layer,
    pub second: Layer,
    pub last: Layer,
}
pub struct Network {
    pub bytes: usize,
    pub sha256: String,
    pub biases: Accumulator,
    pub weights: TransformerWeights,
    pub stacks: Vec<Stack>,
}
struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, count: usize) -> Result<&'a [u8], String> {
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
    pub fn read(bytes: &[u8]) -> Result<Self, String> {
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
        let weights = TransformerWeights::new(raw);
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
            bytes: bytes.len(),
            sha256: crate::nnue::digest(bytes),
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
        let out = kernel.propagate(stack, accumulators, stm);
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
