use lazy_static::lazy_static;
use crate::board_position::BoardPosition;
use crate::board_state::BoardState;
use crate::evaluation::Evaluation;
use crate::fen::START_POS;
use crate::r#move::{Move};
use crate::side::Side;
use crate::statistics::Statistics;
use crate::time::Instant;
use crate::transposition::{Depth, TranspositionTable, Value};

#[derive(Debug, Copy, Clone)]
pub struct SearchResult {
    pub moov: Option<Move>,
    pub score: Value,
    pub stop_it_deep: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

lazy_static! {
    pub static ref LMR_TABLE: [[i32; 64]; 64] = prepare_lmr_table();
}

fn prepare_lmr_table() -> [[i32; 64]; 64] {
    let mut result = [[0i32; 64]; 64];
    for depth in 1..64 {
        for move_number in 1..64 {
            result[depth][move_number] = (0.75f32 + (depth as f32).ln() * (move_number as f32).ln() / 2.25f32) as i32;
        }
    }
    result
}

#[derive(Debug)]
pub struct SearchLimit {
    pub depth: Depth,
    pub max_nodes: u32,
    pub max_ms: u32,
}

impl SearchLimit {
    fn default() -> SearchLimit {
        SearchLimit {
            depth: 3,
            max_nodes: u32::MAX,
            max_ms: u32::MAX,
        }
    }
}

pub struct SearchLimitParams {
    pub depth: Option<Depth>,
    pub max_nodes: Option<u32>,
    pub move_time: Option<u32>,
    pub moves_to_go: Option<u32>,
    pub w_time: Option<u32>,
    pub b_time: Option<u32>,

    pub perft_depth: Option<Depth>,
}

impl SearchLimitParams {
    pub fn prepare(&self, side_on_the_move: Side) -> SearchLimit {
        let mut result = SearchLimit {
            depth: self.depth.unwrap_or(u8::MAX - 1),
            max_nodes: self.max_nodes.unwrap_or(u32::MAX - 1),
            max_ms: self.move_time.unwrap_or(u32::MAX - 1),
        };
        if self.w_time.is_some() && self.b_time.is_some() && self.moves_to_go.is_some() {
            let time = match side_on_the_move { Side::WHITE => self.w_time.unwrap(),
                Side::BLACK => self.b_time.unwrap() };
            result.max_ms = time / self.moves_to_go.unwrap();
        }
        result
    }
}

impl SearchLimitParams {
    // fn new() -> SearchLimitParams {
    //     Self { depth: None, max_nodes: None, perft_depth: None, moves_to_go: None,
    //         w_time: None, b_time: None, move_time: None }
    // }
}

pub struct Search {
    search_position: BoardPosition, // TODO rename to board_position
    sel_depth: Depth,
    pub(crate) stopped: bool,
    statistics: Statistics,
    transposition_table: &'static TranspositionTable,
    start_time: Instant,
    search_limit: SearchLimit,
    time_checking_round: u32,
}

lazy_static! {
    pub static ref TT: TranspositionTable = TranspositionTable::new(1);
}

impl Search {

    pub const INF: Value = 29999;
    pub const NULL_MIN_DEPTH: Depth = 2;

    const LMR_MIN_DEPTH: Depth = 2;
    const LMR_MOVES_WO_REDUCTION: usize = 1; // TODO which type?
    const ASPIRATION_WINDOW: Value = 25;

    pub fn new() -> Self {
        Self {
            search_position: BoardPosition::from_fen(START_POS),
            start_time: Instant::now(),
            sel_depth: 10,
            stopped: false,
            statistics: Statistics::new(),
            transposition_table: &TT,
            search_limit: SearchLimit::default(),
            time_checking_round: 0,
        }
    }

