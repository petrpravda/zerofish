//! Optional linked NNUE reference for differential tests and timing comparisons.
//! The default engine evaluates through stockfish_rust and never calls this module.
use std::ffi::{c_char, c_int, c_void, CStr, CString};
use crate::{board::BoardState, moves::Move, search::MAX_PLY};

pub const SHA256: &str = "c9e480e92425f887df6ab9b05c6180fbd762b07879df9d9845f1aa75440caa73";
pub const RULE50_DENOMINATOR: i32 = 189;
extern "C" {
    fn jf_nnue_create() -> *mut c_void;
    fn jf_nnue_destroy(context: *mut c_void);
    fn jf_nnue_error() -> *const c_char;
    fn jf_nnue_reset_full(context: *mut c_void, fen: *const c_char, chess960: c_int, base: c_int) -> c_int;
    fn jf_nnue_reset(context: *mut c_void, fen: *const c_char, chess960: c_int, base: c_int) -> c_int;
    fn jf_nnue_push(context: *mut c_void, ply: c_int, from: c_int, to: c_int, flag: c_int) -> c_int;
    fn jf_nnue_raw(context: *mut c_void, ply: c_int) -> c_int;
    fn jf_nnue_eval(context: *mut c_void, ply: c_int) -> c_int;
    fn jf_nnue_accumulator(context: *mut c_void, ply: c_int, perspective: c_int, output: *mut i16) -> c_int;
    fn jf_nnue_full_internal(context: *mut c_void, ply: c_int) -> c_int;
}
fn check(value: i32) -> i32 {
    if value == i32::MIN { panic!("Stockfish NNUE: {}", unsafe { CStr::from_ptr(jf_nnue_error()) }.to_string_lossy()); }
    value
}
pub fn init() { drop(Nnue::new()); }
pub struct Nnue {
    context: *mut c_void,
    positions: Vec<Option<(u64, i32)>>,
    base: usize,
    depth: usize,
}
// A context is owned by a single worker. The native network is immutable and
// initialization uses call_once; worker contexts never share mutable state.
unsafe impl Send for Nnue {}
impl Drop for Nnue { fn drop(&mut self) { unsafe { jf_nnue_destroy(self.context) }; } }
impl Default for Nnue { fn default() -> Self { Self::new() } }
impl Nnue {
    pub fn new() -> Self {
        let context = unsafe { jf_nnue_create() };
        if context.is_null() { check(i32::MIN); }
        Self { context, positions: vec![None; MAX_PLY + 2], base: 0, depth: 0 }
    }
    pub fn reset(&mut self, state: &BoardState) { self.positions.fill(None); self.refresh(state, 0); }
    /// Reset with empty selected Finny entries: no previously accumulated features.
    pub fn refresh_full(&mut self, state: &BoardState, ply: usize) {
        let fen = CString::new(state.to_fen()).unwrap();
        check(unsafe { jf_nnue_reset_full(self.context, fen.as_ptr(), state.is_chess960() as i32, ply as i32) });
        self.positions[ply] = Some((state.hash(), state.half_move_clock));
        self.base = ply; self.depth = ply;
    }
    pub fn refresh(&mut self, state: &BoardState, ply: usize) {
        let fen = CString::new(state.to_fen()).unwrap();
        check(unsafe { jf_nnue_reset(self.context, fen.as_ptr(), state.is_chess960() as i32, ply as i32) });
        self.positions[ply] = Some((state.hash(), state.half_move_clock));
        self.base = ply;
        self.depth = ply;
    }
    fn ensure(&mut self, state: &BoardState, ply: usize) {
        // Hash includes side and castling rights; clock is needed by fullInternal.
        let same = self.positions[ply].as_ref().is_some_and(|p| p.0 == state.hash() && p.1 == state.half_move_clock);
        if !same || ply < self.base || ply > self.depth { self.refresh(state, ply); }
    }
    pub fn push(&mut self, before: &BoardState, after: &BoardState, ply: usize) {
        self.push_move(before, after, Move::NULL_MOVE, ply);
    }
    pub fn push_move(&mut self, before: &BoardState, after: &BoardState, mv: Move, ply: usize) {
        self.ensure(before, ply - 1);
        let to = if mv.is_castling() { before.castling_rook_square(before.side_to_play(), mv.flags() == Move::OO) } else { mv.to() };
        check(unsafe { jf_nnue_push(self.context, ply as i32, mv.from() as i32, to as i32, (mv.flags() >> 12) as i32) });
        self.positions[ply] = Some((after.hash(), after.half_move_clock));
        self.depth = ply;
    }
    pub fn accumulator(&mut self, state: &BoardState, ply: usize, perspective: usize) -> Vec<i16> {
        assert!(perspective < 2);
        self.ensure(state, ply);
        let mut output = vec![0; 1024];
        check(unsafe { jf_nnue_accumulator(self.context, ply as i32, perspective as i32, output.as_mut_ptr()) });
        self.depth = ply;
        output
    }
    pub fn raw(&mut self, state: &BoardState, ply: usize) -> i32 {
        self.ensure(state, ply);
        let result = check(unsafe { jf_nnue_raw(self.context, ply as i32) }); self.depth = ply; result
    }
    pub fn full_internal(&mut self, state: &BoardState, ply: usize) -> i32 {
        self.ensure(state, ply);
        let result = check(unsafe { jf_nnue_full_internal(self.context, ply as i32) }); self.depth = ply; result
    }
    pub fn evaluate_position(&mut self, state: &BoardState, ply: usize) -> i32 {
        self.ensure(state, ply);
        let result = check(unsafe { jf_nnue_eval(self.context, ply as i32) }); self.depth = ply; result
    }
    pub fn evaluate(&mut self, state: &BoardState, ply: usize) -> i32 {
        let score = self.evaluate_position(state, ply);
        score - score * state.half_move_clock / RULE50_DENOMINATOR
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{fen::START_POS, random::JavaRandom, types::*};
    fn compare(incremental: &mut Nnue, fresh: &mut Nnue, state: &BoardState, ply: usize) {
        fresh.refresh(state, 0);
        assert_eq!(fresh.raw(state, 0), incremental.raw(state, ply), "{}", state.to_fen());
        assert_eq!(fresh.evaluate(state, 0), incremental.evaluate(state, ply));
        assert_eq!(fresh.full_internal(state, 0), incremental.full_internal(state, ply));
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
                    // Stockfish rejects synthetic roots where the opponent king is already attacked.
                    if before.do_null_move().is_king_attacked() { continue; }
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

    #[test]
    fn raw_and_scaled_values_match_stockfish_executable() {
        use std::{io::Write, process::{Command, Stdio}};
        let binary = std::env::var("STOCKFISH_BINARY").expect("Set STOCKFISH_BINARY for the external oracle test");
        let mut evaluator = Nnue::new();
        for fen in [START_POS, "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", "4k3/P7/8/3pP3/8/8/8/4K3 w - d6 0 1", "8/8/4k3/8/4K3/4P3/8/8 w - - 50 60"] {
            let mut child = Command::new(&binary).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
            writeln!(child.stdin.take().unwrap(), "position fen {fen}\neval\nquit").unwrap();
            let output = child.wait_with_output().unwrap();
            assert!(output.status.success());
            let text = String::from_utf8(output.stdout).unwrap();
            let raw = text.lines().find(|line| line.contains("NNUE evaluation") && line.contains("internal units")).expect(&text);
            let raw: i32 = raw.split_whitespace().nth(2).unwrap().parse().unwrap();
            let state = BoardState::from_fen(fen).unwrap();
            assert_eq!(raw, evaluator.raw(&state, 0), "{fen}");
            if state.half_move_clock == 0 {
                let scaled = text.lines().find(|line| line.contains("Final evaluation")).expect(&text);
                let cp = (scaled.split_whitespace().nth(2).unwrap().parse::<f64>().unwrap() * 100.0).round() as i32;
                assert_eq!(cp * if state.side_to_play() == WHITE { 1 } else { -1 }, evaluator.evaluate(&state, 0), "{fen}");
            }
        }
    }
}
