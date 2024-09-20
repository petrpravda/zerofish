package org.javafish.board;

import org.junit.jupiter.api.Test;

import static org.javafish.bitboard.Bitboard.WHITE_KINGS_ROOK_MASK;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class FenTest {
    @Test
    void testToFen() {
        // Arrange
        BoardState state = BoardState.fromFen(Fen.START_POS);

        // Act
        String fen = Fen.toFen(state);

        // Assert
        assertEquals(Fen.START_POS, fen, "FEN should match the initial position");
    }

    @Test
    void testFromFen() {
        // Arrange
        String fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        // Act
        BoardState state = Fen.fromFen(fen, null);

        // Assert
        assertNotNull(state, "BoardState should not be null");
        assertEquals(Side.WHITE, state.getSideToPlay(), "Side to play should be WHITE");
        assertEquals(0, state.halfMoveClock, "Halfmove clock should be 0");
        assertEquals(1, (state.fullMoveNormalized / 2) + 1, "Full move number should be 1");
    }

    @Test
    void testExpandFenPieces() {
        // Arrange
        String fenPieces = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

        // Act
        String expandedFen = Fen.expandFenPieces(fenPieces);

        // Assert
        String expectedExpandedFen = "rnbqkbnr/pppppppp/11111111/11111111/11111111/11111111/PPPPPPPP/RNBQKBNR";
        assertEquals(expectedExpandedFen, expandedFen, "Expanded FEN should match expected format");
    }

    @Test
    void testFromFenFree_Valid() {
        // Arrange
        String fenFree = "Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

        // Act
        BoardPosition boardPosition = Fen.fromFenFree(fenFree);

        // Assert
        assertNotNull(boardPosition, "BoardPosition should not be null");
    }

    @Test
    void testFromFenFree_Invalid() {
        // Arrange
        String invalidFenFree = "This is an invalid FEN string";

        // Act & Assert
        Exception exception = assertThrows(IllegalArgumentException.class, () -> {
            Fen.fromFenFree(invalidFenFree);
        });
        assertEquals("This is an invalid FEN string doesn't contain 'Fen: '", exception.getMessage());
    }

    @Test
    void testFromFen_ComplexPosition() {
        // Arrange
        String fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 2";

        // Act
        BoardState state = Fen.fromFen(fen, null);

        // Assert
        assertNotNull(state, "BoardState should not be null");
        assertEquals(Side.BLACK, state.getSideToPlay(), "Side to play should be BLACK");
        assertEquals(2, (state.fullMoveNormalized / 2) + 1, "Full move number should be 2");
        assertEquals(0, state.halfMoveClock, "Halfmove clock should be 0");
        assertEquals("e3", Square.getName(Long.numberOfTrailingZeros(state.enPassant)), "En passant square should be e3");
    }

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
                 | r |   |   |   | K |   |   | r |
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
                 | r |   |   |   | K |   |   | r |
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
                         | r |   | B | Q | K | B |   | r | 1
                         +---+---+---+---+---+---+---+---+
                           a   b   c   d   e   f   g   h
                                                
                        Fen: r2q1rk1/pbp2ppp/1pnp1n2/8/2PPp3/2P1P3/P1N2PPP/R1BQKB1R w kq - 0 10
                        Key: CE0A996DEF8E74EC
                        Checkers:\s""");
        assertEquals("r2q1rk1/pbp2ppp/1pnp1n2/8/2PPp3/2P1P3/P1N2PPP/R1BQKB1R w - - 0 10", position.getState().toFen());
    }
}
