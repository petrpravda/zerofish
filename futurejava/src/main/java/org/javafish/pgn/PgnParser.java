package org.javafish.pgn;


import org.javafish.board.BoardState;

import java.util.Arrays;
import java.util.List;

public class PgnParser {
    public static PgnMoves fromSan(String san) {
        String sanitized = stripMoveNumbers(san);
        return fromSan(Arrays.asList(sanitized.split(" +")));
    }

    private static String stripMoveNumbers(String san) {
        return san.replaceAll("\\d+\\.", "")
                .replaceAll("1/2-1/2", "")
                .replaceAll("\\s+", " ")
                .trim();
    }

    public static PgnMoves fromSan(List<String> sans) {
        PgnMoves result = new PgnMoves();
        result.setMoveStrings(sans);
        return result;
    }

    public static UciMoves fromUci(String uciMoves, BoardState state) {
        return fromUci(Arrays.asList(uciMoves.split(" +")), state);
    }

    public static UciMoves fromUci(List<String> uciMoves, BoardState state) {
        UciMoves result = new UciMoves(state);
        result.setMoveStrings(uciMoves);
        return result;
    }
}
