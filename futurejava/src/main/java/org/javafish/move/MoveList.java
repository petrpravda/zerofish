package org.javafish.move;

import org.javafish.bitboard.Bitboard;
import org.javafish.board.BoardState;
import org.javafish.board.Side;
import search.MoveOrdering;
import search.TTEntry;
import search.TranspositionTable;

import java.io.Closeable;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Collections;
import java.util.Iterator;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

import static org.javafish.eval.PieceSquareTable.MGS;

public class MoveList extends MyArrayList<Move> {
// public class MoveList extends ArrayList<Move> {

    // private List<ScoredMove> sortedList;
    public MoveList(){
        super(40);
        //super(218);
    }

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
            toSq = Bitboard.lsb(to);
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

    private List<ScoredMove> scoreMoves(final BoardState state, TranspositionTable transpositionTable) {
        final List<ScoredMove> result;

        if (this.size() == 0) {
            result = Collections.emptyList();
        } else {
            TTEntry ttEntry = transpositionTable.probe(state.hash());
            Move hashMove = ttEntry != null ? ttEntry.move() : null;

            /// tadytady
            //            if (moveOrdering.isKiller(state, move, ply)) {
            //                move.addToScore(MoveOrdering.KillerMoveScore);
            //            }
            ArrayList<ScoredMove> scoredMoves = new ArrayList<>();
            for (int i = 0; i < this.size(); i++) {
                Move move1 = this.get(i);
                int moveScore = move1.equals(hashMove) ? MoveOrdering.HashMoveScore : 0;

//            if (moveOrdering.isKiller(state, move, ply)) {
//                move.addToScore(MoveOrdering.KillerMoveScore);
//            }
                int piece = state.items[move1.from()];
                int piecesScore = switch (move1.flags()) {
                    case Move.PC_BISHOP, Move.PC_KNIGHT, Move.PC_ROOK, Move.PC_QUEEN ->
                            MGS[move1.getPieceTypeForSide(state.getSideToPlay())][move1.to()] -
                                    MGS[piece][move1.from()] - MGS[state.items[move1.to()]][move1.to()];
                    case Move.PR_BISHOP, Move.PR_KNIGHT, Move.PR_ROOK, Move.PR_QUEEN ->
                            MGS[move1.getPieceTypeForSide(state.getSideToPlay())][move1.to()] -
                                    MGS[piece][move1.from()];
                    case Move.CAPTURE -> MGS[piece][move1.to()] - MGS[piece][move1.from()] -
                            MGS[state.items[move1.to()]][move1.to()];
                    case Move.QUIET, Move.EN_PASSANT, Move.DOUBLE_PUSH, Move.OO, Move.OOO ->
                            MGS[piece][move1.to()] - MGS[piece][move1.from()];
                    default -> throw new IllegalStateException();
                };

                int totalScore = moveScore + piecesScore * (state.getSideToPlay() == Side.WHITE ? 1 : -1);
                ScoredMove apply = new ScoredMove(move1, totalScore);
                scoredMoves.add(apply);
            }
            result = scoredMoves;
        }

        return result;
    }

    void pickNextBestMove(int curIndex, List<ScoredMove> sortedList) {
        int max = Integer.MIN_VALUE;
        int maxIndex = -1;
        for (int i = curIndex; i < sortedList.size(); i++){
            if (sortedList.get(i).score > max){
                max = sortedList.get(i).score;
                maxIndex = i;
            }
        }
        Collections.swap(sortedList, curIndex, maxIndex);
    }

    public Iterable<Move> overSorted(final BoardState state, TranspositionTable transpositionTable/*, int ply, MoveOrdering moveOrdering*/) {
        List<ScoredMove> sortedList = scoreMoves(state, transpositionTable/*, ply, moveOrdering*/);
        AtomicInteger index = new AtomicInteger();
        int count = sortedList.size();
        MoveList outer = this;
        return () -> new Iterator<>() {
            @Override
            public boolean hasNext() {
                return index.get() < count;
            }

            @Override
            public Move next() {
                int i = index.get();
                index.getAndIncrement();
                outer.pickNextBestMove(i, sortedList);
                return sortedList.get(i).move;
            }
        };
    }
}
