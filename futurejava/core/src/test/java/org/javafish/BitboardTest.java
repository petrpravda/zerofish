package org.javafish;

import org.javafish.bitboard.Bitboard;
import org.javafish.board.Side;
import org.junit.jupiter.api.Test;

import static org.javafish.bitboard.Bitboard.bitboardToString;
import static org.javafish.bitboard.Bitboard.stringToBitboard;
import static org.junit.jupiter.api.Assertions.assertEquals;

class BitboardTest {

    @Test
    void testPawnAttacksWhite() {
        long pawns =           0b00000000_00000000_00000000_00000000_00001000_00000000_00000000_00000000L;
        long expectedAttacks = 0b00000000_00000000_00000000_00010100_00000000_00000000_00000000_00000000L;
        long pawnAttacks = Bitboard.pawnAttacks(pawns, Side.WHITE);
//        System.out.println(bitboardToString(pawnAttacks));
//        System.out.println(bitboardToFormattedBinary(pawnAttacks));

        assertEquals(expectedAttacks, pawnAttacks);
    }

    @Test
    void testPawnAttacksBlack() {
        long pawns =           0b00000000_00000000_00000000_01000001_00000000_00000000_00000000_00000000L;
        long expectedAttacks = 0b00000000_00000000_00000000_00000000_10100010_00000000_00000000_00000000L;
        assertEquals(expectedAttacks, Bitboard.pawnAttacks(pawns, Side.BLACK));
    }

    @Test
    void testGetKnightAttacks() {
        int knightPosition = 38;
        long expectedAttacks = 0b00000000_10100000_00010000_00000000_00010000_10100000_00000000_00000000L;
        long knightAttacks = Bitboard.getKnightAttacks(knightPosition);
//        System.out.println(bitboardToString(knightAttacks));
//        System.out.println(bitboardToFormattedBinary(knightAttacks));
        assertEquals(expectedAttacks, knightAttacks);
    }

    @Test
    void testGetKingAttacks() {
        int kingPosition = 28;
        // long expectedAttacks = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;
        long expectedAttacks = 0b00000000_00000000_00000000_00111000_00101000_00111000_00000000_00000000L;
        long kingAttacks = Bitboard.getKingAttacks(kingPosition);
        // System.out.println(bitboardToString(kingAttacks));
        assertEquals(expectedAttacks, kingAttacks);
    }

    @Test
    void testGetRookAttacks() {
        int rookPosition = 29;  // a1 in a 0-indexed bitboard
        long occupied = 0b01010101_01110011_00001000_00101000_00100010_11010001_01010100_01110101L;
        long expectedAttacks = 0b00000000_00000000_00000000_00100000_11011110_00100000_00100000_00100000L;
        long rookAttacks = Bitboard.getRookAttacks(rookPosition, occupied);
        assertEquals(expectedAttacks, rookAttacks);
    }

    @Test
    void testGetRookAttacks2() {
        int rookPosition = 0;
        long occupied =        0b11111101_11111111_00000001_00000000_00000000_00100000_11111111_10111111L;
        long expectedAttacks = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000001_00000010L;
        long rookAttacks = Bitboard.getRookAttacks(rookPosition, occupied);
        assertEquals(expectedAttacks, rookAttacks);
    }

    @Test
    void testGetBishopAttacks() {
        int bishopPosition = 27;  // e4
        long occupied = 0b01010101_01110011_00001000_00101000_00100010_11010001_01010100_01110101L;
        long expectedAttacks = 0b00000000_01000001_00100010_00010100_00000000_00010100_00000010_00000001L;
        long bishopAttacks = Bitboard.getBishopAttacks(bishopPosition, occupied);
//        System.out.println(bitboardToString(occupied));
//        System.out.println(bitboardToString(bishopAttacks));
//        System.out.println(bitboardToFormattedBinary(bishopAttacks));
        assertEquals(expectedAttacks, bishopAttacks);
    }

    @Test
    public void testGetBishopAttacks2() {
        int bishopPosition = 2;

        long occupied = stringToBitboard("""
                . . . . . X . .
                X . . . . . . X
                . X . X . . . . 
                . X X X . X . . 
                . . X . . . . .
                X . . . . X . . 
                . . . . . . . . 
                . . . . . . . X""");

        // Define the expected attacks in the form of a string, using the stringToBitboard utility
        long expectedAttacks = stringToBitboard(
                ". . . . . . . .\n" +
                ". . . . . . . .\n" +
                ". . . . . . . X\n" +
                ". . . . . . X .\n" +
                ". . . . . X . .\n" +
                "X . . . X . . .\n" +
                ". X . X . . . .\n" +
                ". . . . . . . ."
        );

        // Get the bishop attacks using the occupied bitboard
        long bishopAttacks = Bitboard.getBishopAttacks(bishopPosition, occupied);

        // Compare the generated bishop attacks with the expected attacks
        assertEquals(bitboardToString(bishopAttacks), bitboardToString(expectedAttacks));
    }


