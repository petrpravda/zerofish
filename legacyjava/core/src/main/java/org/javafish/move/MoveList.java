package org.javafish.move;

import org.javafish.bitboard.Bitboard;

import java.util.ArrayList;
import java.util.Collections;

public class MoveList extends ArrayList<Move> {

    public MoveList(){}

//    public MoveList(MoveList halfMoves){
//        super.addAll(halfMoves);
//    }

    public void makeQ(int fromSq, long to){
        int toSq;
        while (to != 0){
            toSq = Bitboard.lsb(to);
            to = Bitboard.extractLsb(to);
            super.add(new Move(fromSq, toSq, Move.QUIET));
        }
    }

    public void makeC(int fromSq, long to){
        int toSq;
        while (to != 0){
            toSq = Bitboard.lsb(to); // TODO inline
            to = Bitboard.extractLsb(to);
            super.add(new Move(fromSq, toSq, Move.CAPTURE));
        }
    }

    public void makeDP(int fromSq, long to){
        int toSq;
        while (to != 0){
            toSq = Bitboard.lsb(to);
            to = Bitboard.extractLsb(to);
            super.add(new Move(fromSq, toSq, Move.DOUBLE_PUSH));
        }
    }

//    public void makePR(int fromSq, long to){
//        int toSq;
//        while (to != 0){
//            toSq = Bitboard.lsb(to);
//            to = Bitboard.extractLsb(to);
//            super.add(new Move(fromSq, toSq, Move.PR_KNIGHT));
//            super.add(new Move(fromSq, toSq, Move.PR_BISHOP));
//            super.add(new Move(fromSq, toSq, Move.PR_ROOK));
//            super.add(new Move(fromSq, toSq, Move.PR_QUEEN));
//        }
//    }

    public void makePC(int fromSq, long to){
        int toSq;
        while (to != 0){
            toSq = Bitboard.lsb(to);
            to = Bitboard.extractLsb(to);
            super.add(new Move(fromSq, toSq, Move.PC_KNIGHT));
            super.add(new Move(fromSq, toSq, Move.PC_BISHOP));
            super.add(new Move(fromSq, toSq, Move.PC_ROOK));
            super.add(new Move(fromSq, toSq, Move.PC_QUEEN));
        }
    }

    public void pickNextBestMove(int curIndex){
        int max = Integer.MIN_VALUE;
        int maxIndex = -1;
        for (int i = curIndex; i < this.size(); i++){
            if (this.get(i).score() > max){
                max = this.get(i).score();
                maxIndex = i;
            }
        }
        Collections.swap(this, curIndex, maxIndex);
    }
}
