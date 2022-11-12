package search;

import org.javafish.board.BoardState;
import org.javafish.board.Side;
import org.javafish.move.Move;
import org.javafish.move.MoveList;

import java.util.Collections;

import static org.javafish.eval.PieceSquareTable.MGS;


public class MoveOrder {

    private static final int[][][] killerMoves = new int[2][1000][1];
    // private static final int[][] historyMoves = new int[64][64];
    // private static final int[][] MvvLvaScores = new int[6][6];

    private static final int HashMoveScore = 10000;
//    private static final int PromotionScore = 5000;
//    private static final int CaptureScore = 200;
    private static final int KillerMoveScore = 150;

//    static {
//        final int[] VictimScore = {100, 280, 320, 500, 1000, 1000};
//        for (int attacker = PieceType.PAWN; attacker <= PieceType.KING; attacker++) {
//            for (int victim = PieceType.PAWN; victim <= PieceType.KING; victim++) {
//                MvvLvaScores[victim][attacker] = VictimScore[victim] + 6 - (VictimScore[attacker] / 100);
//            }
//        }
//    }

//    public static int seeCapture(BoardState oldBoardState, Move move){
//        int capturedPieceType;
//        if (move.flags() == Move.EN_PASSANT)
//            capturedPieceType = PieceType.PAWN;
//        else
//            capturedPieceType = oldBoardState.pieceTypeAt(move.to());
//        BoardState state = oldBoardState.doMove(move);
//        int value = 0;
//        if (move.isPromotion())
//            switch(move.flags()){
//                case Move.PC_QUEEN:
//                    value = EConstants.PIECE_TYPE_VALUES[PieceType.QUEEN] + EConstants.PIECE_TYPE_VALUES[capturedPieceType] - see(state, move.to());
//                    break;
//                case Move.PC_ROOK:
//                    value = EConstants.PIECE_TYPE_VALUES[PieceType.ROOK] + EConstants.PIECE_TYPE_VALUES[capturedPieceType] - see(state, move.to());
//                    break;
//                case Move.PC_BISHOP:
//                    value = EConstants.PIECE_TYPE_VALUES[PieceType.BISHOP] + EConstants.PIECE_TYPE_VALUES[capturedPieceType] - see(state, move.to());
//                    break;
//                case Move.PC_KNIGHT:
//                    value = EConstants.PIECE_TYPE_VALUES[PieceType.KNIGHT] + EConstants.PIECE_TYPE_VALUES[capturedPieceType] - see(state, move.to());
//                    break;
//                case Move.CAPTURE:
//                case Move.EN_PASSANT:
//                    value = EConstants.PIECE_TYPE_VALUES[capturedPieceType] - see(state, move.to());
//            }
//
//        return Score.eval(value);
//    }

//    public static int see(BoardState oldBoardState, int toSq){
//        int value = 0;
//        int fromSq = oldBoardState.smallestAttacker(toSq, oldBoardState.getSideToPlay());
//        if (fromSq != Square.NO_SQUARE){
//            int capturedPieceValue = Score.eval(EConstants.PIECE_TYPE_VALUES[oldBoardState.pieceTypeAt(toSq)]);
//            BoardState state = oldBoardState.doMove(new Move(fromSq, toSq, Move.CAPTURE));
//            value = Math.max(0, capturedPieceValue - see(state, toSq));
//        }
//        return value;
//    }

    public static void addKiller(BoardState state, Move move, int ply){
        int side = state.getSideToPlay();
        for (int i = killerMoves[side][ply].length - 2; i >= 0; i--)
            killerMoves[side][ply][i+1] = killerMoves[side][ply][i];
        killerMoves[side][ply][0] = move.bits();
    }