    pub fn it_deep(&mut self, position: &BoardPosition, search_limit: SearchLimit) -> SearchResult {
        let mut best_result = SearchResult { moov: None, score: 0, stop_it_deep: false };

        self.search_limit = search_limit;
        self.search_position = position.for_search_depth(self.search_limit.depth);
        self.start_time = Instant::now();
        self.sel_depth = 0;
        self.stopped = false;
        let mut alpha: Value = -Search::INF;
        let mut beta: Value = Search::INF;
        let mut depth: Depth = 1;

        // Deepen until end conditions
        while depth <= self.search_limit.depth && !self.stopped {

            // Check to see if the time has ended
            //long elapsed = System.currentTimeMillis() - Limits.startTime;
//            if (stop || elapsed >= Limits.timeAllocated / 2 || isScoreCheckmate(result.score()))
//                break;


            let result_from_ply = self.nega_max_root(&position.state, depth, alpha, beta);
            if !self.stopped {
                let score = result_from_ply.score;
                if score <= alpha {
                    // Failed low, adjust window
                    alpha = -Search::INF;
                } else if score >= beta {
                    // Failed high, adjust window
                    beta = Search::INF;
                } else {
                    // Adjust the window around the new score and increase the depth
                    self.print_info_line(&position.state, &result_from_ply, depth);
                    best_result = result_from_ply;
                    alpha = score - Search::ASPIRATION_WINDOW;
                    beta = score + Search::ASPIRATION_WINDOW;
                    depth += 1;
                    self.statistics.reset();
                }
            }

            if best_result.stop_it_deep {
                break;
            }
        }

        return best_result;
    }

    pub fn nega_max_root(&mut self, state: &BoardState, depth: Depth, mut alpha: Value, beta: Value) -> SearchResult{
        //let mut value = -Search::INF;
        let moves = state.generate_legal_moves();
        // let inCheck = state.checkers() != 0;
        // if (inCheck) ++depth;
        // if moves.len() == 1 {
        //     return SearchResult{ moov: moves.moves.get(0).copied(), score: 0 }; // new SearchResult(Optional.of(moves.get(0)), 0);
        // }

        let mut best_move: Option<Move> = None;
        // self.score_moves(state, moves, 0);
        for moov in moves.over_sorted(&state, self.transposition_table) {
            // let uciText = moov.uci();
            // if uciText.eq("b1c3") {
            //     println!("{}", moov.uci());
            // }


            // let uci_string = moov.uci();
            // let start_fen = state.to_fen();
            // let uci_best_move = best_move.map(|m| m.uci());
            let new_state = state.do_move(&moov);
            let value = -self.nega_max(&new_state, depth - 1, 1, -beta, -alpha, true);
            if self.stopped {
                break;
            }

            //println!("{} {}", uci_string, value);
            // if (stop || Limits.checkLimits()) {
            //     stop = true;
            //     break;
            // }
            if value > alpha {
                best_move = Some(moov.clone());
                if value >= beta {
                    self.transposition_table.insert(&state, depth, beta, moov, Bound::Lower);
                    //set(state.hash, beta, depth, Bound::Lower, best_move);
                    return SearchResult{ moov: best_move, score: beta, stop_it_deep: false };
                }
                alpha = value;
                self.transposition_table.insert(&state, depth, alpha, moov, Bound::Upper);
            }
        }

       // if moves.len() == 1 {
       //     println!("TADY JSEM");
       //     // bestMove = moves.get(0);
       //     // transpositionTable.set(state.hash(), alpha, depth, TTEntry.EXACT, bestMove);
       // }
       //

        SearchResult{ moov: best_move, score: alpha, stop_it_deep: moves.len() <= 1 }
    }

