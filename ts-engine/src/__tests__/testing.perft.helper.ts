import {BB64Long} from '../BB64Long';
import {BoardState} from '../BoardState';
import {START_POS} from '../Fen';
import {Perft} from '../Perft';
import {Move} from '../Move';

const board = BoardState.fromFen(START_POS);
//const board = BoardState.fromFen("rnbqkb1r/pppppppp/8/8/4n3/3P4/PPPKPPPP/RNBQ1BNR w kq - 3 3");

// let board = BoardState.fromFen("8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1");
// let moves = 6;
// board = board.doMove(Move.fromUciString("b3a4", board)); moves--;
// board = board.doMove(Move.fromUciString("f7e8", board)); moves--;
// board = board.doMove(Move.fromUciString("c5c6", board)); moves--;
// board = board.doMove(Move.fromUciString("e8d7", board)); moves--;

console.info(Perft.perftString(board, 6));
