import {BB64Long, bitboardToString} from '../BB64Long';
import {Side} from '../Side';
import {Bitboard} from '../Bitboard';

describe('Bitboard Tests', () => {

  // test('testPawnAttacksWhite', () => {
  //   const pawns =           new BB64Long(0b00000000_00000000_00000000_00000000_00001000_00000000_00000000_00000000n);
  //   const expectedAttacks = new BB64Long(0b00000000_00000000_00000000_00010100_00000000_00000000_00000000_00000000n);
  //   const pawnAttacks: BB64Long = Bitboard.pawnAttacks(pawns, Side.WHITE);
  //   expect(pawnAttacks.equals(expectedAttacks)).toBe(true); // Compare using equals method
  // });
  //
  // test('testPawnAttacksBlack', () => {
  //   const pawns =           new BB64Long(0b00000000_00000000_00000000_01000001_00000000_00000000_00000000_00000000n);
  //   const expectedAttacks = new BB64Long(0b00000000_00000000_00000000_00000000_10100010_00000000_00000000_00000000n);
  //   const pawnAttacks = Bitboard.pawnAttacks(pawns, Side.BLACK);
  //   expect(pawnAttacks.equals(expectedAttacks)).toBe(true); // Use equals method
  // });

  test('testGetKnightAttacks', () => {
    const knightPosition = 38;
    const expectedAttacks = new BB64Long(0b00000000_10100000_00010000_00000000_00010000_10100000_00000000_00000000n);
    const knightAttacks = Bitboard.getKnightAttacks(knightPosition);
    expect(knightAttacks.equals(expectedAttacks)).toBe(true);
  });

  test('testGetKingAttacks', () => {
    const kingPosition = 28;
    const expectedAttacks = new BB64Long(0b00000000_00000000_00000000_00111000_00101000_00111000_00000000_00000000n);
    const kingAttacks = Bitboard.getKingAttacks(kingPosition);
    expect(kingAttacks.equals(expectedAttacks)).toBe(true);
  });

  test('testGetRookAttacks', () => {
    const rookPosition = 29;
    const occupied = new BB64Long(0b01010101_01110011_00001000_00101000_00100010_11010001_01010100_01110101n);
    const expectedAttacks = new BB64Long(0b00000000_00000000_00000000_00100000_11011110_00100000_00100000_00100000n);
    const rookAttacks = Bitboard.getRookAttacks(rookPosition, occupied);
    console.info(bitboardToString(expectedAttacks));
    console.info(bitboardToString(rookAttacks));
    expect(rookAttacks.equals(expectedAttacks)).toBe(true);
  });

  // test('testGetBishopAttacks', () => {
  //   const bishopPosition = 27;
  //   const occupied = new BB64Long(0b01010101_01110011_00001000_00101000_00100010_11010001_01010100_01110101n);
  //   const expectedAttacks = new BB64Long(0b00000000_01000001_00100010_00010100_00000000_00010100_00000010_00000001n);
  //   const bishopAttacks = Bitboard.getBishopAttacks(bishopPosition, occupied);
  //   expect(bishopAttacks.equals(expectedAttacks)).toBe(true);
  // });
  //
  // test('testGetQueenAttacks', () => {
  //   const queenPosition = 27;
  //   const occupied = Bitboard.FULL_BOARD.AND_NOT(new BB64Long(1n << BigInt(queenPosition)));
  //   const expectedAttacks = Bitboard.getRookAttacks(queenPosition, occupied).OR(Bitboard.getBishopAttacks(queenPosition, occupied));
  //   expect(Bitboard.getQueenAttacks(queenPosition, occupied).equals(expectedAttacks)).toBe(true);
  // });

  // Further tests...
});
