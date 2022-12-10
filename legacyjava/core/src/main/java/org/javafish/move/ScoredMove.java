package org.javafish.move;

import java.util.StringJoiner;

public record ScoredMove(Move move, short score) {

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

    public ScoredMove betterMove(Move move, short value) {
        return value > this.score ? move.scored(value) : this;
    }
}
