package org.javafish.app;

import org.javafish.board.BoardState;
import org.javafish.move.Move;
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
        // BoardState board = BoardState.fromFen(START_POS);
        //BoardState board = BoardState.fromFen("rnbqkbnr/pppppppp/8/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 0 1");
        BoardState board = BoardState.fromFen("rnbqkb1r/pppppppp/8/8/4n3/3P4/PPPKPPPP/RNBQ1BNR w kq - 3 3");

//        BoardState board = BoardState.fromFen("8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1");
//        int moves = 6;
//        board = board.doMove(Move.fromUciString("b3a4", board)); moves--;
//        board = board.doMove(Move.fromUciString("f7e8", board)); moves--;
//        board = board.doMove(Move.fromUciString("c5c6", board)); moves--;
//        board = board.doMove(Move.fromUciString("e8d7", board)); moves--;
//        System.out.println(board.toFen());
        System.out.println(perftString(board, 1));
    }
}
