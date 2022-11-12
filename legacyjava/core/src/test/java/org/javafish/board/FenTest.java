package org.javafish.board;

import org.junit.jupiter.api.Test;

import static org.javafish.bitboard.Bitboard.WHITE_KINGS_ROOK_MASK;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class FenTest {
    @Test
    public void kingsideCastlingFlagsRookMoving() {
        BoardPosition position = Fen.fromFenFree("""
                 +---+---+---+---+---+---+---+---+
                 | r |   |   |   | k |   |   | r |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 | R |   |   |   | K |   |   | R |
                 +---+---+---+---+---+---+---+---+
                                
                Fen: r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 1 1
                Key: 6A7D540DF1022A9E
                Checkers:
                """);
        BoardState state = position.getState();
        assertEquals(0L, state.movements);
        state = position.doMove("h1g1");
        assertTrue((state.movements & WHITE_KINGS_ROOK_MASK) != 0L);
        assertTrue(state.toFen().contains("Qkq"));
    }

    @Test
    public void kingsideCastlingFlagsKingMoving() {
        BoardPosition position = Fen.fromFenFree("""
                 +---+---+---+---+---+---+---+---+
                 | r |   |   |   | k |   |   | r |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 |   |   |   |   |   |   |   |   |
                 +---+---+---+---+---+---+---+---+
                 | R |   |   |   | K |   |   | R |
                 +---+---+---+---+---+---+---+---+
                                
                Fen: r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 1 1
                Key: 6A7D540DF1022A9E
                Checkers:
                """);
        BoardState state = position.getState();
        assertEquals(0L, state.movements);
        state = position.doMove("e1d1");
        assertTrue(state.toFen().contains("kq"));
    }

    @Test
    public void sanitizeCrappyCastlingFlags() {
        BoardPosition position = Fen.fromFenFree("""
                         +---+---+---+---+---+---+---+---+
                         | r |   |   | q |   | r | k |   | 8
                         +---+---+---+---+---+---+---+---+
                         | p | b | p |   |   | p | p | p | 7
                         +---+---+---+---+---+---+---+---+
                         |   | p | n | p |   | n |   |   | 6
                         +---+---+---+---+---+---+---+---+
                         |   |   |   |   |   |   |   |   | 5
                         +---+---+---+---+---+---+---+---+
                         |   |   | P | P | p |   |   |   | 4
                         +---+---+---+---+---+---+---+---+
                         |   |   | P |   | P |   |   |   | 3
                         +---+---+---+---+---+---+---+---+
                         | P |   | N |   |   | P | P | P | 2
                         +---+---+---+---+---+---+---+---+
                         | R |   | B | Q | K | B |   | R | 1
                         +---+---+---+---+---+---+---+---+
                           a   b   c   d   e   f   g   h
                                                
                        Fen: r2q1rk1/pbp2ppp/1pnp1n2/8/2PPp3/2P1P3/P1N2PPP/R1BQKB1R w kq - 0 10
                        Key: CE0A996DEF8E74EC
                        Checkers:\s""");
        assertEquals("r2q1rk1/pbp2ppp/1pnp1n2/8/2PPp3/2P1P3/P1N2PPP/R1BQKB1R w - - 0 10", position.getState().toFen());
    }
}
