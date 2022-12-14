package org.javafish.board;

import org.javafish.bitboard.Bitboard;
import org.javafish.move.MoveList;
import org.junit.jupiter.api.Test;
import search.Search;
import search.TranspositionTable;

import static org.javafish.board.Fen.START_POS;
import static org.junit.jupiter.api.Assertions.assertEquals;

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
                """, Bitboard.bitboardToString(attackedPieces));
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
                """, Bitboard.bitboardToString(attackedPieces));
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
                """, Bitboard.bitboardToString(attackedPiecesUndefended));

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
                """, Bitboard.bitboardToString(attackedPiecesUndefended));

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
                """, Bitboard.bitboardToString(attackedPiecesUndefended));
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
        System.out.println(moves);
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
}
