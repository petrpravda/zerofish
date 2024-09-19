import {Piece} from '../Piece';
import {PieceSquareTable} from '../PieceSquareTable';

describe('PieceSquareTable Tests', () => {
  test('MGS for White Pawn', () => {
    expect(PieceSquareTable.MGS[Piece.WHITE_PAWN][50]).toBe(149);
  });

  test('MGS for Black Knight', () => {
    expect(PieceSquareTable.MGS[Piece.BLACK_KNIGHT][20]).toBe(-401);
  });

  test('EGS for White Bishop', () => {
    expect(PieceSquareTable.EGS[Piece.WHITE_BISHOP][30]).toBe(314);
  });

  test('EGS for Black Queen', () => {
    expect(PieceSquareTable.EGS[Piece.BLACK_QUEEN][40]).toBe(-1000);
  });

  test('MGS for White King', () => {
    expect(PieceSquareTable.MGS[Piece.WHITE_KING][22]).toBe(1487);
  });

  test('EGS for Black Rook', () => {
    expect(PieceSquareTable.EGS[Piece.BLACK_ROOK][33]).toBe(-521);
  });

  test('MGS for White Queen', () => {
    expect(PieceSquareTable.MGS[Piece.WHITE_QUEEN][44]).toBe(1041);
  });

  test('EGS for Black Pawn', () => {
    expect(PieceSquareTable.EGS[Piece.BLACK_PAWN][55]).toBe(-70);
  });

  test('MGS for White Rook', () => {
    expect(PieceSquareTable.MGS[Piece.WHITE_ROOK][28]).toBe(472);
  });

  test('EGS for Black Bishop', () => {
    expect(PieceSquareTable.EGS[Piece.BLACK_BISHOP][38]).toBe(-314);
  });
});
