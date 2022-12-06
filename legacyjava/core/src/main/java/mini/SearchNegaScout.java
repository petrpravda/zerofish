package mini;

import evaluation.Evaluation;
import org.javafish.board.BoardPosition;
import org.javafish.board.BoardState;
import org.javafish.move.Move;
import org.javafish.move.MoveList;
import org.javafish.move.MoveList.IndexedMove;
import search.*;

import java.io.PrintStream;
import java.util.Optional;

import static org.javafish.board.Fen.START_POS;
import static search.Search.INF;

public class SearchNegaScout {
    // https://www.chessprogramming.org/NegaScout
    // https://en.wikipedia.org/wiki/Principal_variation_search ?
    private final TranspTable transpositionTable;
    private final Statistics statistics;
    private final PrintStream streamOut;
    private BoardPosition searchPosition;

    public SearchNegaScout(TranspTable transpositionTable, PrintStream out) {
        this.transpositionTable = transpositionTable;
        this.statistics = new Statistics();
        this.streamOut = out;
    }

//    int negaScout(BoardState state, int alpha, int beta, int depth) {
//        /* compute minimax value of position p */
//        // int b, t, i;
//        if (depth == 0) {
//            return state.interpolatedScore();
//            //return quiesce(state, alpha, beta, depth);           /* leaf node */
//        }
//        MoveList moves = state.generateLegalMoves();
//        int b = beta;
//        for (IndexedMove move : moves.overSorted(state, transpositionTable, -123, null)) {
//            BoardState newState = state.doMove(move.move());
//            int value = -negaScout(newState, -b, -alpha, depth - 1);
//            if ((value > alpha) && (value < beta) && (move.index() > 0)) {
//                value = -negaScout(newState, -beta, -alpha, depth - 1); /* re-search */
//            }
//            alpha = Math.max(alpha, value);
//            if (alpha >= beta) {
//                transpositionTable.set(state.hash(), beta, depth, TTEntry.LOWER_BOUND, move.move());
//                return alpha;                            /* cut-off */
//            }
//            b = alpha + 1;                  /* set new null window */
//        }
//        return alpha;
//    }

//    int quiesce(BoardState state, int alpha, int beta, int depth){
////        if (stop || Limits.checkLimits()){
////            stop = true;
////            return 0;
////        }
////        selDepth = Math.max(ply, selDepth);
//        statistics.qnodes++;
//
//        int value = Evaluation.evaluateState(state);
//
//        if (value >= beta){
//            statistics.qleafs++;
//            return beta;
//        }
//
//        if (alpha < value)
//            alpha = value;
//
//        MoveList moves = state.generateLegalQuiescence();
//        for (IndexedMove indexedMove : moves.overSorted(state, transpositionTable, -123, null)) {
//            Move move = indexedMove.move();
//
//            // Skip if underpromotion.
//            if (move.isPromotion() && move.flags() != Move.PR_QUEEN && move.flags() != Move.PC_QUEEN)
//                continue;
//
//            BoardState newBoardState = state.doMove(move);
//            value = -quiesce(newBoardState, -beta, -alpha, depth - 1);
//
//            if (value > alpha) {
//                if (value >= beta) {
//                    statistics.qbetaCutoffs++;
//                    return beta;
//                }
//                alpha = value;
//            }
//        }
//        return alpha;
//    }









//    public SearchResult negaMaxRoot(BoardState state, int depth, int alpha, int beta){
//        int value;
//        MoveList moves = state.generateLegalMoves();
////        boolean inCheck = state.checkers() != 0;
////        if (inCheck) ++depth;
//        if (moves.size() == 1) {
//            return new SearchResult(Optional.of(moves.get(0)), 0);
//        }
//
//        Move bestMove = null;
//
//        for (IndexedMove indexedMove : moves.overSorted(state, transpositionTable, 0, null)) {
//            Move move = indexedMove.move();
//
//            BoardState newBoardState = state.doMove(move);
//            value = -negaMax(newBoardState, depth - 1, 1, -beta, -alpha, true);
//
////            if (stop || Limits.checkLimits()) {
////                stop = true;
////                break;
////            }
//            if (value > alpha){
//                bestMove = move;
//                if (value >= beta){
//                    transpositionTable.set(state.hash(), beta, depth, TTEntry.LOWER_BOUND, bestMove);
//                    return new SearchResult(Optional.of(bestMove), beta);
//                }
//                alpha = value;
//                transpositionTable.set(state.hash(), alpha, depth, TTEntry.UPPER_BOUND, bestMove);
//            }
//        }
//        if (bestMove == null && moves.size() >= 1) {
//            bestMove = moves.get(0);
//            transpositionTable.set(state.hash(), alpha, depth, TTEntry.EXACT, bestMove);
//        }
//
//        return new SearchResult(Optional.ofNullable(bestMove), alpha);
//    }
//
//
//    // TODO vyhodit ply, vyhodit INF
//    public int negaMax(BoardState state, int depth, int ply, int alpha, int beta, boolean canApplyNull){
//        int mateValue = INF - ply;
//        boolean inCheck;
//        int ttFlag = TTEntry.UPPER_BOUND;
//        int reducedDepth;
//
////        if (stop || Limits.checkLimits()) {
////            stop = true;
////            return 0;
////        }
//
//        // MATE DISTANCE PRUNING
//        if (alpha < -mateValue) alpha = -mateValue;
//        if (beta > mateValue - 1) beta = mateValue - 1;
//        if (alpha >= beta) {
//            statistics.leafs++;
//            return alpha;
//        }
//
//        inCheck = state.isKingAttacked();
//        if (depth <= 0 && !inCheck) return state.interpolatedScore(); //qSearch(state, depth, ply, alpha, beta);
//        statistics.nodes++;
//
//        if (state.isRepetitionOrFifty(this.searchPosition)) {
//            statistics.leafs++;
//            return 0;
//        }
//
//        // PROBE TTABLE
//        final TTEntry ttEntry = transpositionTable.probe(state.hash());
//        if (ttEntry != null && ttEntry.depth() >= depth) {
//            statistics.ttHits++;
//            switch (ttEntry.flag()) {
//                case TTEntry.EXACT -> {
//                    statistics.leafs++;
//                    return ttEntry.score();
//                }
//                case TTEntry.LOWER_BOUND -> alpha = Math.max(alpha, ttEntry.score());
//                case TTEntry.UPPER_BOUND -> beta = Math.min(beta, ttEntry.score());
//            }
//            if (alpha >= beta) {
//                statistics.leafs++;
//                return ttEntry.score();
//            }
//        }
//
////        // NULL MOVE
////        if (canApplyNullWindow(state, depth, beta, inCheck, canApplyNull)) {
////            int R = depth > 6 ? 3 : 2;
////            BoardState newBoardState = state.doNullMove();
////            int value = -negaMax(newBoardState, depth - R - 1, ply, -beta, -beta + 1, false);
////            if (stop) return 0;
////            if (value >= beta){
////                statistics.betaCutoffs++;
////                return beta;
////            }
////        }
//
//        MoveList moves = state.generateLegalMoves();
//        int value;
//        Move bestMove = Move.NULL_MOVE;
//        for (IndexedMove indexedMove : moves.overSorted(state, transpositionTable, ply, null)) {
//            Move move = indexedMove.move();
//            int i = indexedMove.index();
//
//            // LATE MOVE REDUCTION
//            reducedDepth = depth;
////            if (canApplyLMR(depth, move, i)) {
////                reducedDepth -= LMR_TABLE[Math.min(depth, 63)][Math.min(i, 63)];
////            }
//
//            if (inCheck) reducedDepth++;
//
//            BoardState newBoardState = state.doMove(move);
//            value = -negaMax(newBoardState, reducedDepth - 1, ply + 1, -beta, -alpha, true);
//
////            if (stop) return 0;
//
//            if (value > alpha){
//                bestMove = move;
//                if (value >= beta) {
//                    if (move.flags() == Move.QUIET) {
//                        // moveOrdering.addKiller(state, move, ply);
//                        //MoveOrder.addHistory(move, depth);
//                    }
//                    statistics.betaCutoffs++;
//                    ttFlag = TTEntry.LOWER_BOUND;
//                    alpha = beta;
//                    break;
//                }
//                ttFlag = TTEntry.EXACT;
//                alpha = value;
//            }
//        }
//
//        // Check if we are in checkmate or stalemate.
//        if (moves.size() == 0){
//            if (inCheck)
//                alpha = -mateValue;
//            else
//                alpha = 0;
//        }
//
//        if (!bestMove.equals(Move.NULL_MOVE) /*&& !stop*/) transpositionTable.set(state.hash(), alpha, depth, ttFlag, bestMove);
//
//        return alpha;
//    }






    public String getPv(BoardState state, int depth){
        TTEntry bestEntry = transpositionTable.probe(state.hash());
        if (bestEntry == null || depth == 0) {
            return "";
        }
        Move bestMove = bestEntry.move();
        BoardState newBoardState = state.doMove(bestMove);
        return bestMove.uci() + " " + getPv(newBoardState, depth - 1);
    }

//    public SearchResult itDeep(BoardPosition position, int searchDepth) {
//        this.searchPosition = position.forSearchDepth(searchDepth);
//
//        int alpha = -INF;
//        int beta = INF;
//        int depth = searchDepth;
//        BoardState state = position.getState();
//        SearchResult result = negaMaxRoot(state, depth, alpha, beta);
//        String pv = getPv(state, depth);
//        System.out.println(pv);
//        return result; // new SearchResult(Optional.empty(), result);
//    };
//
//    public static void main(String[] args) {
//        BoardPosition position = BoardPosition.fromFen(START_POS);
//        new SearchNegaScout(new TranspTable(), System.out).itDeep(position, 8);
//    }
}
