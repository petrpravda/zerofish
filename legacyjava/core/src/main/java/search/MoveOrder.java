package search;

import org.javafish.board.BoardState;
import org.javafish.board.Side;
import org.javafish.move.Move;
import org.javafish.move.MoveList;

import java.util.Collections;

import static org.javafish.eval.PieceSquareTable.MGS;


public class MoveOrder {

    private static final int[][][] killerMoves = new int[2][1000][1];
    private static final int HashMoveScore = 10000;
    private static final int KillerMoveScore = 150;

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
            int piece = state.items[move.from()];

            switch (move.flags()) {
                case Move.PC_BISHOP:
                case Move.PC_KNIGHT:
                case Move.PC_ROOK:
                case Move.PC_QUEEN:
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
                    move.addToScore(score);
                    break;
                case Move.QUIET:
                case Move.EN_PASSANT:
                case Move.DOUBLE_PUSH:
                case Move.OO:
                case Move.OOO:
                    score = MGS[piece][move.to()] - MGS[piece][move.from()];
                    score *= state.getSideToPlay() == Side.WHITE ? 1 : -1;
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
