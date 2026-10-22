//! Move ordering: staged move picker, killers, butterfly, capture and continuation histories.
//! Histories are atomics so Lazy SMP threads may share them; relaxed races are accepted, as in the Java engine.

use std::sync::atomic::{AtomicI16, AtomicI32, Ordering::Relaxed};
use std::sync::Arc;

use crate::board::BoardState;
use crate::moves::{Move, MoveList, MAX_MOVES};
use crate::search::MAX_PLY;
use crate::tt::TTEntry;
use crate::types::*;

const CONTEXTS: usize = 14 * 64;

fn value(piece_type: usize) -> i32 {
    [100, 320, 330, 500, 900, 20000][piece_type]
}

struct Histories {
    quiet: Vec<AtomicI32>,
    capture: Vec<AtomicI32>,
    continuation: Vec<AtomicI16>,
}

impl Histories {
    fn new() -> Self {
        Histories {
            quiet: (0..2 * 4096).map(|_| AtomicI32::new(0)).collect(),
            capture: (0..14 * 64 * 6).map(|_| AtomicI32::new(0)).collect(),
            continuation: (0..CONTEXTS * CONTEXTS).map(|_| AtomicI16::new(0)).collect(),
        }
    }
}

pub struct MoveOrdering {
    histories: Arc<Histories>,
    killers: Vec<[u16; 2]>,
}

impl Default for MoveOrdering {
    fn default() -> Self {
        Self::new()
    }
}

#[inline(always)]
pub fn tactical(mv: Move) -> bool {
    mv.is_promotion() || mv.flags() & Move::CAPTURE != 0
}

fn adjusted(value: i32, bonus: i32) -> i32 {
    value + bonus - value * bonus.abs() / 16384
}

#[inline(always)]
fn capture_index(piece: usize, to: usize, victim_type: usize) -> usize {
    (piece * 64 + to) * 6 + victim_type
}

impl MoveOrdering {
    pub fn new() -> Self {
        MoveOrdering { histories: Arc::new(Histories::new()), killers: vec![[0; 2]; MAX_PLY + 2] }
    }

    /// Shares the history tables with `other`; killers stay per instance.
    pub fn sharing(other: &MoveOrdering) -> Self {
        MoveOrdering { histories: Arc::clone(&other.histories), killers: vec![[0; 2]; MAX_PLY + 2] }
    }

    pub fn reset(&mut self) {
        let h = &*self.histories;
        h.quiet.iter().chain(&h.capture).for_each(|v| v.store(0, Relaxed));
        h.continuation.iter().for_each(|v| v.store(0, Relaxed));
        self.killers.fill([0; 2]);
    }

    #[inline(always)]
    pub fn quiet_score(&self, state: &BoardState, mv: Move, previous: i32) -> i32 {
        let h = &*self.histories;
        let mut score = h.quiet[state.side_to_play() * 4096 + mv.from() * 64 + mv.to()].load(Relaxed);
        if previous >= 0 {
            score += h.continuation[previous as usize * CONTEXTS + state.piece_at(mv.from()) * 64 + mv.to()].load(Relaxed) as i32;
        }
        score
    }

    fn capture_score(&self, state: &BoardState, mv: Move) -> i32 {
        let victim = state.piece_at(mv.to());
        let victim_type = if victim == NONE { PAWN } else { type_of(victim) };
        let mut score = if mv.flags() & Move::CAPTURE != 0 { value(victim_type) } else { 0 };
        if mv.is_promotion() {
            score += value(mv.piece_type()) - value(PAWN);
        }
        score + self.histories.capture[capture_index(state.piece_at(mv.from()), mv.to(), victim_type)].load(Relaxed) / 4
    }

    fn update(&self, state: &BoardState, mv: Move, previous: i32, bonus: i32) {
        let h = &*self.histories;
        if tactical(mv) {
            let victim = state.piece_at(mv.to());
            let victim_type = if victim == NONE { PAWN } else { type_of(victim) };
            let entry = &h.capture[capture_index(state.piece_at(mv.from()), mv.to(), victim_type)];
            entry.store(adjusted(entry.load(Relaxed), bonus), Relaxed);
        } else {
            let entry = &h.quiet[state.side_to_play() * 4096 + mv.from() * 64 + mv.to()];
            entry.store(adjusted(entry.load(Relaxed), bonus), Relaxed);
            if previous >= 0 {
                let entry = &h.continuation[previous as usize * CONTEXTS + state.piece_at(mv.from()) * 64 + mv.to()];
                entry.store(adjusted(entry.load(Relaxed) as i32, bonus) as i16, Relaxed);
            }
        }
    }

