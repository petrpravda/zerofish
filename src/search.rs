//! Principal variation search with iterative deepening, aspiration windows, quiescence and selectivity.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::{Arc, LazyLock};

use crate::board::BoardState;
use crate::evaluation;
use crate::limits::Limits;
use crate::moves::{Move, MoveList};
use crate::ordering::{tactical, MoveOrdering, Picker};
use crate::position::{is_rule_draw, is_search_draw};
use crate::see::{see, value};
use crate::tt::{TTEntry, TranspositionTable, EVAL_ONLY, EXACT, LOWER_BOUND, UNKNOWN_EVAL, UPPER_BOUND};
use crate::types::*;

pub const INF: i32 = 999_999;
pub const MAX_PLY: usize = 128;

const NULL_MIN_DEPTH: i32 = 2;
const ASPIRATION_WINDOW: i32 = 25;
const TIME_CHECK_INTERVAL: i32 = 1024;
static LMR_TABLE: LazyLock<[[i32; 64]; 64]> = LazyLock::new(|| {
    let mut table = [[0; 64]; 64];
    for depth in 1..64 {
        for move_number in 1..64 {
            // float constants widened to double, as in the Java original
            table[depth][move_number] = (0.92f32 as f64 + (depth as f64).ln() * (move_number as f64).ln() / 3.09f32 as f64) as i32;
        }
    }
    table
});

#[inline(always)]
fn lmr(depth: i32, move_number: i32) -> i32 {
    LMR_TABLE[depth.clamp(0, 63) as usize][move_number.clamp(0, 63) as usize]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub mv: Option<Move>,
    pub score: i32,
}

#[derive(Default)]
struct Statistics {
    nodes: u64,
    qnodes: u64,
    tt_hits: u64,
    beta_cutoffs: u64,
    qbeta_cutoffs: u64,
    cumulative: u64,
}

impl Statistics {
    fn total_nodes(&self) -> u64 {
        self.nodes + self.qnodes
    }

    fn reset(&mut self) {
        let cumulative = self.cumulative;
        *self = Statistics { cumulative, ..Default::default() };
    }
}

#[derive(Default)]
struct SelectiveCounters {
    reverse_futility_cuts: u64,
    late_move_skips: u64,
    see_prunes: u64,
    null_attempts: u64,
    null_verifications: u64,
    singular_checks: u64,
    singular_extensions: u64,
    correction_updates: u64,
    razor_cuts: u64,
    futility_prunes: u64,
    history_prunes: u64,
    delta_prunes: u64,
    internal_reductions: u64,
}

pub struct Search {
    tt: Arc<TranspositionTable>,
    stats: Statistics,
    ordering: MoveOrdering,
    pickers: Vec<Picker>,
    contexts: [i32; MAX_PLY + 2],
    static_evals: [i32; MAX_PLY + 2],
    extensions: [i32; MAX_PLY + 2],
    verification: [bool; MAX_PLY + 2],
    cut_nodes: [bool; MAX_PLY + 2],
    excluded_moves: [Option<Move>; MAX_PLY + 2],
    pub use_pvs: bool,
    pub use_selective: bool,
    counters: SelectiveCounters,
    root_node_share: f64,
    time_check_countdown: i32,
    completed_depth: i32,
    completed_result: Option<SearchResult>,
    stop: Arc<AtomicBool>,
    sel_depth: i32,
    limits: Limits,
    verbose: bool,
    // position keys of the game and the current search path, the current node last
    keys: Vec<u64>,
}

pub fn is_score_checkmate(score: i32) -> bool {
    score.abs() >= INF - MAX_PLY as i32
}

pub fn score_to_table(score: i32, ply: usize) -> i32 {
    if is_score_checkmate(score) {
        score + if score > 0 { ply as i32 } else { -(ply as i32) }
    } else {
        score
    }
}

pub fn score_from_table(score: i32, ply: usize) -> i32 {
    if is_score_checkmate(score) {
        score - if score > 0 { ply as i32 } else { -(ply as i32) }
    } else {
        score
    }
}

fn safe_pruning_window(in_check: bool, alpha: i32, beta: i32) -> bool {
    !in_check && beta - alpha == 1 && !is_score_checkmate(alpha) && !is_score_checkmate(beta)
}

