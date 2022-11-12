import org.javafish.board.BoardState;
import org.junit.jupiter.api.DynamicTest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.TestFactory;

import java.util.Scanner;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Stream;

import static org.javafish.board.BoardState.fromFen;
import static org.javafish.app.Perft.perft;
import static org.javafish.board.Fen.START_POS;
import static org.junit.jupiter.api.Assertions.assertEquals;


class PerftTest {

    // http://www.talkchess.com/forum3/viewtopic.php?t=47318
    private static final String TRICKY_PERFTS = """
        avoid illegal en passant capture:
        8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1 perft 6 = 824064
        8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1 perft 6 = 824064
        en passant capture checks opponent:
        8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1 perft 6 = 1440467
        8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1 perft 6 = 1440467
        short castling gives check:
        5k2/8/8/8/8/8/8/4K2R w K - 0 1 perft 6 = 661072
        4k2r/8/8/8/8/8/8/5K2 b k - 0 1 perft 6 = 661072
        long castling gives check:
        3k4/8/8/8/8/8/8/R3K3 w Q - 0 1 perft 6 = 803711
        r3k3/8/8/8/8/8/8/3K4 b q - 0 1 perft 6 = 803711
        castling (including losing cr due to rook capture):
        r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1 perft 4 = 1274206
        r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1 perft 4 = 1274206
        castling prevented:
        r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1 perft 4 = 1720476
        r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1 perft 4 = 1720476
        promote out of check:
        2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1 perft 6 = 3821001
        3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1 perft 6 = 3821001
        discovered check:
        5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1 perft 5 = 1004658
        8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1 perft 5 = 1004658
        promote to give check:
        4k3/1P6/8/8/8/8/K7/8 w - - 0 1 perft 6 = 217342
        8/k7/8/8/8/8/1p6/4K3 b - - 0 1 perft 6 = 217342
        underpromote to check:
        8/P1k5/K7/8/8/8/8/8 w - - 0 1 perft 6 = 92683
        8/8/8/8/8/k7/p1K5/8 b - - 0 1 perft 6 = 92683
        self stalemate:
        K1k5/8/P7/8/8/8/8/8 w - - 0 1 perft 6 = 2217
        8/8/8/8/8/p7/8/k1K5 b - - 0 1 perft 6 = 2217
        stalemate/checkmate:
        8/k1P5/8/1K6/8/8/8/8 w - - 0 1 perft 7 = 567584
        8/8/8/8/1k6/8/K1p5/8 b - - 0 1 perft 7 = 567584
        double check:
        8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1 perft 4 = 23527
        8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1 perft 4 = 23527
        """;

    @Test
    void simplePerft2() {
        BoardState state = fromFen(START_POS);
        assertEquals(400, perft(state, 2));
    }

    @Test
    void simplePerft3() {
        BoardState state = fromFen(START_POS);
        assertEquals(8902, perft(state, 3));
    }

    @Test
    void simplePerft4() {
        BoardState state = fromFen(START_POS);
        assertEquals(197281, perft(state, 4));
    }

    @Test
    void simplePerft5() {
        BoardState board = fromFen(START_POS);
        assertEquals(4865609, perft(board, 5));
    }

    @Test
    void simplePerft5b() {
        BoardState board = fromFen("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1");
        assertEquals(5363555, perft(board, 5));
    }

    @Test
    void simpleEnPassant() {
        BoardState state = fromFen("rnbqkbnr/p2ppppp/2p5/Pp6/8/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 3");
        assertEquals(23, perft(state, 1));
    }

    @Test
    void simpleEnPassant2() {
        BoardState state = fromFen("r1b2rk1/p3npbp/2p1p1p1/2qpP3/8/2P2N2/PP1N1PPP/R1BQR1K1 w Qq d6 1 1");
        assertEquals(29, perft(state, 1));
    }

    @TestFactory
    Stream<DynamicTest> trickyPerfts() {
        Pattern pattern = Pattern.compile("(.*?):?\\n(.*?)\\n(.*?)\\n");
        return new Scanner(TRICKY_PERFTS).findAll(pattern)
                .map(matcher -> {
                    String description = matcher.group(1);
                    String testWhite = matcher.group(2);
                    String testBlack = matcher.group(3);
                    return DynamicTest.dynamicTest(description, () -> {
                        PerftCase perftWhite = parsePerftCase(testWhite);
                        PerftCase perftBlack = parsePerftCase(testBlack);

                        assertEquals(perftWhite.count, perft(fromFen(perftWhite.fen), perftWhite.depth));
                        assertEquals(perftBlack.count, perft(fromFen(perftBlack.fen), perftBlack.depth));
                        // assertEquals(outout.get(id), translate(word));
                    });
                });
    }


    public record PerftCase(String fen, int depth, int count) {
    }

    private static final Pattern PERFT_CASE_REGEX = Pattern.compile("(.+) perft (\\d+) = (\\d+)");
    private static PerftCase parsePerftCase(String line) {
        Matcher matcher = PERFT_CASE_REGEX.matcher(line);
        if (matcher.find()) {
            return new PerftCase(matcher.group(1), Integer.parseInt(matcher.group(2)), Integer.parseInt(matcher.group(3)));
        } else {
            throw new IllegalArgumentException(String.format("Line \"%s\" cannot be parsed.", line));
        }
    }
}
