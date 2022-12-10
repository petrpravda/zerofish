package search;

import org.javafish.move.Move;

public class TTEntry {
    public final static byte EXACT = 0, LOWER_BOUND = 1, UPPER_BOUND = 2;
    public final static int SIZE = 10; // in bytes

    private final int score;
    private final byte depth, flag;
    private final short bestMove;
    final long key;

    public TTEntry(long key, int score, int depth, int flag, Move bestMove){
        this.key = key;
        this.score = score;
        this.depth = (byte)depth;
        this.flag = (byte)flag;
        this.bestMove = bestMove.bits();
    }

    public int score(){
        return score;
    }

    public int depth(){
        return depth;
    }

    public int flag(){
        return flag;
    }

    public Move move(){
        return new Move(bestMove);
    }

}