    /// Rewards the cutoff move, penalises the earlier moves of the same kind and records a quiet killer.
    pub fn cutoff(&mut self, state: &BoardState, mv: Move, ply: usize, previous: i32, depth: i32, picker: &Picker) {
        let bonus = (depth * depth * 16).min(2048);
        self.update(state, mv, previous, bonus);
        for &earlier in &picker.moves[..picker.cursor.saturating_sub(1)] {
            if tactical(earlier) == tactical(mv) {
                self.update(state, earlier, previous, -bonus);
            }
        }
        if !tactical(mv) && self.killers[ply][0] != mv.bits() {
            self.killers[ply][1] = self.killers[ply][0];
            self.killers[ply][0] = mv.bits();
        }
    }
}

/// Stages: 0 hash move, 1 good tactical, 2 killers, 3 quiets, 4 bad tactical.
pub struct Picker {
    previous: i32,
    moves: [Move; MAX_MOVES],
    stages: [u8; MAX_MOVES],
    scores: [i32; MAX_MOVES],
    skip_quiets: bool,
    count: usize,
    cursor: usize,
    stage: u8,
    quiet_scored: bool,
    generated: MoveList,
    loaded: [bool; 5],
    lazy: bool,
    tactical_only: bool,
    ply: usize,
    tt_move: Move,
    next_index: Option<usize>,
}

impl Default for Picker {
    fn default() -> Self {
        Self::new()
    }
}

impl Picker {
    pub fn new() -> Self {
        Picker {
            previous: -1,
            moves: [Move(0); MAX_MOVES],
            stages: [0; MAX_MOVES],
            scores: [0; MAX_MOVES],
            skip_quiets: false,
            count: 0,
            cursor: 0,
            stage: 0,
            quiet_scored: false,
            generated: MoveList::new(),
            loaded: [false; 5],
            lazy: true,
            tactical_only: false,
            ply: 0,
            tt_move: Move::NULL_MOVE,
            next_index: None,
        }
    }

    /// With `legal` the moves are given up front, otherwise they are generated stage by stage.
    #[allow(clippy::too_many_arguments)]
    pub fn reset(&mut self, ordering: &MoveOrdering, state: &BoardState, legal: Option<&MoveList>, entry: Option<TTEntry>, ply: usize, previous: i32, tactical_only: bool) {
        self.previous = previous;
        self.count = legal.map_or(0, MoveList::len);
        self.cursor = 0;
        self.stage = 0;
        self.quiet_scored = false;
        self.lazy = legal.is_none();
        self.tactical_only = tactical_only;
        self.skip_quiets = false;
        self.ply = ply;
        self.next_index = None;
        self.loaded = [false; 5];
        self.tt_move = entry.map_or(Move::NULL_MOVE, |e| e.mv);
        let killers = ordering.killers[ply];
        if let Some(legal) = legal {
            for (index, mv) in legal.iter().enumerate() {
                self.moves[index] = mv;
                self.scores[index] = 0;
                self.stages[index] = if mv == self.tt_move {
                    0
                } else if tactical(mv) {
                    self.scores[index] = ordering.capture_score(state, mv);
                    1
                } else if killers[0] == mv.bits() || killers[1] == mv.bits() {
                    self.scores[index] = i32::from(killers[0] == mv.bits());
                    2
                } else {
                    3
                };
            }
        }
    }

    fn append(&mut self, mv: Move, stage: u8, score: i32) {
        self.moves[self.count] = mv;
        self.stages[self.count] = stage;
        self.scores[self.count] = score;
        self.count += 1;
    }

    pub fn skip_quiets(&mut self) {
        self.skip_quiets = true;
    }

    pub fn last_stage(&self) -> u8 {
        self.stages[self.cursor - 1]
    }

    pub fn last_score(&self) -> i32 {
        self.scores[self.cursor - 1]
    }

    fn has_next(&mut self, ordering: &MoveOrdering, state: &BoardState) -> bool {
        if self.next_index.is_some() {
            return true;
        }
        while self.stage < 5 {
            if self.skip_quiets && (self.stage == 2 || self.stage == 3) {
                self.stage += 1;
                continue;
            }
            if self.stage == 3 && !self.quiet_scored {
                for index in self.cursor..self.count {
                    if self.stages[index] == 3 {
                        self.scores[index] = ordering.quiet_score(state, self.moves[index], self.previous);
                    }
                }
                self.quiet_scored = true;
            }
            let mut best: Option<usize> = None;
            for index in self.cursor..self.count {
                if self.stages[index] == self.stage && best.is_none_or(|b| self.scores[index] > self.scores[b]) {
                    best = Some(index);
                }
            }
            if best.is_some() {
                self.next_index = best;
                return true;
            }
            self.stage += 1;
        }
        false
    }

    pub fn next_move(&mut self, ordering: &MoveOrdering, state: &BoardState) -> Option<Move> {
        if !self.has_next(ordering, state) {
            return None;
        }
        let best = self.next_index.take().unwrap();
        let cursor = self.cursor;
        self.moves.swap(best, cursor);
        self.stages[best] = self.stages[cursor];
        self.stages[cursor] = self.stage;
        self.scores.swap(best, cursor);
        self.cursor += 1;
        Some(self.moves[cursor])
    }
}
