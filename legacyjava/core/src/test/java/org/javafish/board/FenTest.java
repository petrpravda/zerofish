package org.javafish.board;

import org.javafish.bitboard.Bitboard;
import org.junit.jupiter.api.Test;

import static org.javafish.bitboard.Bitboard.WHITE_KINGS_ROOK_MASK;
import static org.javafish.bitboard.Bitboard.bitboardToFormattedBinary;
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

    @Test
    void toFen() {
    }

    @Test
    void expandFenPieces() {
    }

    @Test
    void fromFen() {
        String fen = "r1b1r1k1/pp2qpp1/3b4/3p1n2/1P3n2/P3p1pP/2P1N1B1/R1Q1BKR1 w - - 0 21";
        BoardState boardState = Fen.fromFen(fen, 10);

        long combinedOccupancy = 0L;
        for (int side = 0; side <= 1; side++) {

            // Iterate over pieces: 0 -> Pawn, 1 -> Knight, 2 -> Bishop, 3 -> Rook, 4 -> Queen, 5 -> King
            for (int pieceType = 0; pieceType <= 5; pieceType++) {
                // OR the results for all piece types on the current side
                combinedOccupancy |= boardState.bitboardOf(side, pieceType);
            }


            // Example assertion if you have expected occupancy values:
            // long expectedOccupancy = <expected value for side>;
            // assertEquals(expectedOccupancy, combinedOccupancy);
        }
        //System.out.println(combinedOccupancy);

        // Print the combined occupancy in binary format
//        System.out.println("Combined Occupancy (binary): " + bitboardToFormattedBinary(combinedOccupancy));
//        System.out.println(Bitboard.bitboardToString(combinedOccupancy));
    }

    @Test
    void fromFenFree() {
    }
}