fn non_pawn_count(state: &BoardState) -> u32 {
    let side = state.side_to_play();
    (state.orthogonal_sliders(side) | state.bitboard_of_side(side, BISHOP) | state.bitboard_of_side(side, KNIGHT)).count_ones()
}

fn advanced_pawn(state: &BoardState, mv: Move) -> bool {
    let rank = mv.to() >> 3;
    state.piece_type_at(mv.from()) == PAWN && (if state.side_to_play() == 0 { rank } else { 7 - rank }) >= 5
}

impl Search {
    pub fn new(tt: Arc<TranspositionTable>, verbose: bool, ordering: MoveOrdering) -> Self {
        Search {
            tt,
            stats: Statistics::default(),
            ordering,
            pickers: (0..MAX_PLY + 2).map(|_| Picker::new()).collect(),
            contexts: [-1; MAX_PLY + 2],
            static_evals: [UNKNOWN_EVAL; MAX_PLY + 2],
            extensions: [0; MAX_PLY + 2],
            verification: [false; MAX_PLY + 2],
            cut_nodes: [false; MAX_PLY + 2],
            excluded_moves: [None; MAX_PLY + 2],
            use_pvs: true,
            use_selective: true,
            counters: SelectiveCounters::default(),
            root_node_share: 0.5,
            time_check_countdown: 0,
            completed_depth: 0,
            completed_result: None,
            stop: Arc::new(AtomicBool::new(false)),
            sel_depth: 0,
            limits: Limits::infinite(),
            verbose,
            keys: Vec::new(),
        }
    }

    /// A single-threaded searcher with classical evaluation and silent output.
    pub fn classical(tt: Arc<TranspositionTable>) -> Self {
        Self::new(tt, false, MoveOrdering::new())
    }

    pub fn set_table(&mut self, tt: Arc<TranspositionTable>) {
        self.tt = tt;
    }

    /// `keys` are the position keys of the game, ending with the key of `state`.
    pub fn it_deep(&mut self, state: &BoardState, keys: &[u64], search_depth: i32, limits: Limits) -> SearchResult {
        self.limits = limits;
        self.tt.new_search();
        self.completed_depth = 0;
        self.completed_result = None;
        self.stats.cumulative = 0;
        self.contexts.fill(-1);
        self.static_evals.fill(UNKNOWN_EVAL);
        self.extensions.fill(0);
        self.verification.fill(false);
        self.cut_nodes.fill(false);
        self.excluded_moves.fill(None);
        self.time_check_countdown = 0;
        self.sel_depth = 0;
        self.keys.clear();
        self.keys.extend_from_slice(keys);
        let mut alpha = -INF;
        let mut beta = INF;
        let mut depth = 1;
        let mut window = ASPIRATION_WINDOW;
        let legal_moves = state.generate_legal_moves();
        let mut result = SearchResult {
            mv: legal_moves.iter().next(),
            score: if legal_moves.is_empty() && state.is_king_attacked() { -INF } else { 0 },
        };
        if legal_moves.is_empty() || is_rule_draw(state, keys) {
            return result;
        }

        while depth <= search_depth.min(MAX_PLY as i32 - 1) && !self.aborted() {
            let iteration = self.nega_max_root(state, depth, alpha, beta);
            if self.stop.load(Relaxed) {
                break;
            }
            if iteration.score <= alpha {
                alpha = (iteration.score - window).max(-INF);
                window = (window * 2).min(INF);
            } else if iteration.score >= beta {
                beta = (iteration.score + window).min(INF);
                window = (window * 2).min(INF);
            } else {
                result = iteration;
                self.completed_depth = depth;
                self.completed_result = Some(result);
                self.print_info(state, result, depth);
                depth += 1;
                window = ASPIRATION_WINDOW;
                alpha = if depth < 4 { -INF } else { (result.score - window).max(-INF) };
                beta = if depth < 4 { INF } else { (result.score + window).min(INF) };
                self.stats.reset();
                if self.limits.soft_expired() || is_score_checkmate(result.score) {
                    break;
                }
            }
        }
        result
    }

