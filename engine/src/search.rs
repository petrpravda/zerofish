// use crate::r#move::Move;
//
// pub struct SearchResult {
//     pub mowe: Option<Move>,
//     pub score: i32,
// }

// pub struct Search {
//
// }
//
// impl Search {
//
//     //     public final static int INF = 999999;
//     //
//     //     private final static int NULL_MIN_DEPTH = 2;
//     //     private final static int LMR_MIN_DEPTH = 2;
//     //     private final static int LMR_MOVES_WO_REDUCTION = 1;
//     //     private final static int ASPIRATION_WINDOW = 25;
//     //
//     //
//     //     private static boolean stop;
//     //     private static int selDepth;
//     //     private static final int[][] LMR_TABLE = new int[64][64];
//     //     static {
//     //         // Ethereal LMR formula with depth and number of performed moves
//     //         for (int depth = 1; depth < 64; depth++) {
//     //             for (int moveNumber = 1; moveNumber < 64; moveNumber++) {
//     //                 LMR_TABLE[depth][moveNumber] = (int) (0.75f + Math.log(depth) * Math.log(moveNumber) / 2.25f);
//     //             }
//     //         }
//     //     }
//     //
//     //     private final PrintStream streamOut;
//     //
//     //     private BoardPosition searchPosition;
//     //
//     //     public Search() {
//     //         // sort of null stream
//     //         this.streamOut = new PrintStream(new ByteArrayOutputStream());
//     //     }
//     //
//     //     public Search(PrintStream out) {
//     //         // sort of null stream
//     //         this.streamOut = out;
//     //     }
//     //
//
//
//     //     pub fn itDeep(&self, position: &BoardPosition, searchDepth: u16) -> SearchResult {
//     //         let result = SearchResult { mowe: None, score: 0 };
//     //
//     // //         self.searchPosition = position.forSearchDepth(searchDepth);
//     // //         // Limits.calcTime(board.getSideToPlay(), board.gamePly());
//     // //         Limits.startTime = System.nanoTime();
//     // //         selDepth = 0;
//     // //         stop = false;
//     // //         int alpha = -INF;
//     // //         int beta = INF;
//     // //         int depth = 1;
//     // //         SearchResult result = new SearchResult(Optional.empty(), 0);
//     // //         // MoveOrder.ageHistory();
//     // //
//     // //         // Deepen until end conditions
//     // //         while (depth <= searchDepth) {
//     // //
//     // //             // Check to see if the time has ended
//     // //             //long elapsed = System.currentTimeMillis() - Limits.startTime;
//     // // //            if (stop || elapsed >= Limits.timeAllocated / 2 || isScoreCheckmate(result.score()))
//     // // //                break;
//     // //
//     // //
//     // //             result = negaMaxRoot(position.getState(), depth, alpha, beta);
//     // //
//     // //             // Failed low, adjust window
//     // //             if (result.score <= alpha) {
//     // //                 alpha = -INF;
//     // //             }
//     // //
//     // //             // Failed high, adjust window
//     // //             else if (result.score >= beta){
//     // //                 beta = INF;
//     // //             }
//     // //
//     // //             // Adjust the window around the new score and increase the depth
//     // //             else {
//     // //                 printInfo(position.getState(), result, depth);
//     // //                 alpha = result.score - ASPIRATION_WINDOW;
//     // //                 beta = result.score + ASPIRATION_WINDOW;
//     // //                 depth++;
//     // //                 Statistics.reset();
//     // //             }
//     // //         }
//     //
//     //         return result;
//     //     }
//
//     //     public SearchResult negaMaxRoot(BoardState state, int depth, int alpha, int beta){
//     //         int value = -INF;
//     //         MoveList moves = state.generateLegalMoves();
//     //         boolean inCheck = state.checkers() != 0;
//     //         if (inCheck) ++depth;
//     //         if (moves.size() == 1) {
//     //             return new SearchResult(Optional.of(moves.get(0)), 0);
//     //         }
//     //
//     //         MoveOrder.scoreMoves(state, moves, 0);
//     //         Move bestMove = null;
//     //         for (int i = 0; i < moves.size(); i++){
//     //             MoveOrder.sortNextBestMove(moves, i);
//     //             Move move = moves.get(i);
//     //
//     //             BoardState newBoardState = state.doMove(move);
//     //             value = -negaMax(newBoardState, depth - 1, 1, -beta, -alpha, true);
//     //
//     //             if (stop || Limits.checkLimits()) {
//     //                 stop = true;
//     //                 break;
//     //             }
//     //             if (value > alpha){
//     //                 bestMove = move;
//     //                 if (value >= beta){
//     //                     TranspTable.set(state.hash(), beta, depth, TTEntry.LOWER_BOUND, bestMove);
//     //                     return new SearchResult(Optional.of(bestMove), beta);
//     //                 }
//     //                 alpha = value;
//     //                 TranspTable.set(state.hash(), alpha, depth, TTEntry.UPPER_BOUND, bestMove);
//     //             }
//     //         }
//     //         if (bestMove == null && moves.size() >= 1) {
//     //             bestMove = moves.get(0);
//     //             TranspTable.set(state.hash(), alpha, depth, TTEntry.EXACT, bestMove);
//     //         }
//     //
//     //         return new SearchResult(Optional.ofNullable(bestMove), alpha);
//     //     }
//     //
//     //     public int negaMax(BoardState state, int depth, int ply, int alpha, int beta, boolean canApplyNull){
//     //         int mateValue = INF - ply;
//     //         boolean inCheck;
//     //         int ttFlag = TTEntry.UPPER_BOUND;
//     //         int reducedDepth;
//     //
//     //         if (stop || Limits.checkLimits()) {
//     //             stop = true;
//     //             return 0;
//     //         }
//     //
//     //         // MATE DISTANCE PRUNING
//     //         if (alpha < -mateValue) alpha = -mateValue;
//     //         if (beta > mateValue - 1) beta = mateValue - 1;
//     //         if (alpha >= beta) {
//     //             Statistics.leafs++;
//     //             return alpha;
//     //         }
//     //
//     //         inCheck = state.kingAttacked();
//     //         if (depth <= 0 && !inCheck) return qSearch(state, depth, ply, alpha, beta);
//     //         Statistics.nodes++;
//     //
//     //         if (state.isRepetitionOrFifty(this.searchPosition)) {
//     //             Statistics.leafs++;
//     //             return 0;
//     //         }
//     //
//     //         // PROBE TTABLE
//     //         final TTEntry ttEntry = TranspTable.probe(state.hash());
//     //         if (ttEntry != null && ttEntry.depth() >= depth) {
//     //             Statistics.ttHits++;
//     //             switch (ttEntry.flag()) {
//     //                 case TTEntry.EXACT:
//     //                     Statistics.leafs++;
//     //                     return ttEntry.score();
//     //                 case TTEntry.LOWER_BOUND:
//     //                     alpha = Math.max(alpha, ttEntry.score());
//     //                     break;
//     //                 case TTEntry.UPPER_BOUND:
//     //                     beta = Math.min(beta, ttEntry.score());
//     //                     break;
//     //             }
//     //             if (alpha >= beta) {
//     //                 Statistics.leafs++;
//     //                 return ttEntry.score();
//     //             }
//     //         }
//     //
//     //         // NULL MOVE
//     //         if (canApplyNullWindow(state, depth, beta, inCheck, canApplyNull)) {
//     //             int R = depth > 6 ? 3 : 2;
//     //             BoardState newBoardState = state.doNullMove();
//     //             int value = -negaMax(newBoardState, depth - R - 1, ply, -beta, -beta + 1, false);
//     //             if (stop) return 0;
//     //             if (value >= beta){
//     //                 Statistics.betaCutoffs++;
//     //                 return beta;
//     //             }
//     //         }
//     //
//     //         MoveList moves = state.generateLegalMoves();
//     //         int value;
//     //         Move bestMove = Move.NULL_MOVE;
//     //         MoveOrder.scoreMoves(state, moves, ply);
//     //         for (int i = 0; i < moves.size(); i++){
//     //             MoveOrder.sortNextBestMove(moves, i);
//     //             Move move = moves.get(i);
//     //
//     //             // LATE MOVE REDUCTION
//     //             reducedDepth = depth;
//     //             if (canApplyLMR(depth, move, i)) {
//     //                 reducedDepth -= LMR_TABLE[Math.min(depth, 63)][Math.min(i, 63)];
//     //             }
//     //
//     //             if (inCheck) reducedDepth++;
//     //
//     //             BoardState newBoardState = state.doMove(move);
//     //             value = -negaMax(newBoardState, reducedDepth - 1, ply + 1, -beta, -alpha, true);
//     //
//     //             if (stop) return 0;
//     //
//     //             if (value > alpha){
//     //                 bestMove = move;
//     //                 if (value >= beta) {
//     //                     if (move.flags() == Move.QUIET) {
//     //                         MoveOrder.addKiller(state, move, ply);
//     //                         //MoveOrder.addHistory(move, depth);
//     //                     }
//     //                     Statistics.betaCutoffs++;
//     //                     ttFlag = TTEntry.LOWER_BOUND;
//     //                     alpha = beta;
//     //                     break;
//     //                 }
//     //                 ttFlag = TTEntry.EXACT;
//     //                 alpha = value;
//     //             }
//     //         }
//     //
//     //         // Check if we are in checkmate or stalemate.
//     //         if (moves.size() == 0){
//     //             if (inCheck)
//     //                 alpha = -mateValue;
//     //             else
//     //                 alpha = 0;
//     //         }
//     //
//     //         if (!bestMove.equals(Move.NULL_MOVE) && !stop) TranspTable.set(state.hash(), alpha, depth, ttFlag, bestMove);
//     //
//     //         return alpha;
//     //     }
//     //
//     //     public int qSearch(BoardState state, int depth, int ply, int alpha, int beta){
//     //         if (stop || Limits.checkLimits()){
//     //             stop = true;
//     //             return 0;
//     //         }
//     //         selDepth = Math.max(ply, selDepth);
//     //         Statistics.qnodes++;
//     //
//     //         int value = Evaluation.evaluateState(state);
//     //
//     //         if (value >= beta){
//     //             Statistics.qleafs++;
//     //             return beta;
//     //         }
//     //
//     //         if (alpha < value)
//     //             alpha = value;
//     //
//     //         MoveList moves = state.generateLegalQuiescence();
//     //         MoveOrder.scoreMoves(state, moves, ply);
//     //         for (int i = 0; i < moves.size(); i++) {
//     //             MoveOrder.sortNextBestMove(moves, i);
//     //             Move move = moves.get(i);
//     //
//     //             // Skip if underpromotion.
//     //             if (move.isPromotion() && move.flags() != Move.PR_QUEEN && move.flags() != Move.PC_QUEEN)
//     //                 continue;
//     //
//     //             BoardState newBoardState = state.doMove(move);
//     //             value = -qSearch(newBoardState, depth - 1, ply + 1, -beta, -alpha);
//     //
//     //             if (stop)
//     //                 return 0;
//     //
//     //             if (value > alpha) {
//     //                 if (value >= beta) {
//     //                     Statistics.qbetaCutoffs++;
//     //                     return beta;
//     //                 }
//     //                 alpha = value;
//     //             }
//     //         }
//     //         return alpha;
//     //     }
//     //
//     //     public static boolean isScoreCheckmate(int score){
//     //         return Math.abs(score) >= INF/2;
//     //     }
//     //
//     //     public static boolean canApplyNullWindow(BoardState state, int depth, int beta, boolean inCheck, boolean canApplyNull){
//     //         return canApplyNull &&
//     //                 !inCheck &&
//     //                 depth >= NULL_MIN_DEPTH &&
//     //                 state.hasNonPawnMaterial(state.getSideToPlay()) &&
//     //                 Evaluation.evaluateState(state) >= beta;
//     //     }
//     //
//     //     public static boolean canApplyLMR(int depth, Move move, int moveIndex){
//     //         return depth > LMR_MIN_DEPTH &&
//     //                 moveIndex > LMR_MOVES_WO_REDUCTION &&
//     //                 move.flags() == Move.QUIET;
//     //     }
//     //
//     //     public static String getPv(BoardState state, int depth){
//     //         Move bestMove;
//     //         if (TranspTable.probe(state.hash()) == null || depth == 0)
//     //             return "";
//     //         else
//     //             bestMove = TranspTable.probe(state.hash()).move();
//     //         //board.push(bestMove);
//     //         BoardState newBoardState = state.doMove(bestMove);
//     //         String pV = bestMove.uci() + " " + getPv(newBoardState, depth - 1);
//     //         return pV;
//     //     }
//     //
//     // //    public static Move getMove(){
//     // //        return Objects.requireNonNullElseGet(IDMove, Move::nullMove);
//     // //    }
//     // //
//     // //    public static int getScore(){
//     // //        return IDScore;
//     // //    }
//     //
//     //     public static void stop(){
//     //         stop = true;
//     //     }
//     //
//     //     public void printInfo(BoardState state, SearchResult result, int depth){
//     //         streamOut.print("info");
//     //         streamOut.print(" currmove " + result.move.map(Move::toString).orElse("(none)"));
//     //         streamOut.print(" depth " + depth);
//     //         streamOut.print(" seldepth " + selDepth);
//     //         streamOut.print(" time " + (int)(Limits.timeElapsed() / MEGA));
//     //         streamOut.print(" score cp " + result.score);
//     //         streamOut.print(" nodes " + Statistics.totalNodes());
//     //         streamOut.printf(" nps %.0f", Statistics.totalNodes()/((double)Limits.timeElapsed()/GIGA));
//     //         streamOut.println(" pv " + getPv(state, depth));
//     //     }
//     //
//     //     public static void main(String[] args) {
//     //         BoardPosition position = BoardPosition.fromFen(START_POS);
//     //         new Search(System.out).itDeep(position, 9);
//     //     }
//
// }
