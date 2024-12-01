package org.javafish.board;

import org.javafish.move.Move;
import org.javafish.move.MoveList;
import org.junit.jupiter.api.Test;
import search.Search;
import search.TranspositionTable;

import java.util.List;

import static org.javafish.bitboard.Bitboard.bitboardToString;
import static org.javafish.board.Fen.START_POS;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class BoardStateTest {
    @Test
    public void complexPositionTroubleWithScore() {
        // actually caused by crappy castling flags
        BoardPosition position = BoardPosition.fromFen("r2q1rk1/pbp2ppp/1pnp1n2/8/2PPp3/2P1P3/P1N2PPP/R1BQKB1R w kq - 0 10");
        new Search(new TranspositionTable()).itDeep(position, 9);
    }

//    @Test
//    void toParamsWhiteOnTheMove() {
//        BoardState state = BoardState.fromFen(START_POS);
//        BoardState.Params params = state.toParams();
//        assertEquals(4, params.wKingPos());
//        assertEquals(60, params.bKingPos());
//        byte[] pieces = params.pieces();
//        assertEquals("11111111", getBinaryString(pieces[Side.WHITE * 48 + PieceType.PAWN * 8 + 6]));
//        assertEquals("1000010", getBinaryString(pieces[Side.WHITE * 48 + PieceType.KNIGHT * 8 + 7]));
//        assertEquals("100100", getBinaryString(pieces[Side.WHITE * 48 + PieceType.BISHOP * 8 + 7]));
//        assertEquals("10000001", getBinaryString(pieces[Side.WHITE * 48 + PieceType.ROOK * 8 + 7]));
//        assertEquals("1000", getBinaryString(pieces[Side.WHITE * 48 + PieceType.QUEEN * 8 + 7]));
//        assertEquals("11111111", getBinaryString(pieces[Side.BLACK * 48 + PieceType.PAWN * 8 + 1 - 8]));
//        assertEquals("1000010", getBinaryString(pieces[Side.BLACK * 48 + PieceType.KNIGHT * 8 + 0 - 8]));
//        assertEquals("100100", getBinaryString(pieces[Side.BLACK * 48 + PieceType.BISHOP * 8 + 0 - 8]));
//        assertEquals("10000001", getBinaryString(pieces[Side.BLACK * 48 + PieceType.ROOK * 8 + 0 - 8]));
//        assertEquals("1000", getBinaryString(pieces[Side.BLACK * 48 + PieceType.QUEEN * 8 + 0 - 8]));
//    }


//    @Test
//    void toParamsBlackOnTheMove() {
//        BoardState state = BoardState.fromFen(START_POS)
//                .doMove(Move.NULL_MOVE);
//        byte[] params = state.toParams();
//        int index = 0;
//        assertEquals("11111111", getBinaryString(params[Side.WHITE * 48 + PieceType.PAWN * 8 + 6]));
//        assertEquals("1000010", getBinaryString(params[Side.WHITE * 48 + PieceType.KNIGHT * 8 + 7]));
//        assertEquals("100100", getBinaryString(params[Side.WHITE * 48 + PieceType.BISHOP * 8 + 7]));
//        assertEquals("10000001", getBinaryString(params[Side.WHITE * 48 + PieceType.ROOK * 8 + 7]));
//        assertEquals("1000", getBinaryString(params[Side.WHITE * 48 + PieceType.QUEEN * 8 + 7]));
//        assertEquals("10000", getBinaryString(params[Side.WHITE * 48 + PieceType.KING * 8 + 7]));
//        assertEquals("11111111", getBinaryString(params[Side.BLACK * 48 + PieceType.PAWN * 8 + 1]));
//        assertEquals("1000010", getBinaryString(params[Side.BLACK * 48 + PieceType.KNIGHT * 8 + 0]));
//        assertEquals("100100", getBinaryString(params[Side.BLACK * 48 + PieceType.BISHOP * 8 + 0]));
//        assertEquals("10000001", getBinaryString(params[Side.BLACK * 48 + PieceType.ROOK * 8 + 0]));
//        assertEquals("1000", getBinaryString(params[Side.BLACK * 48 + PieceType.QUEEN * 8 + 0]));
//        assertEquals("10000", getBinaryString(params[Side.BLACK * 48 + PieceType.KING * 8 + 0]));
//    }

    private static String getBinaryString(byte param) {
        return Integer.toBinaryString(Byte.toUnsignedInt(param));
    }

    @Test
    void attackedPieces() {
        BoardState state = BoardState.fromFen("5k2/p6p/1p1r4/1PpP1P2/2P5/P4K2/8/7R b - - 1 40");
        long attackedPieces = state.attackedPieces(Side.BLACK);
        assertEquals("""
                . . . . . . . .\s
                . . . . . . . X\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                """, bitboardToString(attackedPieces));
    }

    @Test
    void attackedPiecesWithPinning() {
        BoardState state = BoardState.fromFen("6q1/1p2bpk1/1r4p1/3pPB2/1n1P2Q1/6P1/3N2K1/7R b - - 4 39");
        long attackedPieces = state.attackedPieces(Side.WHITE);
        assertEquals("""
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                """, bitboardToString(attackedPieces));
    }

    @Test
    void attackedPiecesUndefended() {
        BoardState state = BoardState.fromFen("5k2/p6p/1p1r4/1PpP1P2/2P5/P4K2/8/7R b - - 1 40");
        long attackedPiecesUndefended = state.attackedPiecesUndefended(Side.BLACK);
        assertEquals("""
                . . . . . . . .\s
                . . . . . . . X\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                """, bitboardToString(attackedPiecesUndefended));

        state = BoardState.fromFen("8/p5kp/1p1r4/1PpP1P2/2P5/P4K2/8/7R w - - 2 41");
        attackedPiecesUndefended = state.attackedPiecesUndefended(Side.BLACK);
        assertEquals(0L, attackedPiecesUndefended);
    }

    @Test
    void attackedPiecesUndefendedBehindSlidingAttacker() {
        BoardState state = BoardState.fromFen("r2qk2r/pp1nbpQp/2p1p1b1/8/4P1P1/5N1P/PPP2PB1/R1B2RK1 b kq - 0 13");
        long attackedPiecesUndefended = state.attackedPiecesUndefended(Side.BLACK);
        assertEquals("""
                . . . . . . . X\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                """, bitboardToString(attackedPiecesUndefended));

        state = BoardState.fromFen("r2qk2r/pp1n1pQp/2p1pbb1/8/4P1P1/5N1P/PPP2PB1/R1B2RK1 w kq - 1 14");
        attackedPiecesUndefended = state.attackedPiecesUndefended(Side.BLACK);
        assertEquals("""
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                """, bitboardToString(attackedPiecesUndefended));
    }

    @Test
    void mgValueTest() {
        BoardState state = BoardState.fromFen("8/8/8/8/8/8/8/8 w KQkq - 0 1");
        assertEquals(0, state.mg());
        state.setPieceAt(Piece.WHITE_ROOK, Square.getSquareFromName("d5"));
        assertEquals(482, state.mg());
        state.setPieceAt(Piece.BLACK_ROOK, Square.getSquareFromName("d2"));
        assertEquals(-20, state.mg());
    }

    @Test
    void fromFailingSts() {
        BoardState state = BoardState.fromFen("2r5/p3k1p1/1p5p/4Pp2/1PPnK3/PB1R2P1/7P/8 w - f6 0 4");
        MoveList moves = state.generateLegalMoves();

        // Expected moves in UCI format
        List<String> expectedMoves = List.of("e4e3", "e4f4", "e4d5", "e4d4", "e5f6");

        // Convert MoveList to a list of UCI strings for comparison
        List<String> actualMoves = moves.stream().map(Move::toString).toList();

        // Assert that the expected moves match the actual moves
        assertEquals(expectedMoves, actualMoves, "The list of legal moves is not as expected.");
    }


    @Test
    void seeScore() {
        BoardState state = BoardState.fromFen("6k1/5pp1/4p2p/8/5P2/4RQP1/rq1rR2P/5K2 b - - 3 33");
        var result = state.seeScore(12, Side.BLACK);
        assertEquals(0, result.score());
    }

//    @Test
//    void phasesOnlyPositive() {
//        String[] moves = "d2d4 c7c5 d4c5 g7g5 c1g5 h7h6 c5c6 h6g5 c6b7 h8h2 b7a8q".split(" ");
//        BoardState state = BoardState.fromFen(START_POS);
//        for (String uciMove : moves) {
//            Move move = Move.fromUciString(uciMove, state);
//            state = state.doMove(move);
//            System.out.println(state.phase);
//        }
//    }

    @Test
    void testClone() {
        BoardState original = BoardState.fromFen(START_POS);
        BoardState cloned = original.clone();

        assertEquals(original.hash(), cloned.hash());
        assertArrayEquals(original.items, cloned.items);
        assertEquals(original.getSideToPlay(), cloned.getSideToPlay());
    }

    @Test
    void testMovePiece() {
        BoardState stateBeforeMove = BoardState.fromFen(START_POS);

        int fromSq = Square.getSquareFromName("d2");
        int toSq = Square.getSquareFromName("d4");

        BoardState stateAfterMove = stateBeforeMove.doMove(new Move(fromSq, toSq));

        assertEquals(Piece.WHITE_PAWN, stateAfterMove.pieceAt(toSq));
        assertEquals(Piece.NONE, stateAfterMove.pieceAt(fromSq));
    }

    @Test
    void testSetPieceAt() {
        BoardState state = BoardState.fromFen(START_POS);

        int square = 20; // d2
        int piece = Piece.WHITE_KNIGHT;
        state.setPieceAt(piece, square);

        assertEquals(piece, state.pieceAt(square));
    }

    @Test
    void testRemovePiece() {
        BoardState state = BoardState.fromFen(START_POS);

        int square = 12; // e2
        state.removePiece(square);

        assertEquals(Piece.NONE, state.pieceAt(square));
    }

    @Test
    void testHash() {
        BoardState state = BoardState.fromFen(START_POS);
        long initialHash = state.hash();

        BoardState stateAfterMove = state.doMove(Move.fromUciString("e2e4", state));
        long newHash = stateAfterMove.hash();

        assertNotEquals(initialHash, newHash);
    }

    @Test
    void testDiagonalSliders() {
        BoardState state = BoardState.fromFen(START_POS);

        long whiteDiagonalSliders = state.diagonalSliders(Side.WHITE);
        long blackDiagonalSliders = state.diagonalSliders(Side.BLACK);

        assertEquals("""
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . X X . X . .\s
                """, bitboardToString(whiteDiagonalSliders));
        assertEquals("""
                . . X X . X . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                """, bitboardToString(blackDiagonalSliders));
    }

    @Test
    void testOrthogonalSliders() {
        BoardState state = BoardState.fromFen(START_POS);

        long whiteOrthogonalSliders = state.orthogonalSliders(Side.WHITE);
        long blackOrthogonalSliders = state.orthogonalSliders(Side.BLACK);

        assertEquals("""
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                X . . X . . . X\s
                """, bitboardToString(whiteOrthogonalSliders));
        assertEquals("""
                X . . X . . . X\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                . . . . . . . .\s
                """, bitboardToString(blackOrthogonalSliders));
    }

    @Test
    void testAllPieces() {
        BoardState state = BoardState.fromFen(START_POS);

        long whitePieces = state.allPieces(Side.WHITE);
        long blackPieces = state.allPieces(Side.BLACK);

        // Assert that all white pieces are on ranks 1 and 2
        assertEquals(0xFFFFL, whitePieces);

        // Assert that all black pieces are on ranks 7 and 8
        assertEquals(0xFFFF000000000000L, blackPieces);
    }

    @Test
    void testIsKingAttacked() {
        BoardState state = BoardState.fromFen(START_POS);
        assertFalse(state.isKingAttacked());

        state = BoardState.fromFen("rnbqk1nr/pppp1ppp/4p3/8/1bPP4/8/PP2PPPP/RNBQKBNR w KQkq - 1 3");
        assertTrue(state.isKingAttacked()); // White king is attacked by black bishop
    }

//    @Test
//    void testIsRepetitionOrFifty() {
//        BoardState state = BoardState.fromFen(START_POS);
//        assertFalse(state.isRepetitionOrFifty());
//
//        state.halfMoveClock = 101;
//        assertTrue(state.isRepetitionOrFifty());
//    }

    @Test
    void testDoMove() {
        BoardState state = BoardState.fromFen(START_POS);
        BoardState nextState = state.doMove("e2e4");

        assertEquals(Piece.NONE, nextState.pieceAt(12)); // e2 is now empty
        assertEquals(Piece.WHITE_PAWN, nextState.pieceAt(28)); // e4 now has a White Pawn
        assertEquals(Side.BLACK, nextState.getSideToPlay()); // It's Black's turn now
    }

    @Test
    void testToFen() {
        BoardState state = BoardState.fromFen(START_POS);
        String fen = state.toFen();
        assertEquals(START_POS, fen);
    }

//    @Test
//    void testGenMovesTester() {
//        BoardState state = BoardState.fromFen("r1bqkbnr/pppppppp/n7/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2");
//        MoveList moves = state.generateLegalMoves();
//        //String fen = state.toFen();
//        //assertEquals(START_POS, fen);
//    }

    @Test
    void checkMoveC1G5IsNotMissing() {
        BoardState state = BoardState.fromFen("r1bqkbnr/pppppppp/n7/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 2");
        List<String> moves = state.generateLegalMoves().stream()
                .map(Object::toString)
                .toList();

        assertTrue(moves.contains("c1g5"), "Move c1g5 should be in the list of legal moves");
    }

    @Test
    void checkMoveF2E1QIsNotMissing() {
        BoardState state = BoardState.fromFen("8/7p/p7/P3P3/4b3/R2rN1kn/1P3p2/4BK2 b - - 3 50");
        List<String> moves = state.generateLegalMoves().stream()
                .map(Object::toString)
                .toList();
        assertEquals(33, moves.size());
        assertTrue(moves.contains("f2e1q"), "Move list should contain the promotion move f2e1q");
    }

    @Test
    void checkMoveG6H5IsNotMissing() {
        BoardState state = BoardState.fromFen("5r1k/1p3p1p/3p1PP1/p1nBp1P1/4PbQ1/1Pq5/2P3R1/5R1K b - - 0 28");
        List<String> moves = state.generateLegalMoves().stream()
                .map(Object::toString)
                .toList();
        assertEquals(41, moves.size());
        assertTrue(moves.contains("g6h5"), "Move list should contain the move g6h5");
    }

    @Test
    void testInterpolatedScore() {
        BoardState state = BoardState.fromFen(START_POS);
        MoveList moves = state.generateLegalMoves();
        int score = state.interpolatedScore();
        assertEquals(0, score); // Initial position, interpolated score should be neutral
    }

    @Test
    void testIsCapture() {
        BoardState state = BoardState.fromFen("rnb1k2r/ppp1nppp/4p3/3pP3/6QN/2b5/PPP2PPP/R1B1KB1R w KQkq - 0 10");

        assertTrue(state.isCapture("b2c3"));
        assertFalse(state.isCapture("e1d1"));
    }

    @Test
    void testHasNonPawnMaterial() {
        // 1. Normal position with non-pawn material for both sides
        BoardState state1 = BoardState.fromFen("rnb1k2r/ppp1nppp/4p3/3pP3/6QN/2b5/PPP2PPP/R1B1KB1R w KQkq - 0 10");
        assertTrue(state1.hasNonPawnMaterial(Side.WHITE)); // White has non-pawn material (Knight, Queen, Rook)
        assertTrue(state1.hasNonPawnMaterial(Side.BLACK)); // Black has non-pawn material (Knight, Bishop, Rook)

        // 2. Only pawns on the board for both sides
        BoardState state2 = BoardState.fromFen("8/8/8/4p3/4P3/8/8/8 w - - 0 1");
        assertFalse(state2.hasNonPawnMaterial(Side.WHITE)); // White has no non-pawn material
        assertFalse(state2.hasNonPawnMaterial(Side.BLACK)); // Black has no non-pawn material

        // 3. White has only pawns, Black has knights
        BoardState state3 = BoardState.fromFen("8/8/8/8/8/8/PPPPPPPP/NNNNNNNN w - - 0 1");
        assertTrue(state3.hasNonPawnMaterial(Side.WHITE)); // White has non-pawn material (Knights)
        assertFalse(state3.hasNonPawnMaterial(Side.BLACK)); // Black only has pawns

        // 4. Empty board, no material
        BoardState state4 = BoardState.fromFen("8/8/8/8/8/8/8/8 w - - 0 1");
        assertFalse(state4.hasNonPawnMaterial(Side.WHITE)); // White has no non-pawn material
        assertFalse(state4.hasNonPawnMaterial(Side.BLACK)); // Black has no non-pawn material

        // 5. White has only a queen, Black has only pawns
        BoardState state5 = BoardState.fromFen("8/8/8/8/8/8/8/Q7 w - - 0 1");
        assertTrue(state5.hasNonPawnMaterial(Side.WHITE)); // White has a queen (non-pawn material)
        assertFalse(state5.hasNonPawnMaterial(Side.BLACK)); // Black has no non-pawn material

        // 6. Black has a bishop and a rook, White has only pawns
        BoardState state6 = BoardState.fromFen("8/8/8/8/8/8/pppppppp/rb6 w - - 0 1");
        assertFalse(state6.hasNonPawnMaterial(Side.WHITE)); // White has no non-pawn material
        assertTrue(state6.hasNonPawnMaterial(Side.BLACK)); // Black has non-pawn material (Bishop, Rook)

        // 7. Both sides have only kings (no other material)
        BoardState state7 = BoardState.fromFen("8/8/8/8/8/8/8/K7 w - - 0 1");
        assertFalse(state7.hasNonPawnMaterial(Side.WHITE)); // White has no non-pawn material
        assertFalse(state7.hasNonPawnMaterial(Side.BLACK)); // Black has no non-pawn material

        // 8. White has a rook and pawns, Black has only a queen
        BoardState state8 = BoardState.fromFen("8/8/8/8/8/8/8/R7 w - - 0 1");
        assertTrue(state8.hasNonPawnMaterial(Side.WHITE)); // White has a rook (non-pawn material)
        assertFalse(state8.hasNonPawnMaterial(Side.BLACK)); // Black has no non-pawn material

        // 9. White has bishops, Black has only pawns
        BoardState state9 = BoardState.fromFen("8/8/8/8/8/8/pppppppp/BBBBBBBB w - - 0 1");
        assertTrue(state9.hasNonPawnMaterial(Side.WHITE)); // White has bishops
        assertFalse(state9.hasNonPawnMaterial(Side.BLACK)); // Black only has pawns

        // 10. White and Black both have queens only
        BoardState state10 = BoardState.fromFen("8/8/8/8/8/8/qqqqqqqq/QQQQQQQQ w - - 0 1");
        assertTrue(state10.hasNonPawnMaterial(Side.WHITE)); // White has queens
        assertTrue(state10.hasNonPawnMaterial(Side.BLACK)); // Black has queens

        // 11. White has a knight, Black has nothing
        BoardState state11 = BoardState.fromFen("8/8/8/8/8/8/8/N7 w - - 0 1");
        assertTrue(state11.hasNonPawnMaterial(Side.WHITE)); // White has a knight
        assertFalse(state11.hasNonPawnMaterial(Side.BLACK)); // Black has nothing

        // 12. Black has a knight, White has nothing
        BoardState state12 = BoardState.fromFen("8/8/8/8/8/8/8/8/8/n7 w - - 0 1");
        assertFalse(state12.hasNonPawnMaterial(Side.WHITE)); // White has nothing
        assertTrue(state12.hasNonPawnMaterial(Side.BLACK)); // Black has a knight

        // 13. White has a bishop and pawns, Black has a rook
        BoardState state13 = BoardState.fromFen("8/8/8/8/8/8/PPP5/B7 w - - 0 1");
        assertTrue(state13.hasNonPawnMaterial(Side.WHITE)); // White has a bishop
        assertFalse(state13.hasNonPawnMaterial(Side.BLACK)); // Black has no non-pawn material

        // 14. White and Black both have knights and bishops
        BoardState state14 = BoardState.fromFen("8/8/8/8/8/8/nnnnnnnn/BBBBBBBB w - - 0 1");
        assertTrue(state14.hasNonPawnMaterial(Side.WHITE)); // White has bishops
        assertTrue(state14.hasNonPawnMaterial(Side.BLACK)); // Black has knights

        // 15. White has no material, Black has pawns
        BoardState state15 = BoardState.fromFen("8/8/8/8/8/8/8/p7 w - - 0 1");
        assertFalse(state15.hasNonPawnMaterial(Side.WHITE)); // White has no non-pawn material
        assertFalse(state15.hasNonPawnMaterial(Side.BLACK)); // Black has no non-pawn material

        // 16. Full material for both sides
        BoardState state16 = BoardState.fromFen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        assertTrue(state16.hasNonPawnMaterial(Side.WHITE)); // White has knights, bishops, rooks, and queens
        assertTrue(state16.hasNonPawnMaterial(Side.BLACK)); // Black has knights, bishops, rooks, and queens

        // 17. Only White King left
        BoardState state17 = BoardState.fromFen("8/k7/8/8/8/8/8/7K w - - 0 1");
        assertFalse(state17.hasNonPawnMaterial(Side.WHITE)); // Only White King left
        assertFalse(state17.hasNonPawnMaterial(Side.BLACK)); // Only Black King left

        // 18. White has two Rooks, Black has only a King
        BoardState state18 = BoardState.fromFen("8/k7/8/8/8/8/8/R3K2R w KQ - 0 1");
        assertTrue(state18.hasNonPawnMaterial(Side.WHITE)); // White has two Rooks
        assertFalse(state18.hasNonPawnMaterial(Side.BLACK)); // Only Black King left
    }
}
