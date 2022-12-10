package search;

import org.javafish.board.BoardPosition;
import org.javafish.board.BoardState;
import org.javafish.move.Move;
import org.javafish.move.MoveList;
import org.javafish.move.ScoredMove;
import search.TranspositionTable.MyTtEntry;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.util.Optional;

import static org.javafish.app.Benchmark.GIGA;
import static org.javafish.app.Benchmark.MEGA;
import static org.javafish.board.Fen.START_POS;

public class Search {

    public final static int INF = 29999; // TODO short

    private final static int NULL_MIN_DEPTH = 2;
    private final static int LMR_MIN_DEPTH = 2;
    private final static int LMR_MOVES_WO_REDUCTION = 1;
    private final static int ASPIRATION_WINDOW = 25;
    public static final ScoredMove WORST_MOVE = Move.NULL_MOVE.scored(Short.MIN_VALUE);
    private final Statistics statistics = new Statistics();
    private final MoveOrdering moveOrdering = new MoveOrdering();

    // Create a transposition table to store previously evaluated positions and their scores
    private final TranspositionTable transpositionTable;
    private final PrintStream streamOut;

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


    private BoardPosition searchPosition;
    private short currentSearchDepth;

    public Search(TranspositionTable transpositionTable) {
        this(transpositionTable, new PrintStream(new ByteArrayOutputStream()));
    }

    public Search(TranspositionTable transpositionTable, PrintStream out) {
        this.transpositionTable = transpositionTable;
        this.streamOut = out;
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

    public search.SearchResult itDeep(BoardPosition position, short searchDepth){
        this.searchPosition = position.forSearchDepth(searchDepth);
        Limits.startTime = System.nanoTime();
        selDepth = 0;
        stop = false;
        int alpha = -INF;
        int beta = INF;
        this.currentSearchDepth = 1;
        //SearchResult result = new SearchResult(Optional.empty(), 0);
        int result = 0;

        // Deepen until end conditions
        while (currentSearchDepth <= searchDepth) {

            // Check to see if the time has ended
            //long elapsed = System.currentTimeMillis() - Limits.startTime;
//            if (stop || elapsed >= Limits.timeAllocated / 2 || isScoreCheckmate(result.score()))
//                break;


            result = negamax(position.getState(), currentSearchDepth, 0, alpha, beta);

//            if (result.score <= alpha) {
//                // Failed low, adjust window
//                alpha = -INF;
//            } else if (result.score >= beta){
//                // Failed high, adjust window
//                beta = INF;
//            } else {
//                // Adjust the window around the new score and increase the depth
//                printInfo(position.getState(), result, depth);
//                alpha = result.score - ASPIRATION_WINDOW;
//                beta = result.score + ASPIRATION_WINDOW;
//                depth++;
//                statistics.reset();
//            }
            search.SearchResult sr = new search.SearchResult(Optional.ofNullable(transpositionTable.probe(position.getState().hash, (short)0)).map(MyTtEntry::getMove), result);
            printInfo(position.getState(), sr, currentSearchDepth);
            currentSearchDepth++;
        }

        return new search.SearchResult(Optional.ofNullable(transpositionTable.probe(position.getState().hash, (short)0)).map(MyTtEntry::getMove), result);
    }



    // TODO return short
    public short negamax(BoardState state, int depth, int ply, int alpha, int beta) {
        statistics.nodes++;

        // Check if the current position has been previously evaluated and stored in the transposition table
        MyTtEntry ttEntry = transpositionTable.probe(state.hash, this.currentSearchDepth);
        if (ttEntry != null) {
            // If it has, retrieve the stored score and return it
            return ttEntry.score();
        }

        // If the maximum search depth has been reached, or if the current position is a checkmate, return the static evaluation of the position
        if (depth == 0) {
            return (short) -state.interpolatedScore();
        }

        // Generate all possible moves from the current position
        MoveList moves = state.generateLegalMoves();

        if (moves.size() == 0) {
            // return either checkmate score or stalemate score
            return (short) (state.isKingAttacked() ? -INF + ply : 0);
        }

        // Set the best value to the lowest possible score
        ScoredMove bestMove = WORST_MOVE;

        // Loop through each possible move
        for (Move move : moves.overSorted(state, this.transpositionTable)) {
            // Make the move on the board
            BoardState stateAfterMove = state.doMove(move);

            // Perform a recursive negamax search with the reduced depth and updated alpha and beta values
            // Negate the score to account for the alternating player perspectives in minimax
            short value = (short) -negamax(stateAfterMove, depth - 1, ply + 1, -beta, -alpha);

            // Update the best value if the new value is better
            bestMove = bestMove.betterMove(move, value);

            // Update alpha if the new value is better than the current alpha
            alpha = Math.max(alpha, value);

            // If alpha is greater than or equal to beta, cut off the search to avoid unnecessary work
            if (alpha >= beta) {
                break;
            }
        }

        // Store the best move found during the search in the transposition table for future reference
        transpositionTable.put(state.hash, bestMove.move(), bestMove.score(), currentSearchDepth);

        // Return the best value
        return bestMove.score();
    }


//    public static boolean isScoreCheckmate(int score){
//        return Math.abs(score) >= INF/2;
//    }
//
//    public static boolean canApplyNullWindow(BoardState state, int depth, int beta, boolean inCheck, boolean canApplyNull){
//        return canApplyNull &&
//                !inCheck &&
//                depth >= NULL_MIN_DEPTH &&
//                state.hasNonPawnMaterial(state.getSideToPlay()) &&
//                Evaluation.evaluateState(state) >= beta;
//    }
//
//    public static boolean canApplyLMR(int depth, Move move, int moveIndex){
//        return depth > LMR_MIN_DEPTH &&
//                moveIndex > LMR_MOVES_WO_REDUCTION &&
//                move.flags() == Move.QUIET;
//    }

    public String getPv(BoardState state, int depth){
        MyTtEntry bestEntry = transpositionTable.probe(state.hash(), (short) 0);
        if (bestEntry == null || depth == 0) {
            return "";
        }
        Move bestMove = bestEntry.getMove();
        BoardState newBoardState = state.doMove(bestMove);
        return bestMove.uci() + " " + getPv(newBoardState, depth - 1);
    }

    public void stop(){
        stop = true;
    }

    public void printInfo(BoardState state, search.SearchResult result, int depth){
        streamOut.print("info");
        streamOut.print(" currmove " + result.move().map(Move::toString).orElse("(none)"));
        streamOut.print(" depth " + depth);
        streamOut.print(" seldepth " + selDepth);
        streamOut.print(" time " + (int)(Limits.timeElapsed() / MEGA));
        streamOut.print(" score cp " + result.score());
        streamOut.print(" nodes " + statistics.totalNodes());
        streamOut.printf(" nps %.0f", statistics.totalNodes()/((double)Limits.timeElapsed()/GIGA));
        streamOut.println(" pv " + getPv(state, depth));
    }

    public static void main(String[] args) {
        BoardPosition position = BoardPosition.fromFen(START_POS);
        new Search(new TranspositionTable(), System.out).itDeep(position, (short)6);
    }

    //         Search.stop();
    //        TranspTable.reset();
    //        MoveOrder.clearKillers();
    //        MoveOrder.clearHistory();
    //        System.gc();
    //        Limits.resetTime();
}
