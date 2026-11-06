//! Explicit evaluation backends. Weights are runtime data, never embedded in engine code.
use crate::{board::BoardState, moves::Move};
use sha2::{Digest, Sha256};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EvaluationBackend {
    ZerofishNnue,
    StockfishNnue,
    Classical,
}
impl EvaluationBackend {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "zerofish-nnue" => Ok(Self::ZerofishNnue),
            "stockfish-nnue" => Ok(Self::StockfishNnue),
            "classical" => Ok(Self::Classical),
            _ => Err(format!("Unknown evaluationBackend: {value}")),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::ZerofishNnue => "zerofish-nnue",
            Self::StockfishNnue => "stockfish-nnue",
            Self::Classical => "classical",
        }
    }
    pub fn is_loaded(self) -> bool {
        match self {
            Self::ZerofishNnue => crate::zerofish_nnue::is_loaded(),
            Self::StockfishNnue => crate::stockfish_rust::is_loaded(),
            Self::Classical => true,
        }
    }
    pub fn load(self, bytes: &[u8]) -> Result<(), String> {
        match self {
            Self::ZerofishNnue => crate::zerofish_nnue::load(bytes),
            Self::StockfishNnue => crate::stockfish_rust::load(bytes),
            Self::Classical if bytes.is_empty() => Ok(()),
            Self::Classical => Err("Classical evaluation does not accept a network".into()),
        }
    }
    pub fn load_file(self, path: Option<&std::path::Path>) -> Result<(), String> {
        if self == Self::Classical {
            return if path.is_none() {
                Ok(())
            } else {
                Err("Classical evaluation does not accept --network".into())
            };
        }
        let fallback = match self {
            Self::ZerofishNnue => std::env::var_os("ZEROFISH_NETWORK")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/resources/weights-original.nnue"
                    )
                    .into()
                }),
            _ => std::env::var_os("ZEROFISH_STOCKFISH_NETWORK")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/../stockfish/src/nn-c9e480e92425.nnue"
                    )
                    .into()
                }),
        };
        let path = path.unwrap_or(&fallback);
        self.load(
            &std::fs::read(path)
                .map_err(|e| format!("Cannot read NNUE {}: {e}", path.display()))?,
        )
    }
}
pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
pub fn validate(bytes: &[u8], size: usize, expected: &str) -> Result<(), String> {
    if bytes.len() != size {
        return Err(format!("NNUE size mismatch: {} != {size}", bytes.len()));
    }
    if digest(bytes) != expected.to_ascii_lowercase() {
        return Err("NNUE SHA-256 mismatch".into());
    }
    Ok(())
}
static DEFAULT_BACKEND: OnceLock<EvaluationBackend> = OnceLock::new();
static EVALUATIONS: AtomicU64 = AtomicU64::new(0);
pub fn evaluation_count() -> u64 {
    EVALUATIONS.load(Ordering::Relaxed)
}
pub fn default_backend() -> EvaluationBackend {
    *DEFAULT_BACKEND
        .get()
        .unwrap_or(&EvaluationBackend::StockfishNnue)
}
pub fn configure(backend: EvaluationBackend, path: Option<&std::path::Path>) -> Result<(), String> {
    if DEFAULT_BACKEND.get().is_some() {
        return Err("Evaluation backend is already configured".into());
    }
    backend.load_file(path)?;
    DEFAULT_BACKEND
        .set(backend)
        .map_err(|_| "Evaluation backend is already configured".into())
}
pub fn init() {
    if default_backend() != EvaluationBackend::Classical {
        let _ = Nnue::new();
    }
}

pub enum Nnue {
    Zerofish(crate::zerofish_nnue::Nnue),
    Stockfish(crate::stockfish_rust::Nnue),
}
impl Default for Nnue {
    fn default() -> Self {
        Self::new()
    }
}
impl Nnue {
    pub fn new() -> Self {
        Self::for_backend(default_backend()).expect("classical evaluation has no NNUE")
    }
    pub fn for_backend(backend: EvaluationBackend) -> Option<Self> {
        match backend {
            EvaluationBackend::ZerofishNnue => {
                Some(Self::Zerofish(crate::zerofish_nnue::Nnue::new()))
            }
            EvaluationBackend::StockfishNnue => {
                Some(Self::Stockfish(crate::stockfish_rust::Nnue::new()))
            }
            EvaluationBackend::Classical => None,
        }
    }
    pub fn reset(&mut self, state: &BoardState) {
        match self {
            Self::Zerofish(n) => n.reset(state),
            Self::Stockfish(n) => n.reset(state),
        }
    }
    pub fn push(&mut self, before: &BoardState, after: &BoardState, ply: usize) {
        match self {
            Self::Zerofish(n) => n.push(before, after, ply),
            Self::Stockfish(n) => n.push(before, after, ply),
        }
    }
    pub fn push_move(&mut self, before: &BoardState, after: &BoardState, mv: Move, ply: usize) {
        match self {
            Self::Zerofish(n) => n.push_move(before, after, mv, ply),
            Self::Stockfish(n) => n.push_move(before, after, mv, ply),
        }
    }
    pub fn evaluate_position(&mut self, state: &BoardState, ply: usize) -> i32 {
        EVALUATIONS.fetch_add(1, Ordering::Relaxed);
        match self {
            Self::Zerofish(n) => n.evaluate_position(state, ply),
            Self::Stockfish(n) => n.evaluate_position(state, ply),
        }
    }
    pub fn clocked(&self, state: &BoardState, score: i32) -> i32 {
        match self {
            Self::Zerofish(_) => crate::zerofish_nnue::Nnue::clocked(state, score),
            Self::Stockfish(_) => crate::stockfish_rust::Nnue::clocked(state, score),
        }
    }
    pub fn evaluate(&mut self, state: &BoardState, ply: usize) -> i32 {
        let score = self.evaluate_position(state, ply);
        self.clocked(state, score)
    }
}

/// Parse explicit native evaluator flags; return unrelated positional arguments.
pub fn cli_configuration(
    default: EvaluationBackend,
) -> Result<(EvaluationBackend, Option<std::path::PathBuf>, Vec<String>), String> {
    let mut backend = default;
    let mut network = None;
    let mut remaining = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--evaluation-backend" => {
                backend = EvaluationBackend::parse(
                    &args.next().ok_or("Missing --evaluation-backend value")?,
                )?
            }
            "--network" => {
                network = Some(std::path::PathBuf::from(
                    args.next().ok_or("Missing --network value")?,
                ))
            }
            _ => remaining.push(arg),
        }
    }
    Ok((backend, network, remaining))
}
