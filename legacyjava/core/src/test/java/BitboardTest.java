import org.javafish.bitboard.Bitboard;
import org.javafish.board.Side;
import org.junit.jupiter.api.Test;

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
                //Bitboard.FULL_BOARD & ~(1L << rookPosition); // Full board without the rook
        long expectedAttacks = 0b00000000_00000000_00000000_00100000_11011110_00100000_00100000_00100000L;
        long rookAttacks = Bitboard.getRookAttacks(rookPosition, occupied);
//        System.out.println(bitboardToString(occupied));
//        System.out.println(bitboardToString(rookAttacks));
//        System.out.println(bitboardToFormattedBinary(rookAttacks));
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
    void testGetQueenAttacks() {
        int queenPosition = 27;  // e4
        long occupied = Bitboard.FULL_BOARD & ~(1L << queenPosition); // Full board without the queen
        long expectedAttacks = Bitboard.getRookAttacks(queenPosition, occupied) |
                Bitboard.getBishopAttacks(queenPosition, occupied);
        assertEquals(expectedAttacks, Bitboard.getQueenAttacks(queenPosition, occupied));
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
}


