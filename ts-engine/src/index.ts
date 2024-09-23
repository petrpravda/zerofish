import { BoardState } from './BoardState';
import {START_POS} from './Fen';
import { Perft } from './Perft';

// src/index.ts
export { BB64Long } from './BB64Long';
export { BoardState } from './BoardState';
export { Fen } from './Fen';
export { MoveList } from './MoveList';
export { Move } from './Move';
export { PieceSquareTable } from './PieceSquareTable';
export { PieceType } from './PieceType';
export { ScoredMove } from './ScoredMove';
export { Square } from './Square';
export { TranspositionTable } from './TranspositionTable';
export { Zobrist } from './Zobrist';
export { Bitboard } from './Bitboard';
export { Constants } from './Constants';
export { MoveOrdering } from './MoveOrdering';
export { Perft } from './Perft';
export { Piece } from './Piece';
export { Side } from './Side';
export { TTEntry } from './TTEntry';


// const board = BoardState.fromFen(START_POS);
// console.info(Perft.perftString(board, 1));