    pub fn nega_max(&mut self, state: &BoardState, depth: Depth, ply: u16, mut alpha: Value, mut beta: Value, can_apply_null: bool) -> Value {
        let mate_value = Search::INF - ply as Value;
        //let mut in_check = false;
        let mut tt_flag = Bound::Upper;
        // let mut reducedDepth = 0; // TODO is really needed?

        if self.check_stopping() {
            return 0;
        }
        // if (stop || Limits.checkLimits()) {
        //     stop = true;
        //     return 0;
        // }

        // // MATE DISTANCE PRUNING - TODO is this working at all?
        // if alpha < -mate_value {
        //     alpha = -mate_value;
        // }
        // if beta > mate_value - 1 {
        //     beta = mate_value - 1;
        // }
        if alpha >= beta {
            self.statistics.increment_leafs();
            return alpha;
        }

        let in_check = state.is_king_attacked();
        if depth <= 0 && !in_check {
            let q_value = self.quiescence(state, depth as i32, ply as Depth, alpha, beta);
            return q_value
        }
        self.statistics.increment_nodes();

        if state.is_repetition_or_fifty(&self.search_position) {
            self.statistics.increment_leafs();
            return 0;
        }

        // PROBE TTABLE
        let tt_entry = self.transposition_table.probe(state);
        if tt_entry.is_some() && tt_entry.unwrap().depth() >= depth {
            let tt_entry_some = tt_entry.unwrap();
            self.statistics.increase_tthits();
            match tt_entry_some.flag() {
                Bound::Exact => {
                    self.statistics.increment_leafs();
                    return tt_entry_some.value();
                }
                Bound::Lower => {
                    alpha = alpha.max(tt_entry_some.value());
                }
                Bound::Upper => {
                    beta = beta.max(tt_entry_some.value());
                }
            }
            if alpha >= beta {
                self.statistics.increment_leafs();
                return tt_entry_some.value();
            }
        }

        // NULL MOVE
        if Search::can_apply_null_window(state, depth, beta, in_check, can_apply_null) {
            let r: i32 = if depth > 6 { 3 } else { 2 };
            let new_state = state.do_null_move();
            // TODO check depth
            let depth_null_search = (depth as i32 - r as i32 - 1).max(0i32) as Depth;
            let value = -self.nega_max(&new_state, depth_null_search, ply, -beta, -beta + 1, false);
            // if (stop) {
            //     return 0;
            // }
            if value >= beta {
                self.statistics.increase_beta_cutoffs();
                return beta;
            }
        }

        let moves = state.generate_legal_moves();
        let mut best_move: Move = Move::NULL_MOVE;
        for (index, moov) in moves.over_sorted(&state, self.transposition_table).enumerate() {

            // LATE MOVE REDUCTION
            let mut reduced_depth = depth;
            if self.can_apply_lmr(depth, &moov, index) {
                reduced_depth = reduced_depth - LMR_TABLE[depth.min(63) as usize][index.min(63) as usize] as u8;
            }

            if in_check {
                reduced_depth += 1;
            }

            // let state_out = state.to_string();
            // let uci_move = moov.uci();
            let new_state = state.do_move(&moov);
            let value = -self.nega_max(&new_state, reduced_depth - 1, ply + 1, -beta, -alpha, true);
            if self.stopped {
                return 0;
            }

            if value > alpha {
                best_move = moov;
                if value >= beta {
                    // if moov.flags() == Move::QUIET {
                    //     //MoveOrder.addKiller(state, move, ply);
                    //     //MoveOrder.addHistory(move, depth);
                    // }
                    self.statistics.increase_beta_cutoffs();
                    tt_flag = Bound::Lower;
                    alpha = beta;
                    break;
                }
                tt_flag = Bound::Exact;
                alpha = value;
            }
        }

        // Check if we are in checkmate or stalemate.
        if moves.len() == 0 {
            if in_check {
                alpha = -mate_value;
            } else {
                alpha = 0;
            }
        }

        if best_move.flags() != Move::NULL { // TODO && !stop) {
            // let best_move_uci = best_move.uci();
            self.transposition_table.insert(&state, depth, alpha, best_move, tt_flag)
        }

         return alpha;
     }

