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
        assertEquals(original.ply, cloned.ply);
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
    void testHasNonPawnMaterial() {
        BoardState state = BoardState.fromFen("8/k7/8/8/8/8/8/7K w - - 0 1");
        assertFalse(state.hasNonPawnMaterial(Side.WHITE)); // Only White King left

        state = BoardState.fromFen("8/k7/8/8/8/8/8/R3K2R w KQ - 0 1");
        assertTrue(state.hasNonPawnMaterial(Side.WHITE)); // White has two Rooks
    }

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

    @Test
    void testInterpolatedScore() {
        BoardState state = BoardState.fromFen(START_POS);
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
    void moveGenerationForPromotion() {
        BoardState state = BoardState.fromFen("8/7p/p7/P3P3/4b3/R2rN1kn/1P3p2/4BK2 b - - 3 50");
        MoveList legalMoves = state.generateLegalMoves();
        List<String> movesAsStrings = legalMoves.stream().map(Move::toString).toList();
        assertEquals(33, movesAsStrings.size());
    }
}
