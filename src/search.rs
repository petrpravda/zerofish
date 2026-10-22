//! Negamax search over the classical evaluation.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;

use crate::board::BoardState;
use crate::evaluation;
use crate::limits::Limits;
use crate::moves::Move;

pub const INF: i32 = 999_999;
pub const MAX_PLY: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub mv: Option<Move>,
    pub score: i32,
}

pub fn is_score_checkmate(score: i32) -> bool {
    score.abs() >= INF - MAX_PLY as i32
}

#[derive(Default)]
pub struct Search {
    stop: Arc<AtomicBool>,
    limits: Limits,
}

impl Search {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn it_deep(&mut self, state: &BoardState, search_depth: i32, limits: Limits) -> SearchResult {
        assert!(search_depth >= 1, "depth must be positive");
        self.limits = limits;
        let legal = state.generate_legal_moves();
        let mut result = SearchResult {
            mv: legal.iter().next(),
            score: if legal.is_empty() && state.is_king_attacked() { -INF } else { 0 },
        };
        if legal.is_empty() {
            return result;
        }
        for depth in 1..=search_depth.min(MAX_PLY as i32 - 1) {
            if self.aborted() {
                break;
            }
            let completed = self.nega_max_root(state, depth, -INF, INF);
            if self.stop.load(Relaxed) {
                break;
            }
            result = completed;
            if self.limits.soft_expired() || result.mv.is_none() || is_score_checkmate(result.score) {
                break;
            }
        }
        result
    }

    fn nega_max_root(&mut self, state: &BoardState, depth: i32, mut alpha: i32, beta: i32) -> SearchResult {
        let mut best_move = None;
        let mut best_score = -INF;
        for mv in state.generate_legal_moves().iter() {
            let child = state.do_move(mv);
            let score = -self.nega_max(&child, depth - 1, 1, -beta, -alpha);
            if self.stop.load(Relaxed) {
                return SearchResult { mv: None, score: 0 };
            }
            if best_move.is_none() || score > best_score {
                best_move = Some(mv);
                best_score = score;
            }
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
        match best_move {
            Some(_) => SearchResult { mv: best_move, score: best_score },
            None => SearchResult { mv: None, score: if state.is_king_attacked() { -INF } else { 0 } },
        }
    }

    fn nega_max(&mut self, state: &BoardState, depth: i32, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        if self.aborted() {
            return 0;
        }
        let moves = state.generate_legal_moves();
        if moves.is_empty() {
            return if state.is_king_attacked() { -INF + ply as i32 } else { 0 };
        }
        if depth <= 0 || ply >= MAX_PLY {
            return self.evaluate(state);
        }
        let mut best_score = -INF;
        for mv in moves.iter() {
            let child = state.do_move(mv);
            let score = -self.nega_max(&child, depth - 1, ply + 1, -beta, -alpha);
            if self.stop.load(Relaxed) {
                return 0;
            }
            best_score = best_score.max(score);
            alpha = alpha.max(score);
            if alpha >= beta {
                break;
            }
        }
        best_score
    }

    fn evaluate(&self, state: &BoardState) -> i32 {
        evaluation::evaluate_state(state)
    }

    fn aborted(&mut self) -> bool {
        if self.limits.expired() {
            self.stop.store(true, Relaxed);
        }
        self.stop.load(Relaxed)
    }

    pub fn stop_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.stop)
    }

    pub fn stop(&self) {
        self.stop.store(true, Relaxed);
    }

    pub fn reset_stop(&self) {
        self.stop.store(false, Relaxed);
    }
}