    @Test
    void testGetQueenAttacks() {
        int queenPosition = 28;
        long occupied = 0b01010101_01110011_00001000_00101000_00100010_11010001_01010100_01110101L;
        long expectedAttacks = 0b00000000_00010000_00010000_00111000_00101110_00111000_01000100_00000000L;
        long queenAttacks = Bitboard.getQueenAttacks(queenPosition, occupied);
        // System.out.println(bitboardToString(queenAttacks));
        assertEquals(expectedAttacks, queenAttacks);
    }

    @Test
    void testCastlingBlockersKingsideWhite() {
        long expectedMask = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000L;
        assertEquals(expectedMask, Bitboard.castlingBlockersKingsideMask(Side.WHITE));
    }

    @Test
    void testCastlingBlockersKingsideBlack() {
        long expectedMask = 0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;
        assertEquals(expectedMask, Bitboard.castlingBlockersKingsideMask(Side.BLACK));
    }

    @Test
    void testCastlingPiecesKingsideWhite() {
        long expectedMask = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10010000L;
        assertEquals(expectedMask, Bitboard.castlingPiecesKingsideMask(Side.WHITE));
    }

    @Test
    void testCastlingPiecesKingsideBlack() {
        long expectedMask = 0b10010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;
        assertEquals(expectedMask, Bitboard.castlingPiecesKingsideMask(Side.BLACK));
    }

    @Test
    void testBetweenSquares() {
        int squareA = 0;  // a1
        int squareB = 7;  // h1
        long expectedBetween = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01111110L;
        assertEquals(expectedBetween, Bitboard.between(squareA, squareB));
    }

    @Test
    void testPushWhite() {
        long pawn =         0b00000000_00000000_00000000_00000000_00000100_00000000_00000000_00000000L;
        long expectedPush = 0b00000000_00000000_00000000_00000100_00000000_00000000_00000000_00000000L;
        assertEquals(expectedPush, Bitboard.push(pawn, Side.WHITE));
    }

    @Test
    void testPushBlack() {
        long pawn =         0b00000000_00000000_00000000_00000000_00000000_00000000_00000010_00000000L;
        long expectedPush = 0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010L;
        long pushed = Bitboard.push(pawn, Side.BLACK);
        assertEquals(expectedPush, pushed);
    }

    @Test
    void testCastlingPiecesKingsideMask() {
        // Test for white
        long expectedWhiteMask = Bitboard.WHITE_KING_SIDE_CASTLING_BIT_PATTERN;
        assertEquals(expectedWhiteMask, Bitboard.castlingPiecesKingsideMask(Side.WHITE),
                "White king-side castling pieces mask should match");

        // Test for black
        long expectedBlackMask = Bitboard.BLACK_KING_SIDE_CASTLING_BIT_PATTERN;
        assertEquals(expectedBlackMask, Bitboard.castlingPiecesKingsideMask(Side.BLACK),
                "Black king-side castling pieces mask should match");
    }

    @Test
    void testCastlingPiecesQueensideMask() {
        // Test for white
        long expectedWhiteMask = Bitboard.WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN;
        assertEquals(expectedWhiteMask, Bitboard.castlingPiecesQueensideMask(Side.WHITE),
                "White queen-side castling pieces mask should match");

        // Test for black
        long expectedBlackMask = Bitboard.BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN;
        assertEquals(expectedBlackMask, Bitboard.castlingPiecesQueensideMask(Side.BLACK),
                "Black queen-side castling pieces mask should match");
    }

    @Test
    void testCastlingBlockersKingsideMask() {
        // Test for white
        long expectedWhiteMask = Bitboard.WHITE_KING_SIDE_CASTLING_BLOCKERS_PATTERN;
        assertEquals(expectedWhiteMask, Bitboard.castlingBlockersKingsideMask(Side.WHITE),
                "White king-side castling blockers mask should match");

        // Test for black
        long expectedBlackMask = Bitboard.BLACK_KING_SIDE_CASTLING_BLOCKERS_PATTERN;
        assertEquals(expectedBlackMask, Bitboard.castlingBlockersKingsideMask(Side.BLACK),
                "Black king-side castling blockers mask should match");
    }

    @Test
    void testCastlingBlockersQueensideMask() {
        // Test for white
        long expectedWhiteMask = Bitboard.WHITE_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN;
        assertEquals(expectedWhiteMask, Bitboard.castlingBlockersQueensideMask(Side.WHITE),
                "White queen-side castling blockers mask should match");

        // Test for black
        long expectedBlackMask = Bitboard.BLACK_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN;
        assertEquals(expectedBlackMask, Bitboard.castlingBlockersQueensideMask(Side.BLACK),
                "Black queen-side castling blockers mask should match");
    }

    @Test
    public void testRoundTripConversion() {
        // Original bitboard represented as two long parts (lower 32 bits, upper 32 bits)
        long originalBB = 0x12345678FEDCBA98L;

        // Convert bitboard to string and back to bitboard
        String bbString = bitboardToString(originalBB);
        long convertedBB = stringToBitboard(bbString);

        // The final bitboard should be equal to the original bitboard
        assertEquals(bitboardToString(convertedBB), bitboardToString(originalBB));
    }
}


