package org.javafish.pgn;

import org.junit.jupiter.api.Test;

import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.List;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

import static org.junit.jupiter.api.Assertions.assertEquals;


public class PgnToUciTest extends PgnParserBase {

    @Test
    public void pgnToUciBasic() {
        PgnMoves pgnMoves = PgnParser.fromSan(SIX_MOVES_SAN);
        assertEquals(SIX_MOVES_SAN, pgnMoves.asSan());
        assertEquals(SIX_MOVES_UCI, pgnMoves.asUci());
    }

    @Test
    public void pgnToUciFullGame() {
//        MoveList moveList = new MoveList();
//        moveList.loadFromSan(FULL_MOVES_SAN);
//        System.out.println(moveList.toString());

        PgnMoves pgnMoves = PgnParser.fromSan(FULL_MOVES_SAN);
        assertEquals(FULL_MOVES_SAN, pgnMoves.asSan());
        assertEquals(FULL_MOVES_UCI, pgnMoves.asUci());
    }

    @Test
    public void pgnToUciPromotion() {
        PgnMoves pgnMoves = PgnParser.fromSan(PROMOTION_MOVES_SAN);
        assertEquals(PROMOTION_MOVES_SAN, pgnMoves.asSan());
        assertEquals(PROMOTION_MOVES_UCI, pgnMoves.asUci());
    }

    @Test
    public void pgnToUciEnPassant() {
        PgnMoves pgnMoves = PgnParser.fromSan("exf6");
        assertEquals("e5f6", pgnMoves.asUciFromPosition("3r1rk1/1R4pp/p3p3/4Pp2/2p1Q3/2q5/2B3PP/4RK2 w - f6 0 25"));
    }

    @Test
    public void pgnToUciComplex() {
        PgnMoves pgnMoves = PgnParser.fromSan(COMPLEX_MOVES_SAN);
        assertEquals(COMPLEX_MOVES_SAN, pgnMoves.asSan());
        assertEquals(COMPLEX_MOVES_UCI, pgnMoves.asUci());
    }

    @Test
    public void pgnToUciPromotionWoCapture() {
        PgnMoves pgnMoves = PgnParser.fromSan(PROMOTION_WO_CAPTURE_MOVES_SAN);
        assertEquals(PROMOTION_WO_CAPTURE_MOVES_SAN, pgnMoves.asSan());
        assertEquals(PROMOTION_WO_CAPTURE_MOVES_UCI, pgnMoves.asUci());
    }

    @Test
    public void pgnToUciNumberedInputWithNewlines() {
        PgnMoves pgnMoves = PgnParser.fromSan("""
               1. e4 c6 2. d4 d5 3. exd5 cxd5 4. Nf3 Nf6 5. Nc3 e6 6. Bf4 Bb4 7. a3 Bxc3+ 8.
               bxc3 Ne4 9. Bxb8 Rxb8 10. Qd3 Qc7 11. Rb1 Nxc3 12. Rb3 Ne4 13. Nd2 Nxd2 14. Qxd2
               O-O 15. Be2 Bd7 16. O-O Rbc8 17. Rfb1 b6 18. c3 e5 19. Ba6 Rce8 20. f3 Ba4 21.
               Rb4 Bc6 22. Re1 exd4 23. Rxd4 b5 24. Re3 Qa5 25. Rxe8 Rxe8 26. a4 Qxa6 27. axb5
               Qxb5 28. Kf2 Qc5 29. f4 Bb5 30. g3 Re2+ 31. Qxe2 Bxe2 32. Kxe2 a5 33. Kd2 Qa3
               34. Ke2 Qa2+ 35. Kf3 f5 36. g4 Qc2 37. Ke3 Qxc3+ 38. Rd3 Qe1+ 39. Kd4 Qb4+ 40.
               Ke3 Qe4+ 41. Kd2 Qxf4+ 42. Kc3 Qb4+ 43. Kc2 Qxg4 44. Rd2 a4 45. Kc1 a3 46. Kb1
               Qe4+ 47. Ka2 Qc4+ 48. Kb1 Qb3+ 49. Ka1 Qc3+ 50. Ka2 Qxd2+ 51. Kxa3 Qxh2 52. Kb4
               Qc2 53. Ka5 Qb1 54. Ka4 Qb6 55. Ka3 Qb1 56. Ka4 Qb6 57. Ka3 Qb1 58. Ka4 1/2-1/2""");
//        assertEquals(PROMOTION_WO_CAPTURE_MOVES_SAN, pgnMoves.asSan());
//        assertEquals(PROMOTION_WO_CAPTURE_MOVES_UCI, pgnMoves.asUci());
    }

    public record Quad(String pgn, String uci, String asUci, String asSan) {

    }

    @Test
    public void pgnToUci5000() {
        InputStream pgnStream = PgnToUciTest.class.getResourceAsStream("/org/javafish/pgn/pgn.5000.games.txt");
        InputStream uciStream = PgnToUciTest.class.getResourceAsStream("/org/javafish/pgn/uci.5000.games.txt");
        List<String> pgns = new BufferedReader(new InputStreamReader(pgnStream)).lines().collect(Collectors.toList());
        List<String> ucis = new BufferedReader(new InputStreamReader(uciStream)).lines().collect(Collectors.toList());

        List<Quad> quads = IntStream.range(0, pgns.size()).parallel()
                .mapToObj(game -> {
                    String pgn = pgns.get(game);
                    String uci = ucis.get(game);

                    PgnMoves pgnMoves = PgnParser.fromSan(pgn);
                    String asUci = pgnMoves.asUci();
                    String asSan = null;
                    try {
                        asSan = pgnMoves.asSan();
                    } catch (Exception e) {
                        throw new IllegalStateException(String.format("Cannot parse: %s: %s", pgn, e.getMessage()), e);
                    }
                    return new Quad(pgn, uci, asUci, asSan);
                }).toList();

        for (Quad quad : quads) {
            assertEquals(quad.pgn, quad.asSan);
            assertEquals(quad.uci, quad.asUci);
        }
//
//        for (int game = 0; game < pgns.size(); game++) {
//            assertEquals(pgn, pgnMoves.asSan());
//            try {
//                assertEquals(uci, pgnMoves.asUci());
//            } catch (Exception e) {
//                throw new IllegalStateException(String.format("Cannot parse: %s: %s", pgn, e.getMessage()), e);
//            }
//        }
    }
}
