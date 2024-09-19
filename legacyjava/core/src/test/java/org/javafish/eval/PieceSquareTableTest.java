package org.javafish.eval;

import org.javafish.board.Piece;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class PieceSquareTableTest {
    @Test
    public void testMGSForWhitePawn() {
        assertEquals(149, PieceSquareTable.MGS[Piece.WHITE_PAWN][50]);
    }

    @Test
    public void testMGSForBlackKnight() {
        assertEquals(-401, PieceSquareTable.MGS[Piece.BLACK_KNIGHT][20]);
    }

    @Test
    public void testEGSForWhiteBishop() {
        assertEquals(314, PieceSquareTable.EGS[Piece.WHITE_BISHOP][30]);
    }

    @Test
    public void testEGSForBlackQueen() {
        assertEquals(-1000, PieceSquareTable.EGS[Piece.BLACK_QUEEN][40]);
    }

    @Test
    public void testMGSForWhiteKing() {
        assertEquals(1487, PieceSquareTable.MGS[Piece.WHITE_KING][22]);
    }

    @Test
    public void testEGSForBlackRook() {
        assertEquals(-521, PieceSquareTable.EGS[Piece.BLACK_ROOK][33]);
    }

    @Test
    public void testMGSForWhiteQueen() {
        assertEquals(1041, PieceSquareTable.MGS[Piece.WHITE_QUEEN][44]);
    }

    @Test
    public void testEGSForBlackPawn() {
        assertEquals(-70, PieceSquareTable.EGS[Piece.BLACK_PAWN][55]);
    }

    @Test
    public void testMGSForWhiteRook() {
        assertEquals(472, PieceSquareTable.MGS[Piece.WHITE_ROOK][28]);
    }

    @Test
    public void testEGSForBlackBishop() {
        assertEquals(-314, PieceSquareTable.EGS[Piece.BLACK_BISHOP][38]);
    }
}
