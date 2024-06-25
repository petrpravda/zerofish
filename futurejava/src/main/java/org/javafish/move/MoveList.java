package org.javafish.move;

import org.javafish.bitboard.Bitboard;
import org.javafish.board.BoardState;
import org.javafish.board.Side;
import search.MoveOrdering;
import search.TTEntry;
import search.TranspositionTable;

import java.util.ArrayList;
import java.util.Collections;
import java.util.Iterator;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Collectors;

import static org.javafish.eval.PieceSquareTable.MGS;

public class MoveList extends ArrayList<Move> {

    // private List<ScoredMove> sortedList;
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

            result = this.stream().map(move -> {
                        int moveScore = move.equals(hashMove) ? MoveOrdering.HashMoveScore : 0;

//            if (moveOrdering.isKiller(state, move, ply)) {
//                move.addToScore(MoveOrdering.KillerMoveScore);
//            }
                        int piece = state.items[move.from()];
                        int piecesScore = switch (move.flags()) {
                            case Move.PC_BISHOP, Move.PC_KNIGHT, Move.PC_ROOK, Move.PC_QUEEN ->
                                    MGS[move.getPieceTypeForSide(state.getSideToPlay())][move.to()] - MGS[piece][move.from()]
                                            - MGS[state.items[move.to()]][move.to()];
                            case Move.PR_BISHOP, Move.PR_KNIGHT, Move.PR_ROOK, Move.PR_QUEEN ->
                                    MGS[move.getPieceTypeForSide(state.getSideToPlay())][move.to()] - MGS[piece][move.from()];
                            case Move.CAPTURE ->
                                    MGS[piece][move.to()] - MGS[piece][move.from()] - MGS[state.items[move.to()]][move.to()];
                            case Move.QUIET, Move.EN_PASSANT, Move.DOUBLE_PUSH, Move.OO, Move.OOO ->
                                    MGS[piece][move.to()] - MGS[piece][move.from()];
                            default -> throw new IllegalStateException();
                        };

                        int totalScore = moveScore + piecesScore * (state.getSideToPlay() == Side.WHITE ? 1 : -1);
                        return new ScoredMove(move, totalScore);
                    })
                    .collect(Collectors.toCollection(ArrayList::new));
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
