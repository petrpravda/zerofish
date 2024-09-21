import {Side} from '../Side';
import {BB64Long, BB_ZERO, bitboardToString} from '../BB64Long';
import {BoardState} from '../BoardState';
import {Square} from '../Square';
import {Piece} from '../Piece';
import {Move} from '../Move';
import {START_POS} from '../Fen';
import {bitboardToNormalized, normalizeBitboard} from './testing.helper';

describe('BoardStateTest', () => {

  // it('complexPositionTroubleWithScore', () => {
  //   const position = BoardState.fromFen("r2q1rk1/pbp2ppp/1pnp1n2/8/2PPp3/2P1P3/P1N2PPP/R1BQKB1R w kq - 0 10");
  //   new Search(new TranspositionTable()).itDeep(position, 9);
  // });

  it('attackedPieces', () => {
    const state = BoardState.fromFen("5k2/p6p/1p1r4/1PpP1P2/2P5/P4K2/8/7R b - - 1 40");
    const attackedPieces = state.attackedPieces(Side.BLACK);
    const expected = normalizeBitboard`
            . . . . . . . . 
            . . . . . . . X 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
        `;
    expect(bitboardToNormalized(attackedPieces)).toEqual(expected);
  });

  it('attackedPiecesWithPinning', () => {
    const state = BoardState.fromFen("6q1/1p2bpk1/1r4p1/3pPB2/1n1P2Q1/6P1/3N2K1/7R b - - 4 39");
    const attackedPieces = state.attackedPieces(Side.WHITE);
    expect(bitboardToNormalized(attackedPieces)).toEqual(normalizeBitboard`
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
        `);
  });

  it('attackedPiecesUndefended', () => {
    let state = BoardState.fromFen("5k2/p6p/1p1r4/1PpP1P2/2P5/P4K2/8/7R b - - 1 40");
    let attackedPiecesUndefended = state.attackedPiecesUndefended(Side.BLACK);
    expect(bitboardToNormalized(attackedPiecesUndefended)).toEqual(normalizeBitboard`
            . . . . . . . . 
            . . . . . . . X 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
        `);

    state = BoardState.fromFen("8/p5kp/1p1r4/1PpP1P2/2P5/P4K2/8/7R w - - 2 41");
    attackedPiecesUndefended = state.attackedPiecesUndefended(Side.BLACK);
    expect(attackedPiecesUndefended).toEqual(BB_ZERO);
  });

  it('attackedPiecesUndefendedBehindSlidingAttacker', () => {
    let state = BoardState.fromFen("r2qk2r/pp1nbpQp/2p1p1b1/8/4P1P1/5N1P/PPP2PB1/R1B2RK1 b kq - 0 13");
    let attackedPiecesUndefended = state.attackedPiecesUndefended(Side.BLACK);
    expect(bitboardToNormalized(attackedPiecesUndefended)).toEqual(normalizeBitboard`
            . . . . . . . X 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
        `);

    state = BoardState.fromFen("r2qk2r/pp1n1pQp/2p1pbb1/8/4P1P1/5N1P/PPP2PB1/R1B2RK1 w kq - 1 14");
    attackedPiecesUndefended = state.attackedPiecesUndefended(Side.BLACK);
    expect(bitboardToNormalized(attackedPiecesUndefended)).toEqual(normalizeBitboard`
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
        `);
  });

  it('mgValueTest', () => {
    let state = BoardState.fromFen("8/8/8/8/8/8/8/8 w KQkq - 0 1");
    expect(state.mg).toEqual(0);
    state.setPieceAt(Piece.WHITE_ROOK, Square.getSquareFromName("d5"));
    expect(state.mg).toEqual(482);
    state.setPieceAt(Piece.BLACK_ROOK, Square.getSquareFromName("d2"));
    expect(state.mg).toEqual(-20);
  });

  it('fromFailingSts', () => {
    const state = BoardState.fromFen("2r5/p3k1p1/1p5p/4Pp2/1PPnK3/PB1R2P1/7P/8 w - f6 0 4");
    const moves = state.generateLegalMoves();

    const expectedMoves = ["e4e3", "e4f4", "e4d5", "e4d4", "e5f6"];
    const actualMoves = moves.map((move: Move) => move.toString());

    expect(actualMoves).toEqual(expectedMoves);
  });

  // it('seeScore', () => {
  //   const state = BoardState.fromFen("6k1/5pp1/4p2p/8/5P2/4RQP1/rq1rR2P/5K2 b - - 3 33");
  //   const result = state.seeScore(12, Side.BLACK);
  //   expect(result.score()).toEqual(0);
  // });

  it('testClone', () => {
    const original = BoardState.fromFen(START_POS);
    const cloned = original.clone();

    expect(cloned.hash).toEqual(original.hash);
    expect(cloned.items).toEqual(original.items);
    expect(cloned.ply).toEqual(original.ply);
    expect(cloned.sideToPlay).toEqual(original.sideToPlay);
  });

  it('testMovePiece', () => {
    const stateBeforeMove = BoardState.fromFen(START_POS);
    const fromSq = Square.getSquareFromName("d2");
    const toSq = Square.getSquareFromName("d4");

    const stateAfterMove = stateBeforeMove.doMove(new Move(fromSq, toSq));

    expect(stateAfterMove.pieceAt(toSq)).toEqual(Piece.WHITE_PAWN);
    expect(stateAfterMove.pieceAt(fromSq)).toEqual(Piece.NONE);
  });

  it('testSetPieceAt', () => {
    const state = BoardState.fromFen(START_POS);

    const square = 20; // d2
    const piece = Piece.WHITE_KNIGHT;
    state.setPieceAt(piece, square);

    expect(state.pieceAt(square)).toEqual(piece);
  });

  it('testRemovePiece', () => {
    const state = BoardState.fromFen(START_POS);

    const square = 12; // e2
    state.removePiece(square);

    expect(state.pieceAt(square)).toEqual(Piece.NONE);
  });

  it('testHash', () => {
    const state = BoardState.fromFen(START_POS);
    const initialHash = state.hash;

    const stateAfterMove = state.doMove(Move.fromUciString("e2e4", state));
    const newHash = stateAfterMove.hash;

    expect(initialHash).not.toEqual(newHash);
  });

  it('testDiagonalSliders', () => {
    const state = BoardState.fromFen(START_POS);

    const whiteDiagonalSliders = state.diagonalSliders(Side.WHITE);
    const blackDiagonalSliders = state.diagonalSliders(Side.BLACK);

    expect(bitboardToNormalized(whiteDiagonalSliders)).toEqual(normalizeBitboard`
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . X X . X . . 
        `);

    expect(bitboardToNormalized(blackDiagonalSliders)).toEqual(normalizeBitboard`
            . . X X . X . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
        `);
  });

  it('testStraightSliders', () => {
    const state = BoardState.fromFen(START_POS);

    const whiteStraightSliders = state.orthogonalSliders(Side.WHITE);
    const blackStraightSliders = state.orthogonalSliders(Side.BLACK);

    expect(bitboardToNormalized(whiteStraightSliders)).toEqual(normalizeBitboard`
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            X . . X . . . X 
        `);

    expect(bitboardToNormalized(blackStraightSliders)).toEqual(normalizeBitboard`
            X . . X . . . X 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
            . . . . . . . . 
        `);
  });

  it('not possible to do non-sense a1g1 move', () => {
    const state = BoardState.fromFen('r1bqkbnr/pppppppp/n7/8/8/5N2/PPPPPPPP/RNBQKB1R w KQkq - 0 2');
    const moves: string[] = state.generateLegalMoves().map(m => m.toString());

    expect(moves).toContain('h2h4');

    // Assert that 'a1g1' is not a legal move
    expect(moves).not.toContain('a1g1');
  });

  it('check that move c1g5 is not missing', () => {
    const state = BoardState.fromFen('r1bqkbnr/pppppppp/n7/8/8/3P4/PPP1PPPP/RNBQKBNR w KQkq - 0 2');
    const moves: string[] = state.generateLegalMoves().map(m => m.toString());

    expect(moves).toContain('c1g5');
  });
});
