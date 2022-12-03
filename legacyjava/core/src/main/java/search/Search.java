package search;

import evaluation.Evaluation;
import org.javafish.board.BoardState;
import org.javafish.board.Side;
import org.javafish.move.Move;
import org.javafish.move.MoveList;
import org.javafish.board.BoardPosition;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.util.Optional;

import static org.javafish.app.Benchmark.GIGA;
import static org.javafish.app.Benchmark.MEGA;
import static org.javafish.board.Fen.START_POS;
import static org.javafish.eval.PieceSquareTable.MGS;

public class Search {
    public final static int INF = 999999;

    private final static int NULL_MIN_DEPTH = 2;
    private final static int LMR_MIN_DEPTH = 2;
    private final static int LMR_MOVES_WO_REDUCTION = 1;
    private final static int ASPIRATION_WINDOW = 25;
    private final TranspTable transpositionTable;
    private final Statistics statistics;
    private final MoveOrdering moveOrdering = new MoveOrdering();


    private boolean stop;
    private int selDepth;

    private static final int[][] LMR_TABLE = new int[64][64];
    static {
        // Ethereal LMR formula with depth and number of performed moves
        for (int depth = 1; depth < 64; depth++) {
            for (int moveNumber = 1; moveNumber < 64; moveNumber++) {
                LMR_TABLE[depth][moveNumber] = (int) (0.75f + Math.log(depth) * Math.log(moveNumber) / 2.25f);
            }
        }
    }

    private final PrintStream streamOut;

    private BoardPosition searchPosition;

    public Search(TranspTable transpositionTable) {
//        this.transpositionTable = transpositionTable;
//        // sort of null stream
//        this.streamOut = new PrintStream(new ByteArrayOutputStream());
        this(transpositionTable, new PrintStream(new ByteArrayOutputStream()));
    }

    public Search(TranspTable transpositionTable, PrintStream out) {
        this.transpositionTable = transpositionTable;
        this.statistics = new Statistics();
        this.streamOut = out;
    }

    public void scoreMoves(final BoardState state, final MoveList moves, int ply) {

        if (moves.size() == 0)
            return;

        Move hashMove = null;
        TTEntry ttEntry = transpositionTable.probe(state.hash());
        if (ttEntry != null) {
            hashMove = ttEntry.move();
        }

        for (Move move : moves) {
            if (move.equals(hashMove)) {
                move.addToScore(MoveOrdering.HashMoveScore);
            }
            if (moveOrdering.isKiller(state, move, ply)) {
                move.addToScore(MoveOrdering.KillerMoveScore);
            }
            int piece = state.items[move.from()];

            switch (move.flags()) {
                case Move.PC_BISHOP:
                case Move.PC_KNIGHT:
                case Move.PC_ROOK:
                case Move.PC_QUEEN:
                    int score = MGS[move.getPieceTypeForSide(state.getSideToPlay())][move.to()] - MGS[piece][move.from()]
                            - MGS[state.items[move.to()]][move.to()];
                    score *= state.getSideToPlay() == Side.WHITE ? 1 : -1;
                    move.addToScore(score);
                    break;

                case Move.PR_BISHOP:
                case Move.PR_KNIGHT:
                case Move.PR_ROOK:
                case Move.PR_QUEEN:
                    score = MGS[move.getPieceTypeForSide(state.getSideToPlay())][move.to()] - MGS[piece][move.from()];
                    score *= state.getSideToPlay() == Side.WHITE ? 1 : -1;
                    move.addToScore(score);
                    break;
                case Move.CAPTURE:
                    score = MGS[piece][move.to()] - MGS[piece][move.from()] - MGS[state.items[move.to()]][move.to()];
                    score *= state.getSideToPlay() == Side.WHITE ? 1 : -1;
                    move.addToScore(score);
                    break;
                case Move.QUIET:
                case Move.EN_PASSANT:
                case Move.DOUBLE_PUSH:
                case Move.OO:
                case Move.OOO:
                    score = MGS[piece][move.to()] - MGS[piece][move.from()];
                    score *= state.getSideToPlay() == Side.WHITE ? 1 : -1;
                    move.addToScore(score);
                    break;
            }
        }
    }

    public record SearchResult(Optional<Move> move, int score) {
        @Override
        public Optional<Move> move() {
            return move;
        }

        @Override
        public int score() {
            return score;
        }
    }

