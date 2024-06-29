import org.javafish.board.BoardState;
import org.javafish.move.Move;
import org.javafish.move.MoveList;

import static org.javafish.app.Benchmark.GIGA;
import static org.javafish.board.BoardState.fromFen;
import static org.javafish.board.Fen.START_POS;

public class Perft {

    public static long printPerft(BoardState state, int depth) {
        final long startTime = System.nanoTime();
        long nodes = 0;
        MoveList moves = state.generateLegalMoves();
        for (Move move : moves){
            System.out.print(move.uci() + " ");
            BoardState newState = state.doMove(move);
            long move_nodes = perft(newState, depth - 1);
            nodes += move_nodes;
            System.out.println(move_nodes);
        }
        final long end_time = System.nanoTime();
        double time_taken = ((double)(end_time - startTime)) / GIGA;
        System.out.println("Nodes: " + nodes);
        System.out.println("Time: " + time_taken + "s");
        System.out.println("NPS: " + nodes/time_taken);
        return nodes;
    }

    public static long perft(BoardState state, int depth) {
        MoveList moves = state.generateLegalMoves();
        if (depth == 1) {
            return moves.size();
        }
        long nodes = 0;
        for (Move move : moves) {
            BoardState newState = state.doMove(move);
            nodes += perft(newState, depth - 1);
        }
        return nodes;
    }

    public static void main(String[] args) {
        printPerft(fromFen(START_POS), 7);
    }
}
