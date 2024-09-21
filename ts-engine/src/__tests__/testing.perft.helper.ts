import {BB64Long} from '../BB64Long';
import {BoardState} from '../BoardState';
import {START_POS} from '../Fen';
import {Perft} from '../Perft';

// const board = BoardState.fromFen(START_POS);
// const board = BoardState.fromFen("r1bqkbnr/pppppppp/n7/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2");
//const board = BoardState.fromFen("rnbqkbnr/pppppppp/8/8/8/3P4/PPP1PPPP/RNBQKBNR b KQkq - 0 1");
const board = BoardState.fromFen("r1bqkbnr/pppppppp/n7/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 2");
console.info(Perft.perftString(board, 1));
