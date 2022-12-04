use crate::board_position::BoardPosition;
use crate::board_state::BoardState;
use crate::evaluation::Evaluation;
use crate::r#move::{Move, MoveList};
use crate::statistics::Statistics;
use crate::time::Instant;
use crate::transposition::{Depth, TranspositionTable, TTEntry};

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

pub struct Search {
    search_position: BoardPosition, // TODO rename to board_position
    start_time: Instant,
    sel_depth: Depth,
    stop: bool,
    statistics: Statistics,
    transposition_table: &'static TranspositionTable,
}

impl Search {

    pub const INF: i32 = 999999;
    //
    //     private final static int NULL_MIN_DEPTH = 2;
    //     private final static int LMR_MIN_DEPTH = 2;
    //     private final static int LMR_MOVES_WO_REDUCTION = 1;
    const ASPIRATION_WINDOW: i32 = 25;
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
            for moov in moves.over_sorted(&state, self.transposition_table) {

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
            let ttFlag = Bound::Upper;
            // let reducedDepth = 0; // TODO is really needed?

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
                self.statistics.incrementLeafs();
                return alpha;
            }

            inCheck = state.is_king_attacked();
            if depth <= 0 && !inCheck {
                return self.qSearch(state, depth, ply as Depth, alpha, beta);
            }
            self.statistics.incrementNodes();

            if state.is_repetition_or_fifty(&self.search_position) {
                self.statistics.incrementLeafs();
                return 0;
            }

            // PROBE TTABLE
            let ttEntry = self.transposition_table.probe(state);
            if ttEntry.is_some() && ttEntry.unwrap().depth() >= depth {
                let tt_entry_some = ttEntry.unwrap();
                self.statistics.increaseTTHits();
                match tt_entry_some.flag() {
                    Bound::Exact => {
                        self.statistics.incrementLeafs();
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
                    self.statistics.incrementLeafs();
                    return tt_entry_some.value();
                }
            }

    //         // NULL MOVE
    //         if (canApplyNullWindow(state, depth, beta, inCheck, canApplyNull)) {
    //             int R = depth > 6 ? 3 : 2;
    //             BoardState newBoardState = state.doNullMove();
    //             int value = -nega_max(newBoardState, depth - R - 1, ply, -beta, -beta + 1, false);
    //             if (stop) return 0;
    //             if (value >= beta){
    //                 Statistics.betaCutoffs++;
    //                 return beta;
    //             }
    //         }
    //
    //         MoveList moves = state.generateLegalMoves();
    //         int value;
    //         Move bestMove = Move.NULL_MOVE;
    //         MoveOrder.scoreMoves(state, moves, ply);
    //         for (int i = 0; i < moves.size(); i++){
    //             MoveOrder.sortNextBestMove(moves, i);
    //             Move move = moves.get(i);
    //
    //             // LATE MOVE REDUCTION
    //             reducedDepth = depth;
    //             if (canApplyLMR(depth, move, i)) {
    //                 reducedDepth -= LMR_TABLE[Math.min(depth, 63)][Math.min(i, 63)];
    //             }
    //
    //             if (inCheck) reducedDepth++;
    //
    //             BoardState newBoardState = state.doMove(move);
    //             value = -nega_max(newBoardState, reducedDepth - 1, ply + 1, -beta, -alpha, true);
    //
    //             if (stop) return 0;
    //
    //             if (value > alpha){
    //                 bestMove = move;
    //                 if (value >= beta) {
    //                     if (move.flags() == Move.QUIET) {
    //                         MoveOrder.addKiller(state, move, ply);
    //                         //MoveOrder.addHistory(move, depth);
    //                     }
    //                     Statistics.betaCutoffs++;
    //                     ttFlag = TTEntry.LOWER_BOUND;
    //                     alpha = beta;
    //                     break;
    //                 }
    //                 ttFlag = TTEntry.EXACT;
    //                 alpha = value;
    //             }
    //         }
    //
    //         // Check if we are in checkmate or stalemate.
    //         if (moves.size() == 0){
    //             if (inCheck)
    //                 alpha = -mateValue;
    //             else
    //                 alpha = 0;
    //         }
    //
    //         if (!bestMove.equals(Move.NULL_MOVE) && !stop) TranspTable.set(state.hash(), alpha, depth, ttFlag, bestMove);
    //
             return alpha;
         }

        pub fn qSearch(&mut self, state: &BoardState, depth: Depth, ply: Depth, mut alpha: i32, beta: i32) -> i32 {
            // if (stop || Limits.checkLimits()){
            //     stop = true;
            //     return 0;
            // }
            self.sel_depth = self.sel_depth.max(ply);
            self.statistics.incrementQNodes();

            let mut value = Evaluation::evaluate_state(state);

            if value >= beta {
                self.statistics.incrementQLeafs();
                return beta;
            }

            if alpha < value {
                alpha = value;
            }

            let mut moves = state.generate_legal_moves_wo(true);
            // MoveOrder.scoreMoves(state, moves, ply);
            for moov in moves.over_sorted(&state, self.transposition_table) {

                // Skip if underpromotion.
                if moov.is_promotion() && moov.flags() != Move::PR_QUEEN && moov.flags() != Move::PC_QUEEN {
                    continue;
                }

                let newBoardState = state.do_move(&moov);
                value = -self.qSearch(&newBoardState, depth - 1, ply + 1, -beta, -alpha);

                // if (stop) {
                //     return 0;
                // }

                if value > alpha {
                    if value >= beta {
                        self.statistics.incrementQBetaCutoffs();
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
    //
    //     public static boolean canApplyNullWindow(BoardState state, int depth, int beta, boolean inCheck, boolean canApplyNull){
    //         return canApplyNull &&
    //                 !inCheck &&
    //                 depth >= NULL_MIN_DEPTH &&
    //                 state.hasNonPawnMaterial(state.getSideToPlay()) &&
    //                 Evaluation.evaluate_state(state) >= beta;
    //     }
    //
    //     public static boolean canApplyLMR(int depth, Move move, int moveIndex){
    //         return depth > LMR_MIN_DEPTH &&
    //                 moveIndex > LMR_MOVES_WO_REDUCTION &&
    //                 move.flags() == Move.QUIET;
    //     }
    //
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
        // TTEntry bestEntry = TranspTable.probe(state.hash());
        // if (bestEntry == null || depth == 0) {
        //     return "";
        // }
        // Move bestMove = bestEntry.move();
        // BoardState newBoardState = state.doMove(bestMove);
        // String pV = bestMove.uci() + " " + getPv(newBoardState, depth - 1);
        // return pV;
        todo!()
    }
}