    pub fn quiescence(&mut self, state: &BoardState, depth: i32, ply: Depth, mut alpha: Value, beta: Value) -> Value {
        // if (stop || Limits.checkLimits()){
        //     stop = true;
        //     return 0;
        // }
        self.sel_depth = self.sel_depth.max(ply);
        self.statistics.increment_qnodes();

        let mut value = Evaluation::evaluate_state(state);

        if value >= beta {
            self.statistics.increment_qleafs();
            return beta;
        }

        if alpha < value {
            alpha = value;
        }

        let moves = state.generate_legal_moves_wo(true);
        for moov in moves.over_sorted(&state, self.transposition_table) {

            // Skip if under-promotion.
            if moov.is_promotion() && moov.flags() != Move::PR_QUEEN && moov.flags() != Move::PC_QUEEN {
                continue;
            }

            let new_state = state.do_move(&moov);
            let depth_m1 = depth - 1;
            // let state_str = state.to_fen();
            // let move_str = moov.uci();
            // let new_state_str = new_state.to_fen();
            value = -self.quiescence(&new_state, depth_m1, ply + 1, -beta, -alpha);

            // if (stop) {
            //     return 0;
            // }

            if value > alpha {
                if value >= beta {
                    self.statistics.increment_qbeta_cutoffs();
                    return beta;
                }
                alpha = value;
            }
        }
        return alpha;
    }

    //     public static boolean isScoreCheckmate(int score){
    //         return Math.abs(score) >= INF/2;
    //     }

    pub fn can_apply_null_window(state: &BoardState, depth: Depth, beta: Value, in_check: bool, can_apply_null: bool) -> bool {
        return can_apply_null &&
                !in_check &&
                depth >= Search::NULL_MIN_DEPTH &&
                state.has_non_pawn_material(state.side_to_play) &&
                Evaluation::evaluate_state(state) >= beta;
    }

    pub fn can_apply_lmr(&self, depth: Depth, moov: &Move, move_index: usize) -> bool {
        return depth > Search::LMR_MIN_DEPTH &&
                move_index > Search::LMR_MOVES_WO_REDUCTION &&
                moov.flags() == Move::QUIET;
    }

    pub fn print_info_line(&self, state: &BoardState, search_result: &SearchResult, depth: Depth) {
        let time_elapsed = self.time_elapsed();
        let info_line = format!("info currmove {} depth {} seldepth {} time {} score cp {} nodes {} nps {} pv {}",
                                search_result.moov.map(|m|m.uci()).unwrap_or(String::from("(none)")),
                                depth,
                                self.sel_depth,
                                time_elapsed,
                                search_result.score,
                                self.statistics.total_nodes(),
                                (self.statistics.total_nodes() as f32 / time_elapsed as f32 * 1000f32) as u32,
                                self.get_pv(state, depth)
        );
        println!("{}", info_line);
    }

    fn get_pv(&self, state: &BoardState, depth: Depth) -> String {
        // TODO simplify
        //let hash = state.hash;
        let best_entry = self.transposition_table.probe(state);
        if best_entry.is_none() || depth == 0 {
            return "".to_string();
        }
        let best_move = best_entry.unwrap().best_move();
        let moov = best_move; //Move::new_from_bits(best_move as u32);
        let uci = moov.uci();
        //let old_board_string = format!("{}{:#020x}\n{}", state.to_string(), state.hash, state.hash);
        let new_board_state = state.do_move(&moov);
        // let board_string = new_board_state.to_string();
        //let board_string = format!("{} {}{:#020x}\n{}", uci, new_board_state.to_string(), new_board_state.hash, new_board_state.hash);
        let primary_value = format!("{} {}", uci, self.get_pv(&new_board_state, depth - 1));
        primary_value
    }

    fn time_elapsed(&self) -> u32 {
        let now = Instant::now();
        let duration = now - self.start_time;
        duration.as_millis() as u32
    }

    fn check_stopping(&mut self) -> bool {
        if self.statistics.nodes >= self.search_limit.max_nodes {
            self.stopped = true;
        }

        self.time_checking_round += 1;

        if self.time_checking_round >= 1000 {
            self.time_checking_round = 0;
            if self.time_elapsed() >= self.search_limit.max_ms {
                self.stopped = true;
            }
        }

        self.stopped
    }
}
