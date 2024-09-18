import {Piece} from '../Piece';

describe('Piece', () => {

  test('flip', () => {
    // Test flipping between white and black pieces
    expect(Piece.flip(Piece.WHITE_PAWN)).toBe(Piece.BLACK_PAWN);
    expect(Piece.flip(Piece.BLACK_KNIGHT)).toBe(Piece.WHITE_KNIGHT);
    expect(Piece.flip(Piece.WHITE_BISHOP)).toBe(Piece.BLACK_BISHOP);
    expect(Piece.flip(Piece.BLACK_ROOK)).toBe(Piece.WHITE_ROOK);
    expect(Piece.flip(Piece.WHITE_QUEEN)).toBe(Piece.BLACK_QUEEN);
    expect(Piece.flip(Piece.BLACK_KING)).toBe(Piece.WHITE_KING);
  });

  test('typeOf', () => {
    // Test getting the piece type without regard to color
    expect(Piece.typeOf(Piece.WHITE_PAWN)).toBe(0);      // Pawn
    expect(Piece.typeOf(Piece.BLACK_KNIGHT)).toBe(1);    // Knight
    expect(Piece.typeOf(Piece.WHITE_BISHOP)).toBe(2);    // Bishop
    expect(Piece.typeOf(Piece.BLACK_ROOK)).toBe(3);      // Rook
    expect(Piece.typeOf(Piece.WHITE_QUEEN)).toBe(4);     // Queen
    expect(Piece.typeOf(Piece.BLACK_KING)).toBe(5);      // King
  });

  test('sideOf', () => {
    // Test getting the side (white or black)
    expect(Piece.sideOf(Piece.WHITE_PAWN)).toBe(0);      // White
    expect(Piece.sideOf(Piece.BLACK_KNIGHT)).toBe(1);    // Black
    expect(Piece.sideOf(Piece.WHITE_BISHOP)).toBe(0);    // White
    expect(Piece.sideOf(Piece.BLACK_ROOK)).toBe(1);      // Black
    expect(Piece.sideOf(Piece.WHITE_QUEEN)).toBe(0);     // White
    expect(Piece.sideOf(Piece.BLACK_KING)).toBe(1);      // Black
  });

  test('makePiece', () => {
    // Test making a piece from a side and piece type
    expect(Piece.makePiece(0, 0)).toBe(Piece.WHITE_PAWN);      // White Pawn
    expect(Piece.makePiece(1, 1)).toBe(Piece.BLACK_KNIGHT);    // Black Knight
    expect(Piece.makePiece(0, 2)).toBe(Piece.WHITE_BISHOP);    // White Bishop
    expect(Piece.makePiece(1, 3)).toBe(Piece.BLACK_ROOK);      // Black Rook
    expect(Piece.makePiece(0, 4)).toBe(Piece.WHITE_QUEEN);     // White Queen
    expect(Piece.makePiece(1, 5)).toBe(Piece.BLACK_KING);      // Black King
  });

  test('getNotation', () => {
    // Test the correct notation for each piece
    expect(Piece.getNotation(Piece.WHITE_PAWN)).toBe('P');     // White Pawn
    expect(Piece.getNotation(Piece.WHITE_KNIGHT)).toBe('N');   // White Knight
    expect(Piece.getNotation(Piece.WHITE_BISHOP)).toBe('B');   // White Bishop
    expect(Piece.getNotation(Piece.WHITE_ROOK)).toBe('R');     // White Rook
    expect(Piece.getNotation(Piece.WHITE_QUEEN)).toBe('Q');    // White Queen
    expect(Piece.getNotation(Piece.WHITE_KING)).toBe('K');     // White King

    expect(Piece.getNotation(Piece.BLACK_PAWN)).toBe('p');     // Black Pawn
    expect(Piece.getNotation(Piece.BLACK_KNIGHT)).toBe('n');   // Black Knight
    expect(Piece.getNotation(Piece.BLACK_BISHOP)).toBe('b');   // Black Bishop
    expect(Piece.getNotation(Piece.BLACK_ROOK)).toBe('r');     // Black Rook
    expect(Piece.getNotation(Piece.BLACK_QUEEN)).toBe('q');    // Black Queen
    expect(Piece.getNotation(Piece.BLACK_KING)).toBe('k');     // Black King

    expect(Piece.getNotation(Piece.NONE)).toBe(' ');           // No piece
  });
});
