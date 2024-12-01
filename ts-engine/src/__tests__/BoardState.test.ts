import {Side} from '../Side';
import {BB_ZERO} from '../BB64Long';
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
    // expect(cloned.ply).toEqual(original.ply);
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

  it('check that move f2e1q is not missing', () => {
    const state = BoardState.fromFen('8/7p/p7/P3P3/4b3/R2rN1kn/1P3p2/4BK2 b - - 3 50');
    const moves: string[] = state.generateLegalMoves().map(m => m.toString());

    expect(moves).toContain('f2e1q');
  });

  describe('generateUciMoves', () => {
    it('should generate the correct UCI moves from the starting position', () => {
      const board = BoardState.fromFen('r1b1k2r/p2pbppp/1p2pn2/1PpP4/8/2B1PN1P/5PP1/R2QKB1R w KQkq - 0 14');
      const expectedUciMoves = ["e1d2", "e1e2", "f3g1",
"f3d2",
"f3h2",
"f3d4",
"f3h4",
"f3e5",
"f3g5",
"d1c2",
"d1e2",
"d1b3",
"d1a4",
"f1e2",
"f1d3",
"f1c4",
"c3f6",
"c3b2",
"c3d2",
"c3b4",
"c3d4",
"c3a5",
"c3e5",
"a1a7",
"a1b1",
"a1c1",
"a1a2",
"a1a3",
"a1a4",
"a1a5",
"a1a6",
"d1b1",
"d1c1",
"d1d2",
"d1d3",
"d1d4",
"h1g1",
"h1h2",
"g2g3",
"e3e4",
"h3h4",
"d5d6",
"g2g4",
"d5e6"
      ];

      const uciMoves = board.generateUciMoves();

      expect(uciMoves).toEqual(expectedUciMoves);
    });

    it('should generate the correct UCI moves after some moves', () => {
      let board = BoardState.fromFen('r1b1k2r/p2pbppp/1p2pn2/1PpP4/8/2B1PN1P/5PP1/R2QKB1R w KQkq - 0 14');
      // Make some moves
      board = board.doMove(Move.fromUciString('f3d4', board));
      board = board.doMove(Move.fromUciString('e7d6', board));
      board = board.doMove(Move.fromUciString('d1c2', board));
      board = board.doMove(Move.fromUciString('e6e5', board));

      const expectedUciMovesAfterMoves = [
        "e1d1",
        "e1d2",
        "e1e2",
        "e1c1",
        "d4e2",
        "d4b3",
        "d4f3",
        "d4f5",
        "d4c6",
        "d4e6",
        "f1e2",
        "f1d3",
        "f1c4",
        "c2h7",
        "c2b1",
        "c2d1",
        "c2b3",
        "c2d3",
        "c2a4",
        "c2e4",
        "c2f5",
        "c2g6",
        "c3b2",
        "c3d2",
        "c3b4",
        "c3a5",
        "a1a7",
        "a1b1",
        "a1c1",
        "a1d1",
        "a1a2",
        "a1a3",
        "a1a4",
        "a1a5",
        "a1a6",
        "h1g1",
        "h1h2",
        "c2c1",
        "c2a2",
        "c2b2",
        "c2d2",
        "c2e2",
        "f2f3",
        "g2g3",
        "e3e4",
        "h3h4",
        "f2f4",
        "g2g4"
      ];

      const uciMovesAfterMoves = board.generateUciMoves();

      expect(uciMovesAfterMoves).toEqual(expectedUciMovesAfterMoves);
    });
  });

  describe('hasNonPawnMaterial', () => {
    test('Normal position with non-pawn material for both sides', () => {
      const state = BoardState.fromFen("rnb1k2r/ppp1nppp/4p3/3pP3/6QN/2b5/PPP2PPP/R1B1KB1R w KQkq - 0 10");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has non-pawn material (Knight, Queen, Rook)
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(true);  // Black has non-pawn material (Knight, Bishop, Rook)
    });

    test('Only pawns on the board for both sides', () => {
      const state = BoardState.fromFen("8/8/8/4p3/4P3/8/8/8 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(false);  // White has no non-pawn material
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black has no non-pawn material
    });

    test('White has only pawns, Black has knights', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/PPPPPPPP/NNNNNNNN w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has non-pawn material (Knights)
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black only has pawns
    });

    test('Empty board, no material', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/8/8 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(false);  // White has no non-pawn material
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black has no non-pawn material
    });

    test('White has only a queen, Black has only pawns', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/8/Q7 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has a queen (non-pawn material)
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black has no non-pawn material
    });

    test('Black has a bishop and a rook, White has only pawns', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/pppppppp/rb6 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(false);  // White has no non-pawn material
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(true);  // Black has non-pawn material (Bishop, Rook)
    });

    test('Both sides have only kings', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/8/K7 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(false);  // White has no non-pawn material
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black has no non-pawn material
    });

    test('White has a rook, Black has only a king', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/8/R7 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has a rook (non-pawn material)
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black has no non-pawn material
    });

    test('White has bishops, Black has only pawns', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/pppppppp/BBBBBBBB w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has bishops
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black only has pawns
    });

    test('White and Black both have queens only', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/qqqqqqqq/QQQQQQQQ w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has queens
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(true);  // Black has queens
    });

    test('White has a knight, Black has nothing', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/8/N7 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has a knight
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black has nothing
    });

    test('Black has a knight, White has nothing', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/8/n7 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(false);  // White has nothing
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(true);  // Black has a knight
    });

    test('White has a bishop and pawns, Black has a rook', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/PPP5/B7 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has a bishop
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black has no non-pawn material
    });

    test('White and Black both have knights and bishops', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/nnnnnnnn/BBBBBBBB w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has bishops
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(true);  // Black has knights
    });

    test('White has no material, Black has pawns', () => {
      const state = BoardState.fromFen("8/8/8/8/8/8/8/p7 w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(false);  // White has no non-pawn material
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Black has no non-pawn material
    });

    test('Full material for both sides', () => {
      const state = BoardState.fromFen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has knights, bishops, rooks, and queens
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(true);  // Black has knights, bishops, rooks, and queens
    });

    test('Only White King left', () => {
      const state = BoardState.fromFen("8/k7/8/8/8/8/8/7K w - - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(false);  // Only White King left
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Only Black King left
    });

    test('White has two Rooks, Black has only a King', () => {
      const state = BoardState.fromFen("8/k7/8/8/8/8/8/R3K2R w KQ - 0 1");
      expect(state.hasNonPawnMaterial(Side.WHITE)).toBe(true);  // White has two Rooks
      expect(state.hasNonPawnMaterial(Side.BLACK)).toBe(false);  // Only Black King left
    });
  });
});
