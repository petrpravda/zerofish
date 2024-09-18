import {Move} from '../Move';

describe('Move tests', () => {

  test('Move constructor with flags', () => {
    const move = new Move(12, 28, Move.CAPTURE);
    expect(move.from()).toBe(12);
    expect(move.to()).toBe(28);
    expect(move.flags()).toBe(Move.CAPTURE);
  });

  test('Move constructor without flags', () => {
    const move = new Move(12, 28);
    expect(move.from()).toBe(12);
    expect(move.to()).toBe(28);
    expect(move.flags()).toBe(Move.QUIET);
  });

  test('to() method', () => {
    const move = new Move(5, 23, Move.QUIET);
    expect(move.to()).toBe(23);
  });

  test('from() method', () => {
    const move = new Move(5, 23, Move.QUIET);
    expect(move.from()).toBe(5);
  });

  test('flags() method', () => {
    const move = new Move(12, 28, Move.EN_PASSANT);
    expect(move.flags()).toBe(Move.EN_PASSANT);
  });

  test('isPromotion()', () => {
    const movePromotion = new Move(12, 28, Move.PR_QUEEN);
    expect(movePromotion.isPromotion()).toBe(true);

    const moveNonPromotion = new Move(12, 28, Move.QUIET);
    expect(moveNonPromotion.isPromotion()).toBe(false);
  });

  test('equals()', () => {
    const move1 = new Move(12, 28, Move.CAPTURE);
    const move2 = new Move(12, 28, Move.CAPTURE);
    expect(move1).toEqual(move2);

    const move3 = new Move(12, 28, Move.QUIET);
    expect(move1).not.toEqual(move3);
  });

  test('toString() for non-null move', () => {
    const move = new Move(12, 28, Move.QUIET);
    expect(move.toString()).not.toBe('NULL_MOVE');
  });

  test('isNullMove()', () => {
    const nullMove = new Move(0, 0, Move.NULL);
    expect(nullMove.isNullMove()).toBe(true);

    const nonNullMove = new Move(12, 28, Move.QUIET);
    expect(nonNullMove.isNullMove()).toBe(false);
  });

  test('uci() method for quiet move', () => {
    const move = new Move(40, 48, Move.QUIET);
    expect(move.uci()).toBe('a6a7'); // Adjusted UCI based on Square class
  });

  test('uci() method for promotion move', () => {
    const promotionMove = new Move(48, 56, Move.PR_QUEEN);
    expect(promotionMove.uci()).toBe('a7a8q');
  });

  test('getPieceType()', () => {
    const move = new Move(12, 28, Move.CAPTURE);
    expect(move.getPieceType()).toBe(1);

    const promotionMove = new Move(12, 28, Move.PC_QUEEN);
    expect(promotionMove.getPieceType()).toBe(4);
  });

  test('getPieceTypeForSide()', () => {
    const move = new Move(12, 28, Move.CAPTURE);
    expect(move.getPieceTypeForSide(1)).toBe(9); // Assuming side 1 is white
  });

  test('isCastling()', () => {
    const moveOO = new Move(4, 6, Move.OO);
    expect(moveOO.isCastling()).toBe(true);

    const moveOOO = new Move(4, 2, Move.OOO);
    expect(moveOOO.isCastling()).toBe(true);

    const regularMove = new Move(4, 6, Move.QUIET);
    expect(regularMove.isCastling()).toBe(false);
  });

  test('fromUciString() for promotion', () => {
    const state = BoardState.fromFen("6k1/P7/4p1B1/4B3/8/2Q5/2PK1P1P/R6R w - - 1 40");
    const move = Move.fromUciString('a7a8q', state);
    expect(move).not.toBeNull();
    expect(move.uci()).toBe('a7a8q');
  });

  test('fromUciString() for normal move', () => {
    const state = BoardState.fromFen("5k2/8/P3p1B1/4B3/8/2Q5/2PK1P1P/R6R w - - 1 39");
    const move = Move.fromUciString('a6a7', state);
    expect(move).not.toBeNull();
    expect(move.uci()).toBe('a6a7');
  });

  test('Null move constant', () => {
    expect(Move.NULL_MOVE).toEqual(new Move(0, 0, Move.NULL));
  });
});
