package org.javafish.app;

import org.javafish.board.BoardState;
import org.javafish.move.MoveList;

import java.util.List;
import java.util.concurrent.atomic.AtomicLong;

import static org.javafish.board.BoardState.fromFen;
import static org.javafish.board.Fen.START_POS;

public class Perft {
    public static long perft(BoardState state, int depth) {
        MoveList moves = state.generateLegalMoves();

        if (depth == 1) {
            return moves.size();
        }
        AtomicLong nodes = new AtomicLong();
        moves
                //.stream()
                .forEach(move -> {
                    BoardState newBoardState = state.doMove(move);
                    long c1 = perft(newBoardState, depth - 1);
                    nodes.addAndGet(c1);
                });

        return nodes.get();
    }

    public static long perft2(BoardState state, int depth) {
        MoveList moves = state.generateLegalMoves();

        if (depth == 0) {
            return 1;
        }
        AtomicLong nodes = new AtomicLong();
        moves
                .stream()
                .forEach(move -> {
                    BoardState newBoardState = state.doMove(move);
                    if (depth >= 1) {
                        long c1 = perft2(newBoardState, depth - 1);
                        nodes.addAndGet(c1);
                    }
                });

        return nodes.get();
    }

    public static String perftString(BoardState state, int depth) {
        MoveList moveList = state.generateLegalMoves();
        AtomicLong nodes = new AtomicLong();

        List<String> list = moveList.stream()
                .map(move -> {
                    long count = 0;

                    if (depth > 1) {
                        BoardState newBoardState = state.doMove(move);
                        count += perft(newBoardState, depth - 1);
                        // board.undo_move(move, moveResult);
                    } else {
                        count += 1;
                    }

                    nodes.addAndGet(count);
                    return String.format("%s: %d", move, count);
                }).toList();

        String tableData = String.join("\n", list);
        return String.format("%s\n\nNodes searched: %d", tableData, nodes.get());
    }

    public static void main(String[] args) {
        BoardState board = BoardState.fromFen("r1bqkbnr/pppppppp/n7/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2");
        //BoardState state = fromFen("r6r/3k4/8/8/3Q4/3q4/8/3RK2R b K - 3 2");
        //System.out.println(perftString(board, 5));
        System.out.println(perftString(board, 1));
    }
}
