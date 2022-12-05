use lazy_static::lazy_static;
use crate::board_position::BoardPosition;
use crate::board_state::BoardState;
use crate::evaluation::Evaluation;
use crate::fen::START_POS;
use crate::r#move::{Move, MoveList};
use crate::statistics::Statistics;
use crate::time::Instant;
use crate::transposition::{Depth, TranspositionTable, TTEntry};

#[derive(Debug)]
pub struct SearchResult {
    pub mowe: Option<Move>,
    pub score: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

//     private static final int[][] LMR_TABLE = new int[64][64];
//     static {
//         // Ethereal LMR formula with depth and number of performed moves
//     }
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

pub struct Search {
    search_position: BoardPosition, // TODO rename to board_position
    start_time: Instant,
    sel_depth: Depth,
    stop: bool,
    statistics: Statistics,
    transposition_table: &'static TranspositionTable,
}

lazy_static! {
    pub static ref TT: TranspositionTable = TranspositionTable::new(1);
}

impl Search {

    pub const INF: i32 = 999999;
    pub const NULL_MIN_DEPTH: Depth = 2;

    const LMR_MIN_DEPTH: Depth = 2;
    const LMR_MOVES_WO_REDUCTION: usize = 1; // TODO which type?
    const ASPIRATION_WINDOW: i32 = 25;

    pub fn new() -> Self {
        Self {
            search_position: BoardPosition::from_fen(START_POS),
            start_time: Instant::now(),
            sel_depth: 10,
            stop: false,
            statistics: Statistics::new(),
            transposition_table: &TT,
        }
    }

    //
    //
    //     private static boolean stop;
    //     private static int selDepth;
    //     private static final int[][] LMR_TABLE = new int[64][64];
    //     static {
    //         // Ethereal LMR formula with depth and number of performed moves
    //         for (int depth = 1; depth < 64; depth++) {
    //             for (int moveNumber = 1; moveNumber < 64; moveNumber++) {
    //                 LMR_TABLE[depth][moveNumber] = (int) (0.75f + Math.log(depth) * Math.log(moveNumber) / 2.25f);
    //             }
    //         }
    //     }
    //
    //     private final PrintStream streamOut;
    //
    //     private BoardPosition searchPosition;
    //
    //     public Search() {
    //         // sort of null stream
    //         this.streamOut = new PrintStream(new ByteArrayOutputStream());
    //     }
    //
    //     public Search(PrintStream out) {
    //         // sort of null stream
    //         this.streamOut = out;
    //     }
    //


        pub fn itDeep(&mut self, position: &BoardPosition, search_depth: Depth) -> SearchResult {
            let mut result = SearchResult { mowe: None, score: 0 };

            self.search_position = position.for_search_depth(search_depth);
            self.start_time = Instant::now();
            self.sel_depth = 0;
            self.stop = false;
            let mut alpha: i32 = -Search::INF;
            let mut beta: i32 = Search::INF;
            let mut depth: Depth = 1;

            // Deepen until end conditions
            while depth <= search_depth {

                // Check to see if the time has ended
                //long elapsed = System.currentTimeMillis() - Limits.startTime;
    //            if (stop || elapsed >= Limits.timeAllocated / 2 || isScoreCheckmate(result.score()))
    //                break;


                result = self.negaMaxRoot(&position.state, depth, alpha, beta);

                // Failed low, adjust window
                if result.score <= alpha {
                    alpha = -Search::INF;
                }

                // Failed high, adjust window
                else if result.score >= beta {
                    beta = Search::INF;
                }

                // Adjust the window around the new score and increase the depth
                else {
                    self.printInfo(&position.state, &result, depth);
                    alpha = result.score - Search::ASPIRATION_WINDOW;
                    beta = result.score + Search::ASPIRATION_WINDOW;
                    depth += 1;
                    self.statistics.reset();
                }
            }

            return result;
        }

