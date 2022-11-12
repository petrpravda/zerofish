package org.javafish.pgn;


import org.junit.jupiter.api.BeforeEach;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class UciToPgnTest extends PgnParserBase {

//    private BoardState state;

    @BeforeEach
    public void setUp() {
//        BoardPosition boardPosition = fromFen(START_POS);
//        this.state = BoardState.fromPosition(boardPosition);
    }

//    @Test
//    public void uciToPgnBasic() {
//        UciMoves uciMoves = PgnParser.fromUci(SIX_MOVES_UCI, state);
//        assertEquals(SIX_MOVES_UCI, uciMoves.asUci());
//        assertEquals(SIX_MOVES_SAN, uciMoves.asSan());
//    }
//
//    @Test
//    public void uciToPgnFull() {
//        UciMoves uciMoves = PgnParser.fromUci(FULL_MOVES_UCI, state);
//        assertEquals(FULL_MOVES_UCI, uciMoves.asUci());
//        assertEquals(FULL_MOVES_SAN, uciMoves.asSan());
//    }
//
//    @Test
//    public void uciToPgnPromotion() {
//        UciMoves uciMoves = PgnParser.fromUci(PROMOTION_MOVES_UCI, state);
//        assertEquals(PROMOTION_MOVES_UCI, uciMoves.asUci());
//        assertEquals(PROMOTION_MOVES_SAN, uciMoves.asSan());
//    }
//
//    @Test
//    public void uciToPgnComplex() {
//        UciMoves uciMoves = PgnParser.fromUci(COMPLEX_MOVES_UCI, state);
//        assertEquals(COMPLEX_MOVES_UCI, uciMoves.asUci());
//        assertEquals(COMPLEX_MOVES_SAN, uciMoves.asSan());
//    }
//
//    @Test
//    public void uciToPgnPromotionWoCapture() {
//        UciMoves uciMoves = PgnParser.fromUci(PROMOTION_WO_CAPTURE_MOVES_UCI, state);
//        assertEquals(PROMOTION_WO_CAPTURE_MOVES_UCI, uciMoves.asUci());
//        assertEquals(PROMOTION_WO_CAPTURE_MOVES_SAN, uciMoves.asSan());
//    }
//
//    @Test
//    public void uciToPgn5000() {
//        InputStream pgnStream = PgnToUciTest.class.getResourceAsStream("/org/babayaga/pgn/pgn.5000.games.txt");
//        InputStream uciStream = PgnToUciTest.class.getResourceAsStream("/org/babayaga/pgn/uci.5000.games.txt");
//        List<String> pgns = new BufferedReader(new InputStreamReader(pgnStream)).lines().collect(Collectors.toList());
//        List<String> ucis = new BufferedReader(new InputStreamReader(uciStream)).lines().collect(Collectors.toList());
//
//
//        List<PgnToUciTest.Quad> quads = IntStream.range(0, pgns.size()).parallel()
//                .mapToObj(game -> {
//                    String pgn = pgns.get(game);
//                    String uci = ucis.get(game);
//                    BoardState boardState = BoardState.fromPosition(fromFen(START_POS));
//
//                    UciMoves uciMoves = PgnParser.fromUci(uci, boardState);
//                    return new PgnToUciTest.Quad(pgn, uci, uciMoves.asUci(), uciMoves.asSan());
//                }).toList();
//
//        for (PgnToUciTest.Quad quad : quads) {
//            assertEquals(quad.pgn(), quad.asSan());
//            assertEquals(quad.uci(), quad.asUci());
//        }
//    }
}
