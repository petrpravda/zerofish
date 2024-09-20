import {Move} from '../Move';
import {BoardState} from '../BoardState';
import {START_POS} from '../Fen';
import {ScoredMove} from '../ScoredMove';
import {TranspositionTable} from '../TranspositionTable';
import {MoveList} from '../MoveList';
import {fromBigInt} from '../BB64Long';

let moveList: MoveList;

beforeEach(() => {
  moveList = new MoveList();
});

test('testMakeQ_AddsQuietMoves', () => {
  const to = fromBigInt(0b101n);
  const fromSq = 10;

  // Test makeQ
  moveList.makeQ(fromSq, to);

  // Verify the correct moves are added to the list
  expect(moveList.length).toBe(2);
  expect(moveList[0]).toEqual(new Move(fromSq, 0, Move.QUIET));
  expect(moveList[1]).toEqual(new Move(fromSq, 2, Move.QUIET));
});

test('testMakeC_AddsCaptureMoves', () => {
  const to = fromBigInt(0b110n); // binary for testing (positions with LSBs)
  const fromSq = 12;

  // Test makeC
  moveList.makeC(fromSq, to);

  // Verify the correct moves are added to the list
  expect(moveList.length).toBe(2);
  expect(moveList[0]).toEqual(new Move(fromSq, 1, Move.CAPTURE));
  expect(moveList[1]).toEqual(new Move(fromSq, 2, Move.CAPTURE));
});

test('testMakeDP_AddsDoublePushMoves', () => {
  const to = fromBigInt(0b11n);
  const fromSq = 15;

  // Test makeDP
  moveList.makeDP(fromSq, to);

  // Verify the correct moves are added to the list
  expect(moveList.length).toBe(2);
  expect(moveList[0]).toEqual(new Move(fromSq, 0, Move.DOUBLE_PUSH));
  expect(moveList[1]).toEqual(new Move(fromSq, 1, Move.DOUBLE_PUSH));
});

test('testMakePC_AddsPromotionMoves', () => {
  const to = fromBigInt(0b10n); // binary for testing (positions with LSBs)
  const fromSq = 9;

  // Test makePC
  moveList.makePC(fromSq, to);

  // Verify the correct promotion moves are added to the list
  expect(moveList.length).toBe(4);
  expect(moveList[0]).toEqual(new Move(fromSq, 1, Move.PC_KNIGHT));
  expect(moveList[1]).toEqual(new Move(fromSq, 1, Move.PC_BISHOP));
  expect(moveList[2]).toEqual(new Move(fromSq, 1, Move.PC_ROOK));
  expect(moveList[3]).toEqual(new Move(fromSq, 1, Move.PC_QUEEN));
});

test('testPickNextBestMove_PicksHighestScore', () => {
  const scoredMoves = [
    new ScoredMove(new Move(1, 2, Move.QUIET), 10),
    new ScoredMove(new Move(2, 3, Move.CAPTURE), 20),
  ];

  // Swap the best move to the front
  moveList.pickNextBestMove(0, scoredMoves);

  // Verify that the highest score is now first
  expect(scoredMoves[0].score).toBe(20);
  expect(scoredMoves[1].score).toBe(10);
});

test('testOverSorted_IteratorCorrectlyIterates', () => {
  const boardState = BoardState.fromFen(START_POS);
  const transpositionTable = new TranspositionTable();
  const moves = boardState.generateLegalMoves();

  const leftPawnMove = Move.fromUciString('d2d4', boardState);
  const rightPawnMove = Move.fromUciString('e2e4', boardState);
  moveList.push(leftPawnMove);
  moveList.push(rightPawnMove);

  const iterable = moveList.overSorted(boardState, transpositionTable);
  const iterator = iterable[Symbol.iterator]();

  let nextMove = iterator.next().value;
  expect(nextMove.toString()).toEqual(leftPawnMove.toString()); // This should be the higher score
  nextMove = iterator.next().value;
  expect(nextMove.toString()).toEqual(rightPawnMove.toString());
  expect(iterator.next().done).toBe(true);
});