    fn nega_max_root(&mut self, state: &BoardState, depth: i32, mut alpha: i32, beta: i32) -> SearchResult {
        let moves = state.generate_legal_moves();
        if moves.is_empty() {
            return SearchResult { mv: None, score: if state.is_king_attacked() { -INF } else { 0 } };
        }
        let original_alpha = alpha;
        let mut best_score = -INF;
        let mut best_move: Option<Move> = None;
        let entry = self.tt.probe(state.transposition_key());
        self.pickers[0].reset(&self.ordering, state, Some(&moves), entry, 0, -1, false);
        let mut move_index = 0;
        let mut total_work = 0u64;
        let mut best_work = 0u64;
        while let Some(mv) = self.pickers[0].next_move(&self.ordering, state) {
            let before = self.stats.total_nodes();
            let child = self.child(state, mv, 0);
            let index = move_index;
            move_index += 1;
            let mut value;
            if !self.use_pvs || index == 0 {
                value = self.search_child(&child, depth - 1, 1, -beta, -alpha, false, true);
            } else {
                let reduction = if self.use_selective && depth >= 3 && index >= 4 && !tactical(mv) && !state.is_king_attacked() && !child.is_king_attacked() {
                    (lmr(depth, index + 1) - 1).clamp(0, depth - 2)
                } else {
                    0
                };
                value = self.search_child(&child, depth - 1 - reduction, 1, -alpha - 1, -alpha, true, true);
                if reduction > 0 && value > alpha && !self.aborted() {
                    value = self.search_child(&child, depth - 1, 1, -alpha - 1, -alpha, false, true);
                }
                if value > alpha && value < beta && !self.aborted() {
                    value = self.search_child(&child, depth - 1, 1, -beta, -alpha, false, true);
                }
            }
            if self.stop.load(Relaxed) {
                return SearchResult { mv: None, score: 0 };
            }
            let work = (self.stats.total_nodes() - before).max(1);
            total_work += work;
            if best_move.is_none() || value > best_score {
                best_move = Some(mv);
                best_score = value;
                best_work = work;
            }
            alpha = alpha.max(value);
            if alpha >= beta {
                break;
            }
        }
        let flag = if best_score >= beta {
            LOWER_BOUND
        } else if best_score <= original_alpha {
            UPPER_BOUND
        } else {
            EXACT
        };
        self.root_node_share = if total_work == 0 { 0.5 } else { best_work as f64 / total_work as f64 };
        self.tt.set(state.transposition_key(), best_score, depth, flag, best_move.unwrap_or(Move::NULL_MOVE), UNKNOWN_EVAL);
        SearchResult { mv: best_move, score: best_score }
    }

