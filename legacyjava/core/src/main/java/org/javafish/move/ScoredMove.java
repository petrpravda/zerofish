package org.javafish.move;

import java.util.StringJoiner;

public class ScoredMove {
    final Move move;
    final int score;

    public ScoredMove(Move move, int score) {
        this.move = move;
        this.score = score;
    }

//    public Move getMove() {
//        return move;
//    }
//
//    public int getScore() {
//        return score;
//    }

    @Override
    public String toString() {
        return new StringJoiner(", ", ScoredMove.class.getSimpleName() + "[", "]")
                .add("move=" + move)
                .add("score=" + score)
                .toString();
    }
}
