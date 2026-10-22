//! Negamax search over the classical evaluation.

use crate::board::BoardState;
use crate::evaluation;
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
}

impl Search {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn it_deep(&mut self, state: &BoardState, search_depth: i32) -> SearchResult {
        assert!(search_depth >= 1, "depth must be positive");
        self.nega_max_root(state, search_depth, -INF, INF)
    }

    fn nega_max_root(&mut self, state: &BoardState, depth: i32, alpha: i32, beta: i32) -> SearchResult {
        let mut best_move = None;
        let mut best_score = -INF;
        for mv in state.generate_legal_moves().iter() {
            let child = state.do_move(mv);
            let score = -self.nega_max(&child, depth - 1, 1, -beta, -alpha);
            if best_move.is_none() || score > best_score {
                best_move = Some(mv);
                best_score = score;
            }
        }
        match best_move {
            Some(_) => SearchResult { mv: best_move, score: best_score },
            None => SearchResult { mv: None, score: if state.is_king_attacked() { -INF } else { 0 } },
        }
    }

    fn nega_max(&mut self, state: &BoardState, depth: i32, ply: usize, alpha: i32, beta: i32) -> i32 {
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
            best_score = best_score.max(score);
        }
        best_score
    }

    fn evaluate(&self, state: &BoardState) -> i32 {
        evaluation::evaluate_state(state)
    }
}
