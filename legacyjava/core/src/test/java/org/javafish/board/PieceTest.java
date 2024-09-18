package org.javafish.board;

import org.junit.jupiter.api.Test;

import static org.javafish.board.Piece.WHITE_BISHOP;
import static org.javafish.board.Piece.WHITE_KING;
import static org.javafish.board.Piece.WHITE_KNIGHT;
import static org.javafish.board.Piece.WHITE_PAWN;
import static org.javafish.board.Piece.WHITE_QUEEN;
import static org.javafish.board.Piece.WHITE_ROOK;
import static org.junit.jupiter.api.Assertions.assertEquals;

public class PieceTest {

    @Test
    void testFlip() {
        // Test flipping between white and black pieces
        assertEquals(Piece.BLACK_PAWN, Piece.flip(WHITE_PAWN));
        assertEquals(WHITE_KNIGHT, Piece.flip(Piece.BLACK_KNIGHT));
        assertEquals(Piece.BLACK_BISHOP, Piece.flip(WHITE_BISHOP));
        assertEquals(Piece.WHITE_ROOK, Piece.flip(Piece.BLACK_ROOK));
        assertEquals(Piece.BLACK_QUEEN, Piece.flip(Piece.WHITE_QUEEN));
        assertEquals(Piece.WHITE_KING, Piece.flip(Piece.BLACK_KING));
    }

    @Test
    void testTypeOf() {
        // Test getting the piece type without regard to color
        assertEquals(WHITE_PAWN, Piece.typeOf(WHITE_PAWN));      // Pawn
        assertEquals(WHITE_KNIGHT, Piece.typeOf(Piece.BLACK_KNIGHT));    // Knight
        assertEquals(WHITE_BISHOP, Piece.typeOf(WHITE_BISHOP));    // Bishop
        assertEquals(WHITE_ROOK, Piece.typeOf(Piece.BLACK_ROOK));      // Rook
        assertEquals(WHITE_QUEEN, Piece.typeOf(Piece.WHITE_QUEEN));     // Queen
        assertEquals(WHITE_KING, Piece.typeOf(Piece.BLACK_KING));      // King
    }

    @Test
    void testSideOf() {
        // Test getting the side (white or black)
        assertEquals(Side.WHITE, Piece.sideOf(WHITE_PAWN));      // White
        assertEquals(Side.BLACK, Piece.sideOf(Piece.BLACK_KNIGHT));    // Black
        assertEquals(Side.WHITE, Piece.sideOf(WHITE_BISHOP));    // White
        assertEquals(Side.BLACK, Piece.sideOf(Piece.BLACK_ROOK));      // Black
        assertEquals(Side.WHITE, Piece.sideOf(Piece.WHITE_QUEEN));     // White
        assertEquals(Side.BLACK, Piece.sideOf(Piece.BLACK_KING));      // Black
    }

    @Test
    void testMakePiece() {
        // Test making a piece from a side and piece type
        assertEquals(WHITE_PAWN, Piece.makePiece(0, 0));      // White Pawn
        assertEquals(Piece.BLACK_KNIGHT, Piece.makePiece(1, 1));    // Black Knight
        assertEquals(WHITE_BISHOP, Piece.makePiece(0, 2));    // White Bishop
        assertEquals(Piece.BLACK_ROOK, Piece.makePiece(1, 3));      // Black Rook
        assertEquals(Piece.WHITE_QUEEN, Piece.makePiece(0, 4));     // White Queen
        assertEquals(Piece.BLACK_KING, Piece.makePiece(1, 5));      // Black King
    }

    @Test
    void testGetNotation() {
        // Test the correct notation for each piece
        assertEquals("P", Piece.getNotation(WHITE_PAWN));     // White Pawn
        assertEquals("N", Piece.getNotation(WHITE_KNIGHT));   // White Knight
        assertEquals("B", Piece.getNotation(WHITE_BISHOP));   // White Bishop
        assertEquals("R", Piece.getNotation(Piece.WHITE_ROOK));     // White Rook
        assertEquals("Q", Piece.getNotation(Piece.WHITE_QUEEN));    // White Queen
        assertEquals("K", Piece.getNotation(Piece.WHITE_KING));     // White King

        assertEquals("p", Piece.getNotation(Piece.BLACK_PAWN));     // Black Pawn
        assertEquals("n", Piece.getNotation(Piece.BLACK_KNIGHT));   // Black Knight
        assertEquals("b", Piece.getNotation(Piece.BLACK_BISHOP));   // Black Bishop
        assertEquals("r", Piece.getNotation(Piece.BLACK_ROOK));     // Black Rook
        assertEquals("q", Piece.getNotation(Piece.BLACK_QUEEN));    // Black Queen
        assertEquals("k", Piece.getNotation(Piece.BLACK_KING));     // Black King

        assertEquals(" ", Piece.getNotation(Piece.NONE));           // No piece
    }
}
