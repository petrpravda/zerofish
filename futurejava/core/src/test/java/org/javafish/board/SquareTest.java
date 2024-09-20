package org.javafish.board;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

public class SquareTest {

    @Test
    void testGetName() {
        // Test a few known squares
        assertEquals("a1", Square.getName(Square.A1));
        assertEquals("h1", Square.getName(Square.H1));
        assertEquals("d4", Square.getName(27)); // d4 corresponds to index 27
        assertEquals("e8", Square.getName(Square.E8));
    }

    @Test
    void testGetSquareFromName() {
        // Test getting squares from UCI notation
        assertEquals(Square.A1, Square.getSquareFromName("a1"));
        assertEquals(Square.H1, Square.getSquareFromName("h1"));
        assertEquals(27, Square.getSquareFromName("d4")); // d4 corresponds to index 27
        assertEquals(Square.E8, Square.getSquareFromName("e8"));
    }

    @Test
    void testGetFile() {
        // Test file extraction
        assertEquals('a', Square.getFile(Square.A1));
        assertEquals('h', Square.getFile(Square.H1));
        assertEquals('d', Square.getFile(27)); // d4 corresponds to index 27
        assertEquals('e', Square.getFile(Square.E8));
    }

    @Test
    void testGetRank() {
        // Test rank extraction
        assertEquals('1', Square.getRank(Square.A1));
        assertEquals('8', Square.getRank(Square.A8));
        assertEquals('4', Square.getRank(27)); // d4 corresponds to index 27
        assertEquals('8', Square.getRank(Square.E8));
    }

    @Test
    void testGetRankIndex() {
        // Test rank index extraction
        assertEquals(0, Square.getRankIndex(Square.A1));  // Rank 1 (index 0)
        assertEquals(7, Square.getRankIndex(Square.A8));  // Rank 8 (index 7)
        assertEquals(3, Square.getRankIndex(27));         // d4 corresponds to rank 4 (index 3)
    }

    @Test
    void testGetFileIndex() {
        // Test file index extraction
        assertEquals(0, Square.getFileIndex(Square.A1));  // File 'a' (index 0)
        assertEquals(7, Square.getFileIndex(Square.H1));  // File 'h' (index 7)
        assertEquals(3, Square.getFileIndex(27));         // d4 corresponds to file 'd' (index 3)
    }

    @Test
    void testGetDiagonalIndex() {
        // Test diagonal index calculation
        assertEquals(7, Square.getDiagonalIndex(Square.A1));  // Main diagonal
        assertEquals(7, Square.getDiagonalIndex(Square.H8));  // Main diagonal
        assertEquals(6, Square.getDiagonalIndex(Square.E4));
    }

    @Test
    void testGetAntiDiagonalIndex() {
        // Test anti-diagonal index calculation
        assertEquals(0, Square.getAntiDiagonalIndex(Square.A1));  // Main anti-diagonal
        assertEquals(14, Square.getAntiDiagonalIndex(Square.H8));  // Main anti-diagonal
        assertEquals(7, Square.getAntiDiagonalIndex(Square.E4));   // Anti-diagonal index for e4
    }

    @Test
    void testDirection() {
        // Test direction adjustment based on side
        assertEquals(Square.FORWARD, Square.direction(Square.FORWARD, Side.WHITE));
        assertEquals(-Square.FORWARD, Square.direction(Square.FORWARD, Side.BLACK));

        assertEquals(Square.RIGHT, Square.direction(Square.RIGHT, Side.WHITE));
        assertEquals(-Square.RIGHT, Square.direction(Square.RIGHT, Side.BLACK));
    }
}