    fn nega_max(&mut self, state: &BoardState, mut depth: i32, ply: usize, mut alpha: i32, beta: i32, can_apply_null: bool) -> i32 {
        if self.aborted() {
            return 0;
        }
        if depth <= 0 {
            return self.quiescence(state, ply, alpha, beta);
        }
        self.stats.nodes += 1;
        self.stats.cumulative += 1;
        self.sel_depth = self.sel_depth.max(ply as i32);
        let in_check = state.is_king_attacked();
        if is_search_draw(state, &self.keys) {
            return 0;
        }
        if ply >= MAX_PLY {
            return if in_check { 0 } else { self.clocked_evaluation(state, ply) };
        }
        let original_alpha = alpha;
        let pv = beta - alpha > 1;
        let cut_node = self.cut_nodes[ply] && !pv;
        let excluded = self.excluded_moves[ply];
        let selective = self.use_selective && !self.verification[ply];
        let entry = self.tt.probe(state.transposition_key());
        if let Some(entry) = entry {
            if excluded.is_none() && !self.verification[ply] && entry.depth >= depth {
                let score = score_from_table(entry.score, ply);
                if entry.flag == EXACT || entry.flag == LOWER_BOUND && score >= beta || entry.flag == UPPER_BOUND && score <= alpha {
                    self.stats.tt_hits += 1;
                    return score;
                }
            }
        }
        let raw_eval = if in_check { UNKNOWN_EVAL } else { self.static_evaluation(state, ply, entry) };
        let base_eval = if in_check { raw_eval } else { self.clocked(state, raw_eval) };
        let static_eval = if in_check { raw_eval } else { base_eval };
        self.static_evals[ply] = static_eval;
        let improving = !in_check && ply >= 2 && self.static_evals[ply - 2] != UNKNOWN_EVAL && static_eval > self.static_evals[ply - 2];
        let pruning = selective && state.has_non_pawn_material(state.side_to_play()) && safe_pruning_window(in_check, alpha, beta);
        let tt_move = entry.map(|e| e.mv).filter(|mv| !mv.is_null());
        if selective && excluded.is_none() && (pv || cut_node) && depth >= 4 && tt_move.is_none() {
            self.counters.internal_reductions += 1;
            depth -= 1;
        }
        let margin = depth * 140 - (if improving { 60 } else { 0 });
        if pruning && depth <= 8 && !is_score_checkmate(static_eval) && static_eval - margin >= beta {
            self.counters.reverse_futility_cuts += 1;
            return beta + (static_eval - beta) / 3;
        }
        if pruning && depth <= 3 && static_eval + 400 * depth < alpha {
            let score = self.quiescence(state, ply, alpha, alpha + 1);
            if self.aborted() {
                return 0;
            }
            if score < alpha {
                self.counters.razor_cuts += 1;
                return score;
            }
        }
        let non_pawns = non_pawn_count(state);
        if pruning
            && can_apply_null
            && depth >= NULL_MIN_DEPTH
            && non_pawns >= 2
            && state.half_move_clock < 90
            && static_eval >= beta
            && entry.is_none_or(|e| cut_node || e.flag == EVAL_ONLY || score_from_table(e.score, ply) >= beta)
        {
            let reduction = depth.min(3 + depth / 3 + ((static_eval - beta) / 200).min(3));
            self.counters.null_attempts += 1;
            let null_state = state.do_null_move();
            self.contexts[ply + 1] = -1;
            self.verification[ply + 1] = false;
            self.extensions[ply + 1] = self.extensions[ply];
            let mut score = self.search_child(&null_state, depth - reduction, ply + 1, -beta, -beta + 1, !cut_node, false);
            if self.aborted() {
                return 0;
            }
            if score >= beta && !is_score_checkmate(score) {
                if depth >= 14 || non_pawns <= 2 {
                    self.counters.null_verifications += 1;
                    self.verification[ply] = true;
                    score = self.nega_max(state, depth - reduction, ply, beta - 1, beta, false);
                    self.verification[ply] = false;
                    if self.aborted() {
                        return 0;
                    }
                }
                if score >= beta {
                    return score;
                }
            }
        }

        let mut best_score = -INF;
        let mut best_move = Move::NULL_MOVE;
        let mut move_index: i32 = 0;
        let mut searched = 0;
        let mut legal = 0;
        if in_check {
            let mut evasions = MoveList::new();
            state.generate_legal_moves_into(&mut evasions);
            self.pickers[ply].reset(&self.ordering, state, Some(&evasions), entry, ply, self.contexts[ply], false);
        } else {
            self.pickers[ply].reset(&self.ordering, state, None, entry, ply, self.contexts[ply], false);
        }
        while let Some(mv) = self.pickers[ply].next_move(&self.ordering, state) {
            let stage = self.pickers[ply].last_stage();
            let pick_score = self.pickers[ply].last_score();
            let original_index = move_index;
            legal += 1;
            if Some(mv) == excluded {
                continue;
            }
            let is_tactical = tactical(mv);
            let protected_move = tt_move == Some(mv) || mv.is_promotion() || advanced_pawn(state, mv);
            let history = if is_tactical {
                0
            } else if stage == 3 {
                pick_score
            } else {
                self.ordering.quiet_score(state, mv, self.contexts[ply])
            };
            if pruning && searched > 0 && !protected_move && !is_score_checkmate(best_score) {
                let lmr_depth = (depth - 1 - lmr(depth, move_index + 1)).max(0);
                let late_move_limit = if improving { 4 + 5 * depth } else { 3 + 4 * depth };
                if !is_tactical && depth <= 8 && move_index >= late_move_limit {
                    self.counters.late_move_skips += 1;
                    self.pickers[ply].skip_quiets();
                    continue;
                }
                let futility_base = 140;
                if !is_tactical && lmr_depth <= 8 && static_eval + futility_base + 76 * lmr_depth + history / 87 - 3 * searched <= alpha {
                    self.counters.futility_prunes += 1;
                    self.pickers[ply].skip_quiets();
                    continue;
                }
                if !is_tactical && lmr_depth <= 5 && history < -2048 * depth - 1024 {
                    self.counters.history_prunes += 1;
                    self.pickers[ply].skip_quiets();
                    continue;
                }
            }
            let child = self.child(state, mv, ply);
            let gives_check = child.is_king_attacked();
            let extension = i32::from(self.extensions[ply] < 8 && in_check);
            self.extensions[ply + 1] += extension;
            let full_depth = depth - 1 + extension;
            let mut reduction = 0;
            let reduce_from = if pv { 3 } else { 2 };
            if selective && !in_check && depth >= 2 && original_index >= reduce_from && !(pv && is_tactical && stage <= 1) {
                let mut r = lmr(depth, original_index + 1) * 1024;
                r -= if pv { 1264 } else { 0 };
                r += if cut_node { 1910 } else { 0 };
                r += if improving { 0 } else { 275 };
                r -= if gives_check { 1024 } else { 0 };
                r -= history * 1024 / 3419;
                r += if static_eval + 101 + 80 * depth + history / 71 <= alpha { 966 } else { 0 };
                reduction = (r / 1024).clamp(0, (full_depth - 1).max(0));
            }
            move_index += 1;
            let probe = self.use_pvs && searched > 0;
            let probe_alpha = if probe { -alpha - 1 } else { -beta };
            let mut score;
            if reduction > 0 {
                score = self.search_child(&child, full_depth - reduction, ply + 1, probe_alpha, -alpha, true, true);
                if score > alpha && !self.aborted() {
                    let new_depth = full_depth + i32::from(score > best_score + 37 + 5 * full_depth) - i32::from(score < best_score + full_depth);
                    score = self.search_child(&child, new_depth, ply + 1, probe_alpha, -alpha, !cut_node, true);
                }
            } else {
                score = self.search_child(&child, full_depth, ply + 1, probe_alpha, -alpha, !(pv && searched == 0) && !cut_node, true);
            }
            if probe && pv && score > alpha && score < beta && !self.aborted() {
                score = self.search_child(&child, full_depth, ply + 1, -beta, -alpha, false, true);
            }
            if self.aborted() {
                return 0;
            }
            searched += 1;
            if score > best_score {
                best_score = score;
                best_move = mv;
            }
            alpha = alpha.max(score);
            if alpha >= beta {
                self.stats.beta_cutoffs += 1;
                if !self.verification[ply] {
                    self.ordering.cutoff(state, mv, ply, self.contexts[ply], depth, &self.pickers[ply]);
                }
                break;
            }
        }
        if legal == 0 {
            return if in_check { -INF + ply as i32 } else { 0 };
        }
        if searched == 0 {
            return if excluded.is_some() { -INF + ply as i32 } else { alpha };
        }
        let flag = if best_score >= beta {
            LOWER_BOUND
        } else if best_score <= original_alpha {
            UPPER_BOUND
        } else {
            EXACT
        };
        if !self.verification[ply] {
            self.tt.set(state.transposition_key(), score_to_table(best_score, ply), depth, flag, best_move, raw_eval);
        }
        best_score
    }

