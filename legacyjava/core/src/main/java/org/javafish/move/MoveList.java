package org.javafish.move;

import org.javafish.bitboard.Bitboard;
import org.javafish.board.BoardState;
import org.javafish.board.Side;
import search.MoveOrdering;
import search.TTEntry;
import search.TranspTable;

import java.util.ArrayList;
import java.util.Collections;
import java.util.Iterator;
import java.util.concurrent.atomic.AtomicInteger;

import static org.javafish.eval.PieceSquareTable.MGS;

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

    private void scoreMoves(final BoardState state, TranspTable transpositionTable, int ply, MoveOrdering moveOrdering) {
        if (this.size() == 0)
            return;

        Move hashMove = null;
        TTEntry ttEntry = transpositionTable.probe(state.hash());
        if (ttEntry != null) {
            hashMove = ttEntry.move();
        }

        for (Move move : this) {
            if (move.equals(hashMove)) {
                move.addToScore(MoveOrdering.HashMoveScore);
            }
//            if (moveOrdering.isKiller(state, move, ply)) {
//                move.addToScore(MoveOrdering.KillerMoveScore);
//            }
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

    public Iterable<IndexedMove> overSorted(final BoardState state, TranspTable transpositionTable, int ply, MoveOrdering moveOrdering) {
        scoreMoves(state, transpositionTable, ply, moveOrdering);
        // //        moves.scoreMoves(state, transpositionTable, 0, moveOrdering);
        ////        for (int i = 0; i < moves.size(); i++){
        ////            moves.pickNextBestMove(i);
        ////            Move move = moves.get(i);
        AtomicInteger index = new AtomicInteger();
        int count = this.size();
        MoveList outer = this;
        return () -> new Iterator<>() {
            @Override
            public boolean hasNext() {
                return index.get() < count;
            }

            @Override
            public IndexedMove next() {
                int i = index.get();
                index.getAndIncrement();
                outer.pickNextBestMove(i);
                return new IndexedMove(outer.get(i), i);
            }
        };
    }

    public record IndexedMove(Move move, int index) {}
}
