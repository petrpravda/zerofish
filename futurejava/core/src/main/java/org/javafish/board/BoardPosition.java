package org.javafish.board;

import org.javafish.move.Move;

public final class BoardPosition {
    public static final int MAX_GAME_HISTORY_DEPTH = 5871;
    private BoardState state;
    public int historyIndex = 0;
    public long[] history = new long[MAX_GAME_HISTORY_DEPTH];

    public static BoardPosition fromFen(String fen) {
        BoardPosition result = new BoardPosition();
        result.state = BoardState.fromFen(fen);
        return result;
    }

    public BoardState getState() {
        return state;
    }

    public BoardState doMove(Move move) {
        this.state = this.state.doMove(move);
        this.history[this.historyIndex++] = move.bits();
        return this.state;
    }

    public BoardState doMove(String uciMove) {
        Move move = this.state.generateLegalMoves().stream().filter(m -> m.toString().equals(uciMove)).findFirst()
                .orElseThrow();
        return doMove(move);
    }

//    public BoardPosition forSearchDepth(int searchDepth) {
//        BoardPosition result = new BoardPosition();
//        result.state = this.state.forSearchDepth(searchDepth);
//        result.historyIndex = this.historyIndex;
//        result.history = this.history.clone();
//        return result;
//    }
}
