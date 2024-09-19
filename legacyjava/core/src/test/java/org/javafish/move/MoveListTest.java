package org.javafish.move;

import org.javafish.board.BoardState;
import org.javafish.board.Fen;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import search.TranspositionTable;

import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MoveListTest {

    private MoveList moveList;

    @BeforeEach
    void setUp() {
        moveList = new MoveList();
    }

    @Test
    void testMakeQ_AddsQuietMoves() {
        long to = 0b101;  // binary for testing (positions with LSBs)
        int fromSq = 10;

        // Test makeQ
        moveList.makeQ(fromSq, to);

        // Verify the correct moves are added to the list
        assertEquals(2, moveList.size());
        assertEquals(new Move(fromSq, 0, Move.QUIET), moveList.get(0));
        assertEquals(new Move(fromSq, 2, Move.QUIET), moveList.get(1));
    }

    @Test
    void testMakeC_AddsCaptureMoves() {
        long to = 0b110;  // binary for testing (positions with LSBs)
        int fromSq = 12;

        // Test makeC
        moveList.makeC(fromSq, to);

        // Verify the correct moves are added to the list
        assertEquals(2, moveList.size());
        assertEquals(new Move(fromSq, 1, Move.CAPTURE), moveList.get(0));
        assertEquals(new Move(fromSq, 2, Move.CAPTURE), moveList.get(1));
    }

    @Test
    void testMakeDP_AddsDoublePushMoves() {
        long to = 0b11;  // binary for testing (positions with LSBs)
        int fromSq = 15;

        // Test makeDP
        moveList.makeDP(fromSq, to);

        // Verify the correct moves are added to the list
        assertEquals(2, moveList.size());
        assertEquals(new Move(fromSq, 0, Move.DOUBLE_PUSH), moveList.get(0));
        assertEquals(new Move(fromSq, 1, Move.DOUBLE_PUSH), moveList.get(1));
    }

    @Test
    void testMakePC_AddsPromotionMoves() {
        long to = 0b10;  // binary for testing (positions with LSBs)
        int fromSq = 9;

        // Test makePC
        moveList.makePC(fromSq, to);

        // Verify the correct promotion moves are added to the list
        assertEquals(4, moveList.size());
        assertEquals(new Move(fromSq, 1, Move.PC_KNIGHT), moveList.get(0));
        assertEquals(new Move(fromSq, 1, Move.PC_BISHOP), moveList.get(1));
        assertEquals(new Move(fromSq, 1, Move.PC_ROOK), moveList.get(2));
        assertEquals(new Move(fromSq, 1, Move.PC_QUEEN), moveList.get(3));
    }

    @Test
    void testPickNextBestMove_PicksHighestScore() {
        List<ScoredMove> scoredMoves = new ArrayList<>(List.of(
                new ScoredMove(new Move(1, 2, Move.QUIET), 10),
                new ScoredMove(new Move(2, 3, Move.CAPTURE), 20)
        ));

        // Swap the best move to the front
        moveList.pickNextBestMove(0, scoredMoves);

        // Verify that the highest score is now first
        assertEquals(20, scoredMoves.get(0).score);
        assertEquals(10, scoredMoves.get(1).score);
    }

    @Test
    void testOverSorted_IteratorCorrectlyIterates() {
        BoardState boardState = BoardState.fromFen(Fen.START_POS);
        TranspositionTable transpositionTable = new TranspositionTable();

        Move leftPawnMove = Move.fromUciString("d2d4", boardState);
        Move rightPawnMove = Move.fromUciString("e2e4", boardState);
        moveList.add(leftPawnMove);
        moveList.add(rightPawnMove);

        Iterable<Move> iterable = moveList.overSorted(boardState, transpositionTable);
        Iterator<Move> iterator = iterable.iterator();

        assertTrue(iterator.hasNext());
        Move nextMove = iterator.next();
        assertEquals(leftPawnMove, nextMove);  // This should be the highest score
        assertTrue(iterator.hasNext());
        nextMove = iterator.next();
        assertEquals(rightPawnMove, nextMove);
        assertFalse(iterator.hasNext());
    }

    // Additional tests can be implemented similarly
}