    fn quiescence(&mut self, state: &BoardState, ply: usize, mut alpha: i32, beta: i32) -> i32 {
        if self.aborted() {
            return 0;
        }
        self.sel_depth = self.sel_depth.max(ply as i32);
        self.stats.qnodes += 1;
        self.stats.cumulative += 1;
        let in_check = state.is_king_attacked();
        let mut evasions = MoveList::new();
        if in_check {
            state.generate_legal_moves_into(&mut evasions);
            if evasions.is_empty() {
                return -INF + ply as i32;
            }
        }
        if is_search_draw(state, &self.keys) {
            return 0;
        }
        if ply >= MAX_PLY {
            return if in_check { 0 } else { self.clocked_evaluation(state, ply) };
        }
        let entry = self.tt.probe(state.transposition_key());
        if let Some(entry) = entry {
            if !self.verification[ply] && entry.flag != EVAL_ONLY {
                let score = score_from_table(entry.score, ply);
                if entry.flag == EXACT || entry.flag == LOWER_BOUND && score >= beta || entry.flag == UPPER_BOUND && score <= alpha {
                    self.stats.tt_hits += 1;
                    return score;
                }
            }
        }
        let original_alpha = alpha;
        let mut raw_eval = UNKNOWN_EVAL;
        let mut stand_pat = -INF;
        let mut best_score = -INF;
        let mut best_move = Move::NULL_MOVE;
        if !in_check {
            raw_eval = self.static_evaluation(state, ply, entry);
            let base_eval = self.clocked(state, raw_eval);
            stand_pat = base_eval;
            if stand_pat >= beta {
                self.store_quiescence(state, ply, stand_pat, LOWER_BOUND, best_move, raw_eval, entry);
                return stand_pat;
            }
            best_score = stand_pat;
            alpha = alpha.max(stand_pat);
        }
        if in_check {
            self.pickers[ply].reset(&self.ordering, state, Some(&evasions), entry, ply, self.contexts[ply], false);
        } else {
            self.pickers[ply].reset(&self.ordering, state, None, entry, ply, self.contexts[ply], true);
        }
        let mut searched = 0;
        while let Some(mv) = self.pickers[ply].next_move(&self.ordering, state) {
            let is_tactical = tactical(mv);
            if !in_check && !is_tactical {
                continue;
            }
            let child = self.child(state, mv, ply);
            searched += 1;
            self.keys.push(child.hash());
            let value = -self.quiescence(&child, ply + 1, -beta, -alpha);
            self.keys.pop();

            if self.stop.load(Relaxed) {
                return 0;
            }
            if value > best_score {
                best_score = value;
                if value > alpha {
                    best_move = mv;
                    if value >= beta {
                        self.stats.qbeta_cutoffs += 1;
                        self.store_quiescence(state, ply, value, LOWER_BOUND, best_move, raw_eval, entry);
                        return value;
                    }
                    alpha = value;
                }
            }
        }
        let flag = if best_score > original_alpha { EXACT } else { UPPER_BOUND };
        self.store_quiescence(state, ply, best_score, flag, best_move, raw_eval, entry);
        best_score
    }