    public SearchResult itDeep(BoardPosition position, int searchDepth){
        this.searchPosition = position.forSearchDepth(searchDepth);
        // Limits.calcTime(board.getSideToPlay(), board.gamePly());
        Limits.startTime = System.nanoTime();
        selDepth = 0;
        stop = false;
        int alpha = -INF;
        int beta = INF;
        int depth = 1;
        SearchResult result = new SearchResult(Optional.empty(), 0);
        // MoveOrder.ageHistory();

        // Deepen until end conditions
        while (depth <= searchDepth) {

            // Check to see if the time has ended
            //long elapsed = System.currentTimeMillis() - Limits.startTime;
//            if (stop || elapsed >= Limits.timeAllocated / 2 || isScoreCheckmate(result.score()))
//                break;


            result = negaMaxRoot(position.getState(), depth, alpha, beta);

            // Failed low, adjust window
            if (result.score <= alpha) {
                alpha = -INF;
            }

            // Failed high, adjust window
            else if (result.score >= beta){
                beta = INF;
            }

            // Adjust the window around the new score and increase the depth
            else {
                printInfo(position.getState(), result, depth);
                alpha = result.score - ASPIRATION_WINDOW;
                beta = result.score + ASPIRATION_WINDOW;
                depth++;
                statistics.reset();
            }
        }

        return result;
    }

    public SearchResult negaMaxRoot(BoardState state, int depth, int alpha, int beta){
        int value;
        MoveList moves = state.generateLegalMoves();
//        boolean inCheck = state.checkers() != 0;
//        if (inCheck) ++depth;
        if (moves.size() == 1) {
            return new SearchResult(Optional.of(moves.get(0)), 0);
        }

        scoreMoves(state, moves, 0);
        Move bestMove = null;
        for (int i = 0; i < moves.size(); i++){
            moves.pickNextBestMove(i);
            Move move = moves.get(i);

            BoardState newBoardState = state.doMove(move);
            value = -negaMax(newBoardState, depth - 1, 1, -beta, -alpha, true);

            if (stop || Limits.checkLimits()) {
                stop = true;
                break;
            }
            if (value > alpha){
                bestMove = move;
                if (value >= beta){
                    transpositionTable.set(state.hash(), beta, depth, TTEntry.LOWER_BOUND, bestMove);
                    return new SearchResult(Optional.of(bestMove), beta);
                }
                alpha = value;
                transpositionTable.set(state.hash(), alpha, depth, TTEntry.UPPER_BOUND, bestMove);
            }
        }
        if (bestMove == null && moves.size() >= 1) {
            bestMove = moves.get(0);
            transpositionTable.set(state.hash(), alpha, depth, TTEntry.EXACT, bestMove);
        }

        return new SearchResult(Optional.ofNullable(bestMove), alpha);
    }

    public int negaMax(BoardState state, int depth, int ply, int alpha, int beta, boolean canApplyNull){
        int mateValue = INF - ply;
        boolean inCheck;
        int ttFlag = TTEntry.UPPER_BOUND;
        int reducedDepth;

        if (stop || Limits.checkLimits()) {
            stop = true;
            return 0;
        }

        // MATE DISTANCE PRUNING
        if (alpha < -mateValue) alpha = -mateValue;
        if (beta > mateValue - 1) beta = mateValue - 1;
        if (alpha >= beta) {
            statistics.leafs++;
            return alpha;
        }

        inCheck = state.kingAttacked();
        if (depth <= 0 && !inCheck) return qSearch(state, depth, ply, alpha, beta);
        statistics.nodes++;

        if (state.isRepetitionOrFifty(this.searchPosition)) {
            statistics.leafs++;
            return 0;
        }

        // PROBE TTABLE
        final TTEntry ttEntry = transpositionTable.probe(state.hash());
        if (ttEntry != null && ttEntry.depth() >= depth) {
            statistics.ttHits++;
            switch (ttEntry.flag()) {
                case TTEntry.EXACT -> {
                    statistics.leafs++;
                    return ttEntry.score();
                }
                case TTEntry.LOWER_BOUND -> alpha = Math.max(alpha, ttEntry.score());
                case TTEntry.UPPER_BOUND -> beta = Math.min(beta, ttEntry.score());
            }
            if (alpha >= beta) {
                statistics.leafs++;
                return ttEntry.score();
            }
        }

        // NULL MOVE
        if (canApplyNullWindow(state, depth, beta, inCheck, canApplyNull)) {
            int R = depth > 6 ? 3 : 2;
            BoardState newBoardState = state.doNullMove();
            int value = -negaMax(newBoardState, depth - R - 1, ply, -beta, -beta + 1, false);
            if (stop) return 0;
            if (value >= beta){
                statistics.betaCutoffs++;
                return beta;
            }
        }

        MoveList moves = state.generateLegalMoves();
        int value;
        Move bestMove = Move.NULL_MOVE;
        scoreMoves(state, moves, ply);
        for (int i = 0; i < moves.size(); i++){
            moves.pickNextBestMove(i);
            Move move = moves.get(i);

            // LATE MOVE REDUCTION
            reducedDepth = depth;
            if (canApplyLMR(depth, move, i)) {
                reducedDepth -= LMR_TABLE[Math.min(depth, 63)][Math.min(i, 63)];
            }

            if (inCheck) reducedDepth++;

            BoardState newBoardState = state.doMove(move);
            value = -negaMax(newBoardState, reducedDepth - 1, ply + 1, -beta, -alpha, true);

            if (stop) return 0;

            if (value > alpha){
                bestMove = move;
                if (value >= beta) {
                    if (move.flags() == Move.QUIET) {
                        moveOrdering.addKiller(state, move, ply);
                        //MoveOrder.addHistory(move, depth);
                    }
                    statistics.betaCutoffs++;
                    ttFlag = TTEntry.LOWER_BOUND;
                    alpha = beta;
                    break;
                }
                ttFlag = TTEntry.EXACT;
                alpha = value;
            }
        }

        // Check if we are in checkmate or stalemate.
        if (moves.size() == 0){
            if (inCheck)
                alpha = -mateValue;
            else
                alpha = 0;
        }

        if (!bestMove.equals(Move.NULL_MOVE) && !stop) transpositionTable.set(state.hash(), alpha, depth, ttFlag, bestMove);

        return alpha;
    }

