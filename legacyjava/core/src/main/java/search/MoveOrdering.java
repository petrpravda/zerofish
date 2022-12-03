package search;

import org.javafish.board.BoardState;
import org.javafish.move.Move;
import org.javafish.move.MoveList;

import java.util.Collections;


public class MoveOrdering {
    private final int[][][] killerMoves = new int[2][1000][1];
    static final int HashMoveScore = 10000;
    static final int KillerMoveScore = 150;

    public void addKiller(BoardState state, Move move, int ply){
        int side = state.getSideToPlay();
        for (int i = killerMoves[side][ply].length - 2; i >= 0; i--)
            killerMoves[side][ply][i+1] = killerMoves[side][ply][i];
        killerMoves[side][ply][0] = move.bits();
    }

    public boolean isKiller(BoardState state, Move move, int ply){
        int moveInt = move.bits();
        int side = state.getSideToPlay();
        for (int i = 0; i < killerMoves[side][ply].length; i++){
            if (moveInt == killerMoves[side][ply][i])
                return true;
        }
        return false;
    }
}