    #[allow(clippy::too_many_arguments)]
    fn store_quiescence(&self, state: &BoardState, ply: usize, score: i32, flag: i32, mv: Move, raw_eval: i32, entry: Option<TTEntry>) {
        if self.verification[ply] || entry.is_some_and(|e| e.depth > 0) {
            return;
        }
        self.tt.set(state.transposition_key(), score_to_table(score, ply), 0, flag, mv, raw_eval);
    }

    #[allow(clippy::too_many_arguments)]
    fn search_child(&mut self, child: &BoardState, depth: i32, ply: usize, alpha: i32, beta: i32, cut_node: bool, allow_null: bool) -> i32 {
        self.cut_nodes[ply] = cut_node;
        self.keys.push(child.hash());
        let score = -self.nega_max(child, depth, ply, alpha, beta, allow_null);
        self.keys.pop();
        score
    }

    fn child(&mut self, state: &BoardState, mv: Move, ply: usize) -> BoardState {
        let child = state.do_move(mv);
        self.contexts[ply + 1] = (state.piece_at(mv.from()) * 64 + mv.to()) as i32;
        self.extensions[ply + 1] = self.extensions[ply];
        self.verification[ply + 1] = false;
        child
    }

    fn evaluate(&mut self, state: &BoardState, _ply: usize) -> i32 {
        evaluation::evaluate_state(state)
    }

    fn clocked(&self, _state: &BoardState, evaluation: i32) -> i32 {
        evaluation
    }

    fn clocked_evaluation(&mut self, state: &BoardState, ply: usize) -> i32 {
        let evaluation = self.evaluate(state, ply);
        self.clocked(state, evaluation)
    }