    public static boolean isKiller(BoardState state, Move move, int ply){
        int moveInt = move.bits();
        int side = state.getSideToPlay();
        for (int i = 0; i < killerMoves[side][ply].length; i++){
            if (moveInt == killerMoves[side][ply][i])
                return true;
        }
        return false;
    }

//    public static void clearKillers(){
//        for (int color = Side.WHITE; color <= Side.BLACK; color++){
//            for (int ply = 0; ply < killerMoves[0].length; ply++){
//                for (int killer_i = 0; killer_i < killerMoves[0][0].length; killer_i++){
//                    killerMoves[color][ply][killer_i] = 0;
//                }
//            }
//        }
//    }

//    public static void addHistory(Move move, int depth){
//        int from = move.from();
//        int to = move.to();
//        historyMoves[from][to] += depth*depth;
//
//        if (historyMoves[from][to] > Integer.MAX_VALUE / 2) {
//            for (int sq1 = Square.A1; sq1 <= Square.H8; sq1++){
//                for (int sq2 = Square.A1; sq2 <= Square.H8; sq2++){
//                    historyMoves[sq1][sq2] /= 2;
//                }
//            }
//        }
//
//    }
//
//    public static int getHistoryValue(Move move){
//        return historyMoves[move.from()][move.to()];
//    }
//
//    public static void clearHistory(){
//        for (int sq1 = Square.A1; sq1 <= Square.H8; sq1++){
//            for (int sq2 = Square.A1; sq2 <= Square.H8; sq2++){
//                historyMoves[sq1][sq2] = 0;
//            }
//        }
//    }
//
//    public static void ageHistory(){
//        for (int sq1 = Square.A1; sq1 <= Square.H8; sq1++){
//            for (int sq2 = Square.A1; sq2 <= Square.H8; sq2++){
//                historyMoves[sq1][sq2] /= 8;
//            }
//        }
//    }

//    public static int getMvvLvaScore(BoardState state, Move move){
//        return MvvLvaScores[state.pieceTypeAt(move.to())][state.pieceTypeAt(move.from())];
//    }

    public static void scoreMoves(final BoardState state, final MoveList moves, int ply) {

        if (moves.size() == 0)
            return;

        Move hashMove = null;
        TTEntry ttEntry = TranspTable.probe(state.hash());
        if (ttEntry != null) {
            hashMove = ttEntry.move();
        }

        for (Move move : moves) {
            if (move.equals(hashMove)) {
                move.addToScore(HashMoveScore);
            }
            if (isKiller(state, move, ply)) {
                move.addToScore(KillerMoveScore);
            }
            //int pieceType = state.pieceTypeAt(move.from());
            int piece = state.items[move.from()];

            switch (move.flags()) {
                case Move.PC_BISHOP:
                case Move.PC_KNIGHT:
                case Move.PC_ROOK:
                case Move.PC_QUEEN:
//                    move.addToScore(CaptureScore);
//                    move.addToScore(getMvvLvaScore(state, move));
                    int score = MGS[move.getPieceTypeForSide(state.getSideToPlay())][move.to()] - MGS[piece][move.from()]
                            - MGS[state.items[move.to()]][move.to()];
                    score *= state.getSideToPlay() == Side.WHITE ? 1 : -1;
                    move.addToScore(score);
                    break;

                case Move.PR_BISHOP:
                case Move.PR_KNIGHT:
                case Move.PR_ROOK:
                case Move.PR_QUEEN:
                    score = MGS[move.getPieceTypeForSide(state.getSideToPlay())][move.to()] - MGS[piece][move.from()];
                    score *= state.getSideToPlay() == Side.WHITE ? 1 : -1;
                    move.addToScore(score);
                    break;
                case Move.CAPTURE:
                    score = MGS[piece][move.to()] - MGS[piece][move.from()] - MGS[state.items[move.to()]][move.to()];
                    score *= state.getSideToPlay() == Side.WHITE ? 1 : -1;
                    //move.addToScore(MGS[pieceType][move.to()]);
                    move.addToScore(score);

                    //move.addToScore(CaptureScore);
                    //move.addToScore(getMvvLvaScore(state, move));
                    break;
                case Move.QUIET:
                case Move.EN_PASSANT:
                case Move.DOUBLE_PUSH:
                case Move.OO:
                case Move.OOO:
                    score = MGS[piece][move.to()] - MGS[piece][move.from()];
                    score *= state.getSideToPlay() == Side.WHITE ? 1 : -1;
                    //move.addToScore(MGS[pieceType][move.to()]);
                    move.addToScore(score);
                    break;
            }
        }
    }

    public static void sortNextBestMove(MoveList moves, int curIndex){
        int max = Integer.MIN_VALUE;
        int maxIndex = -1;
        for (int i = curIndex; i < moves.size(); i++){
            if (moves.get(i).score() > max){
                max = moves.get(i).score();
                maxIndex = i;
            }
        }
        Collections.swap(moves, curIndex, maxIndex);
    }

}
