import { Move } from './Move'; // Import Move and ScoredMove models
import { BoardState } from './BoardState';
import { Bitboard } from './Bitboard';
import { BB64Long } from './BB64Long';
import { Side } from './Side';
import {ScoredMove} from './ScoredMove';
import {TranspositionTable} from './TranspositionTable';
import {MoveOrdering} from './MoveOrdering';
import {PieceSquareTable} from './PieceSquareTable'; // Assuming Side is an enum for WHITE/BLACK

export class MoveList extends Array<Move> {

  constructor() {
    super();
  }

  makeQ(fromSq: number, to: BB64Long) {
    let toSq: number;
    while (!to.empty()) {
      toSq = to.LSB();
      to = to.popLSB();
      this.push(new Move(fromSq, toSq, Move.QUIET));
    }
  }

  makeC(fromSq: number, to: BB64Long) {
    let toSq: number;
    while (!to.empty()) {
      toSq = to.LSB();
      to = to.popLSB();
      this.push(new Move(fromSq, toSq, Move.CAPTURE));
    }
  }

  makeDP(fromSq: number, to: BB64Long) {
    let toSq: number;
    while (!to.empty()) {
      toSq = to.LSB();
      to = to.popLSB();
      this.push(new Move(fromSq, toSq, Move.DOUBLE_PUSH));
    }
  }

  makePC(fromSq: number, to: BB64Long) {
    let toSq: number;
    while (!to.empty()) {
      toSq = to.LSB();
      to = to.popLSB();
      this.push(new Move(fromSq, toSq, Move.PC_KNIGHT));
      this.push(new Move(fromSq, toSq, Move.PC_BISHOP));
      this.push(new Move(fromSq, toSq, Move.PC_ROOK));
      this.push(new Move(fromSq, toSq, Move.PC_QUEEN));
    }
  }

  private scoreMoves(state: BoardState, transpositionTable: TranspositionTable): ScoredMove[] {
    if (this.length === 0) {
      return [];
    }

    const ttEntry = transpositionTable.probe(state.hash);
    const hashMove: Move|null = ttEntry ? ttEntry.move() : null;

    return this.map((move: Move) => {
      let moveScore = hashMove !== null && hashMove.equals(hashMove) ? MoveOrdering.HashMoveScore : 0;

      const piece = state.items[move.from()];
      const piecesScore = (() => {
        switch (move.flags()) {
          case Move.PC_BISHOP:
          case Move.PC_KNIGHT:
          case Move.PC_ROOK:
          case Move.PC_QUEEN:
            return PieceSquareTable.MGS[move.getPieceTypeForSide(state.sideToPlay)][move.to()] -
              PieceSquareTable.MGS[piece][move.from()] - PieceSquareTable.MGS[state.items[move.to()]][move.to()];
          case Move.PR_BISHOP:
          case Move.PR_KNIGHT:
          case Move.PR_ROOK:
          case Move.PR_QUEEN:
            return PieceSquareTable.MGS[move.getPieceTypeForSide(state.sideToPlay)][move.to()] - PieceSquareTable.MGS[piece][move.from()];
          case Move.CAPTURE:
            return PieceSquareTable.MGS[piece][move.to()] - PieceSquareTable.MGS[piece][move.from()] - PieceSquareTable.MGS[state.items[move.to()]][move.to()];
          case Move.QUIET:
          case Move.EN_PASSANT:
          case Move.DOUBLE_PUSH:
          case Move.OO:
          case Move.OOO:
            return PieceSquareTable.MGS[piece][move.to()] - PieceSquareTable.MGS[piece][move.from()];
          default:
            throw new Error('Invalid move flag');
        }
      })();

      const totalScore = moveScore + piecesScore * (state.sideToPlay === Side.WHITE ? 1 : -1);
      return new ScoredMove(move, totalScore);
    });
  }

  pickNextBestMove(curIndex: number, sortedList: ScoredMove[]) {
    let max = Number.MIN_SAFE_INTEGER;
    let maxIndex = -1;
    for (let i = curIndex; i < sortedList.length; i++) {
      if (sortedList[i].score > max) {
        max = sortedList[i].score;
        maxIndex = i;
      }
    }
    if (maxIndex !== -1) {
      [sortedList[curIndex], sortedList[maxIndex]] = [sortedList[maxIndex], sortedList[curIndex]]; // Swap
    }
  }

  overSorted(state: BoardState, transpositionTable: TranspositionTable): IterableIterator<Move> {
    const sortedList = this.scoreMoves(state, transpositionTable);
    let index = 0;
    const moveList = this;

    return {
      [Symbol.iterator](): IterableIterator<Move> {
        return this;
      },

      next(): IteratorResult<Move> {
        if (index < sortedList.length) {
          moveList.pickNextBestMove(index, sortedList);
          const move = sortedList[index].move;
          // console.info(`returning idx: ${index}, move: ${move.toString()}, score: ${sortedList[index].score}`);
          index++;
          return { value: move, done: false };
        } else {
          return { value: undefined, done: true };
        }
      }
    };
  }
}