        pub fn negaMaxRoot(&mut self, state: &BoardState, depth: Depth, mut alpha: i32, beta: i32) -> SearchResult{
            let mut value = -Search::INF;
            let mut moves = state.generate_legal_moves();
            // let inCheck = state.checkers() != 0;
            // if (inCheck) ++depth;
            if moves.len() == 1 {
                return SearchResult{ mowe: moves.moves.get(0).copied(), score: 0 }; // new SearchResult(Optional.of(moves.get(0)), 0);
            }

            let mut bestMove: Option<Move> = None;
            // self.score_moves(state, moves, 0);
            for indexed_moov in moves.over_sorted(&state, self.transposition_table) {
                let moov = indexed_moov.moov;

                let newBoardState = state.do_move(&moov);
                value = -self.nega_max(&newBoardState, depth - 1, 1, -beta, -alpha, true);

                // if (stop || Limits.checkLimits()) {
                //     stop = true;
                //     break;
                // }
                if value > alpha {
                    bestMove = Some(moov.clone());
                    if value >= beta {
                        self.transposition_table.insert(&state, depth, beta, moov.base_move(), Bound::Lower);
                        //set(state.hash, beta, depth, Bound::Lower, bestMove);
                        return SearchResult{ mowe: bestMove, score: beta };
                    }
                    alpha = value;
                    self.transposition_table.insert(&state, depth, alpha, moov.base_move(), Bound::Upper);
                }
            }

            if bestMove.is_some() && moves.len() >= 1 {
                bestMove = Some(moves.moves[0].clone());
                self.transposition_table.insert(&state, depth, alpha, bestMove.unwrap().base_move(), Bound::Exact);
                //TranspTable.set(state.hash(), alpha, depth, TTEntry.EXACT, bestMove);
            }

            SearchResult{ mowe: bestMove, score: alpha }  // (Optional.ofNullable(bestMove), alpha);
        }

