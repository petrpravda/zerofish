package search;

import org.javafish.move.Move;

import java.util.Optional;

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
