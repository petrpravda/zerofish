package org.javafish.move;

import org.javafish.board.BoardState;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MoveTest {

    @Test
    void testMoveConstructorWithFlags() {
        Move move = new Move(12, 28, Move.CAPTURE);
        assertEquals(12, move.from());
        assertEquals(28, move.to());
        assertEquals(Move.CAPTURE, move.flags());
    }

    @Test
    void testMoveConstructorWithoutFlags() {
        Move move = new Move(12, 28);
        assertEquals(12, move.from());
        assertEquals(28, move.to());
        assertEquals(Move.QUIET, move.flags());
    }

    @Test
    void testToMethod() {
        Move move = new Move(5, 23, Move.QUIET);
        assertEquals(23, move.to());
    }

    @Test
    void testFromMethod() {
        Move move = new Move(5, 23, Move.QUIET);
        assertEquals(5, move.from());
    }

    @Test
    void testFlagsMethod() {
        Move move = new Move(12, 28, Move.EN_PASSANT);
        assertEquals(Move.EN_PASSANT, move.flags());
    }

    @Test
    void testIsPromotion() {
        Move movePromotion = new Move(12, 28, Move.PR_QUEEN);
        assertTrue(movePromotion.isPromotion());

        Move moveNonPromotion = new Move(12, 28, Move.QUIET);
        assertFalse(moveNonPromotion.isPromotion());
    }

    @Test
    void testEquals() {
        Move move1 = new Move(12, 28, Move.CAPTURE);
        Move move2 = new Move(12, 28, Move.CAPTURE);
        assertEquals(move1, move2);

        Move move3 = new Move(12, 28, Move.QUIET);
        assertNotEquals(move1, move3);
    }

    @Test
    void testToStringForNonNullMove() {
        Move move = new Move(12, 28, Move.QUIET);
        assertNotEquals("NULL_MOVE", move.toString());
    }

    @Test
    void testIsNullMove() {
        Move move = new Move(0, 0, Move.NULL);
        assertTrue(move.isNullMove());

        Move nonNullMove = new Move(12, 28, Move.QUIET);
        assertFalse(nonNullMove.isNullMove());
    }

    @Test
    void testUciMethodForQuietMove() {
        Move move = new Move(40, 48, Move.QUIET);
        assertEquals("a6a7", move.uci());
    }

    @Test
    void testUciMethodForPromotionMove() {
        Move promotionMove = new Move(48, 56, Move.PR_QUEEN);
        assertEquals("a7a8q", promotionMove.uci());
    }

    @Test
    void testGetPieceType() {
        Move move = new Move(12, 28, Move.CAPTURE);
        assertEquals(1, move.getPieceType());

        Move promotionMove = new Move(12, 28, Move.PC_QUEEN);
        assertEquals(4, promotionMove.getPieceType());
    }

    @Test
    void testGetPieceTypeForSide() {
        Move move = new Move(12, 28, Move.CAPTURE);
        assertEquals(9, move.getPieceTypeForSide(1)); // Adding for side 1
    }

    @Test
    void testIsCastling() {
        Move moveOO = new Move(4, 6, Move.OO);
        assertTrue(moveOO.isCastling());

        Move moveOOO = new Move(4, 2, Move.OOO);
        assertTrue(moveOOO.isCastling());

        Move regularMove = new Move(4, 6, Move.QUIET);
        assertFalse(regularMove.isCastling());
    }

    @Test
    void testFromUciStringForPromotion() {
        BoardState state = BoardState.fromFen("6k1/P7/4p1B1/4B3/8/2Q5/2PK1P1P/R6R w - - 1 40"); // Mock or dummy object
        Move move = Move.fromUciString("a7a8q", state); // e7 -> e8 with promotion to queen
        assertNotNull(move);
        assertEquals("a7a8q", move.uci());
    }

    @Test
    void testFromUciStringForNormalMove() {
        BoardState state = BoardState.fromFen("5k2/8/P3p1B1/4B3/8/2Q5/2PK1P1P/R6R w - - 1 39"); // Mock or dummy object
        Move move = Move.fromUciString("a6a7", state);
        assertNotNull(move);
        assertEquals("a6a7", move.uci());
    }

    @Test
    void testNullMoveConstant() {
        assertEquals(Move.NULL_MOVE, new Move(0, 0, Move.NULL));
    }
}