    public int qSearch(BoardState state, int depth, int ply, int alpha, int beta){
        if (stop || Limits.checkLimits()){
            stop = true;
            return 0;
        }
        selDepth = Math.max(ply, selDepth);
        statistics.qnodes++;

        int value = Evaluation.evaluateState(state);

        if (value >= beta){
            statistics.qleafs++;
            return beta;
        }

        if (alpha < value)
            alpha = value;

        MoveList moves = state.generateLegalQuiescence();
        scoreMoves(state, moves, ply);
        for (int i = 0; i < moves.size(); i++) {
            moves.pickNextBestMove(i);
            Move move = moves.get(i);

            // Skip if underpromotion.
            if (move.isPromotion() && move.flags() != Move.PR_QUEEN && move.flags() != Move.PC_QUEEN)
                continue;

            BoardState newBoardState = state.doMove(move);
            value = -qSearch(newBoardState, depth - 1, ply + 1, -beta, -alpha);

            if (stop)
                return 0;

            if (value > alpha) {
                if (value >= beta) {
                    statistics.qbetaCutoffs++;
                    return beta;
                }
                alpha = value;
            }
        }
        return alpha;
    }

    public static boolean isScoreCheckmate(int score){
        return Math.abs(score) >= INF/2;
    }

    public static boolean canApplyNullWindow(BoardState state, int depth, int beta, boolean inCheck, boolean canApplyNull){
        return canApplyNull &&
                !inCheck &&
                depth >= NULL_MIN_DEPTH &&
                state.hasNonPawnMaterial(state.getSideToPlay()) &&
                Evaluation.evaluateState(state) >= beta;
    }

    public static boolean canApplyLMR(int depth, Move move, int moveIndex){
        return depth > LMR_MIN_DEPTH &&
                moveIndex > LMR_MOVES_WO_REDUCTION &&
                move.flags() == Move.QUIET;
    }

    public String getPv(BoardState state, int depth){
        TTEntry bestEntry = transpositionTable.probe(state.hash());
        if (bestEntry == null || depth == 0) {
            return "";
        }
        Move bestMove = bestEntry.move();
        BoardState newBoardState = state.doMove(bestMove);
        return bestMove.uci() + " " + getPv(newBoardState, depth - 1);
    }

//    public static Move getMove(){
//        return Objects.requireNonNullElseGet(IDMove, Move::nullMove);
//    }
//
//    public static int getScore(){
//        return IDScore;
//    }

    public void stop(){
        stop = true;
    }

    public void printInfo(BoardState state, SearchResult result, int depth){
        streamOut.print("info");
        streamOut.print(" currmove " + result.move.map(Move::toString).orElse("(none)"));
        streamOut.print(" depth " + depth);
        streamOut.print(" seldepth " + selDepth);
        streamOut.print(" time " + (int)(Limits.timeElapsed() / MEGA));
        streamOut.print(" score cp " + result.score);
        streamOut.print(" nodes " + statistics.totalNodes());
        streamOut.printf(" nps %.0f", statistics.totalNodes()/((double)Limits.timeElapsed()/GIGA));
        streamOut.println(" pv " + getPv(state, depth));
    }

    public static void main(String[] args) {
        BoardPosition position = BoardPosition.fromFen(START_POS);
        new Search(new TranspTable(), System.out).itDeep(position, 9);
    }

    //         Search.stop();
    //        TranspTable.reset();
    //        MoveOrder.clearKillers();
    //        MoveOrder.clearHistory();
    //        System.gc();
    //        Limits.resetTime();
}