        pub fn nega_max(&mut self, state: &BoardState, depth: Depth, ply: u16, mut alpha: i32, mut beta: i32, canApplyNull: bool) -> i32 {
            let mateValue = Search::INF - ply as i32;
            let mut inCheck = false;
            let mut ttFlag = Bound::Upper;
            let mut reducedDepth = 0; // TODO is really needed?

            // if (stop || Limits.checkLimits()) {
            //     stop = true;
            //     return 0;
            // }

            // MATE DISTANCE PRUNING
            if alpha < -mateValue {
                alpha = -mateValue;
            }
            if beta > mateValue - 1 {
                beta = mateValue - 1;
            }
            if alpha >= beta {
                self.statistics.increment_leafs();
                return alpha;
            }

            inCheck = state.is_king_attacked();
            if depth <= 0 && !inCheck {
                return self.qSearch(state, depth, ply as Depth, alpha, beta);
            }
            self.statistics.increment_nodes();

            if state.is_repetition_or_fifty(&self.search_position) {
                self.statistics.increment_leafs();
                return 0;
            }

            // PROBE TTABLE
            let ttEntry = self.transposition_table.probe(state);
            if ttEntry.is_some() && ttEntry.unwrap().depth() >= depth {
                let tt_entry_some = ttEntry.unwrap();
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
            if Search::can_apply_null_window(state, depth, beta, inCheck, canApplyNull) {
                let R = if depth > 6 { 3 } else { 2 };
                let newBoardState = state.do_null_move();
                let value = -self.nega_max(&newBoardState, depth - R - 1, ply, -beta, -beta + 1, false);
                // if (stop) {
                //     return 0;
                // }
                if value >= beta {
                    self.statistics.increase_beta_cutoffs();
                    return beta;
                }
            }

            let mut moves = state.generate_legal_moves();
            let mut value = 0;
            let mut bestMove: Option<Move> = None;
            // MoveOrder.scoreMoves(state, moves, ply);
            for indexed_moov in moves.over_sorted(&state, self.transposition_table) {
                let moov = indexed_moov.moov;
                let index = indexed_moov.index;

                // LATE MOVE REDUCTION
                reducedDepth = depth;
                if self.can_apply_lmr(depth, &moov, index) {
                    reducedDepth = reducedDepth - LMR_TABLE[depth.min(63) as usize][index.min(63) as usize] as u8;
                }

                if inCheck {
                    reducedDepth += 1;
                }

                let newBoardState = state.do_move(&moov);
                value = -self.nega_max(&newBoardState, reducedDepth - 1, ply + 1, -beta, -alpha, true);

                // if (stop) {
                //     return 0;
                // }

                if value > alpha {
                    bestMove = Some(moov);
                    if value >= beta {
                        if moov.flags() == Move::QUIET {
                            //MoveOrder.addKiller(state, move, ply);
                            //MoveOrder.addHistory(move, depth);
                        }
                        self.statistics.increase_beta_cutoffs();
                        ttFlag = Bound::Lower;
                        alpha = beta;
                        break;
                    }
                    ttFlag = Bound::Exact;
                    alpha = value;
                }
            }

            // Check if we are in checkmate or stalemate.
            if moves.len() == 0 {
                if inCheck {
                    alpha = -mateValue;
                } else {
                    alpha = 0;
                }
            }

            if !bestMove.unwrap().bits == Move::NULL_MOVE.bits { // TODO && !stop) {
                // TranspTable.set(state.hash(), alpha, depth, ttFlag, bestMove);
                self.transposition_table.insert(&state, depth, alpha, bestMove.unwrap().base_move(), ttFlag)
            }

             return alpha;
         }

        pub fn qSearch(&mut self, state: &BoardState, depth: Depth, ply: Depth, mut alpha: i32, beta: i32) -> i32 {
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

            let mut moves = state.generate_legal_moves_wo(true);
            // MoveOrder.scoreMoves(state, moves, ply);
            for indexed_moov in moves.over_sorted(&state, self.transposition_table) {
                let moov = indexed_moov.moov;

                // Skip if underpromotion.
                if moov.is_promotion() && moov.flags() != Move::PR_QUEEN && moov.flags() != Move::PC_QUEEN {
                    continue;
                }

                let newBoardState = state.do_move(&moov);
                let depth_m1 = depth - 1;
                value = -self.qSearch(&newBoardState, depth_m1, ply + 1, -beta, -alpha);

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

        pub fn can_apply_null_window(state: &BoardState, depth: Depth, beta: i32, in_check: bool, can_apply_null: bool) -> bool {
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

    //     public static String getPv(BoardState state, int depth){
    //         Move bestMove;
    //         if (TranspTable.probe(state.hash()) == null || depth == 0)
    //             return "";
    //         else
    //             bestMove = TranspTable.probe(state.hash()).move();
    //         //board.push(bestMove);
    //         BoardState newBoardState = state.doMove(bestMove);
    //         String pV = bestMove.uci() + " " + getPv(newBoardState, depth - 1);
    //         return pV;
    //     }
    //
    // //    public static Move getMove(){
    // //        return Objects.requireNonNullElseGet(IDMove, Move::nullMove);
    // //    }
    // //
    // //    public static int getScore(){
    // //        return IDScore;
    // //    }
    //
    //     public static void stop(){
    //         stop = true;
    //     }

        pub fn printInfo(&self, state: &BoardState, searchResult: &SearchResult, depth: Depth) {
            let info_line = format!("info currmove {} depth {} seldepth {} time {} score cp {} nodes {} nps {} pv {}",
                    searchResult.mowe.map(|m|m.uci()).unwrap_or(String::from("(none)")),
                depth,
                self.sel_depth,
                self.time_elapsed(),
                searchResult.score,
                0, // Statistics.totalNodes());
                0, // Statistics.totalNodes()/((double)Limits.timeElapsed()/GIGA));
                self.get_pv(state, depth)
            );
            println!("{}", info_line);
            // streamOut.print("info");
            // streamOut.print(" currmove " + result.move.map(Move::toString).orElse("(none)"));
            // streamOut.print(" depth " + depth);
            // streamOut.print(" seldepth " + selDepth);
            // streamOut.print(" time " + (int)(Limits.timeElapsed() / MEGA));
            // streamOut.print(" score cp " + result.score);
            // streamOut.print(" nodes " + Statistics.totalNodes());
            // streamOut.printf(" nps %.0f", Statistics.totalNodes()/((double)Limits.timeElapsed()/GIGA));
            // streamOut.println(" pv " + getPv(state, depth));
        }

    //     public static void main(String[] args) {
    //         BoardPosition position = BoardPosition.fromFen(START_POS);
    //         new Search(System.out).itDeep(position, 9);
    //     }

    fn time_elapsed(&self) -> u64 {
        let now = Instant::now();
        let duration = now - self.start_time;
        duration.as_millis() as u64
    }

    fn get_pv(&self, state: &BoardState, depth: Depth) -> String {
        let best_entry = self.transposition_table.probe(state);
        if best_entry.is_none() || depth == 0 {
            return "".to_string();
        }
        let best_move = best_entry.unwrap().best_move();
        let moov = Move::new_from_bits(best_move as u32);
        let new_board_state = state.do_move(&moov);
        let primary_value = format!("{} {}", moov.uci(), self.get_pv(&new_board_state, depth - 1));
        primary_value
    }
}