    fn static_evaluation(&mut self, state: &BoardState, ply: usize, entry: Option<TTEntry>) -> i32 {
        if let Some(entry) = entry.filter(|e| e.static_eval != UNKNOWN_EVAL) {
            return entry.static_eval;
        }
        let evaluation = self.evaluate(state, ply);
        self.tt.set_static_eval(state.transposition_key(), evaluation);
        evaluation
    }

    fn aborted(&mut self) -> bool {
        if self.stop.load(Relaxed) {
            return true;
        }
        self.time_check_countdown -= 1;
        if self.time_check_countdown > 0 {
            return false;
        }
        self.time_check_countdown = TIME_CHECK_INTERVAL;
        if self.limits.expired() {
            self.stop.store(true, Relaxed);
        }
        self.stop.load(Relaxed)
    }

    /// The principal variation as stored in the transposition table.
    pub fn get_pv(&self, state: &BoardState, depth: i32) -> String {
        let mut moves = Vec::new();
        let mut current = state.clone();
        for _ in 0..depth.clamp(0, MAX_PLY as i32) {
            let Some(entry) = self.tt.probe(current.transposition_key()) else { break };
            if !current.generate_legal_moves().contains(entry.mv) {
                break;
            }
            moves.push(entry.mv.uci());
            current = current.do_move(entry.mv);
        }
        moves.join(" ")
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

    pub fn reset_histories(&mut self) {
        self.ordering.reset();
    }

    pub fn completed_depth(&self) -> i32 {
        self.completed_depth
    }

    pub fn completed_result(&self) -> Option<SearchResult> {
        self.completed_result
    }

    pub fn searched_nodes(&self) -> u64 {
        self.stats.cumulative
    }

    pub fn selective_summary(&self) -> String {
        let c = &self.counters;
        format!(
            "p2 rfp={} lmp={} see={} null={} nullverify={} singularchecks={} singular={} correction={} razor={} futility={} history={} delta={} iir={}",
            c.reverse_futility_cuts,
            c.late_move_skips,
            c.see_prunes,
            c.null_attempts,
            c.null_verifications,
            c.singular_checks,
            c.singular_extensions,
            c.correction_updates,
            c.razor_cuts,
            c.futility_prunes,
            c.history_prunes,
            c.delta_prunes,
            c.internal_reductions
        )
    }

    fn print_info(&self, state: &BoardState, result: SearchResult, depth: i32) {
        if !self.verbose {
            return;
        }
        let elapsed = self.limits.elapsed_nanos();
        let score = if is_score_checkmate(result.score) {
            format!("mate {}", result.score.signum() * ((INF - result.score.abs() + 1) / 2))
        } else {
            format!("cp {}", result.score)
        };
        let nodes = self.stats.total_nodes();
        println!(
            "info currmove {} depth {} seldepth {} time {} score {} nodes {} nps {:.0} pv {}",
            result.mv.map_or("(none)".to_string(), |m| m.to_string()),
            depth,
            self.sel_depth,
            elapsed / 1_000_000,
            score,
            nodes,
            nodes as f64 / (elapsed.max(1) as f64 / 1e9),
            self.get_pv(state, depth)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::BoardPosition;

    fn search_position(fen: &str, depth: i32) -> SearchResult {
        let position = BoardPosition::from_fen(fen).unwrap();
        let mut search = Search::classical(Arc::new(TranspositionTable::default()));
        search.it_deep(position.state(), position.keys(), depth, Limits::infinite())
    }

    #[test]
    fn finds_mate_in_one() {
        let result = search_position("6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1", 4);
        assert_eq!(result.mv.unwrap().uci(), "d1d8");
        assert!(is_score_checkmate(result.score) && result.score > 0);
    }

    #[test]
    fn captures_hanging_queen() {
        let result = search_position("4k3/8/8/3q4/8/8/8/3RK3 w - - 0 1", 5);
        assert_eq!(result.mv.unwrap().uci(), "d1d5");
    }

    #[test]
    fn reports_mate_and_stalemate() {
        assert_eq!(search_position("7k/6Q1/6K1/8/8/8/8/8 b - - 0 1", 3), SearchResult { mv: None, score: -INF });
        assert_eq!(search_position("7k/8/6QK/8/8/8/8/8 b - - 0 1", 3), SearchResult { mv: None, score: 0 });
    }

}
