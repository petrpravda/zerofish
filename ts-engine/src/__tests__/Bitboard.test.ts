import {BB64Long, bitboardToFormattedBinary, bitboardToString, fromBigInt, stringToBitboard} from '../BB64Long';
import {Side} from '../Side';
import {Bitboard, getLineAttacks, LineAttackMask, WHITE_KING_SIDE_CASTLING_BIT_PATTERN} from '../Bitboard';

describe('Bitboard Tests', () => {

  test('testPawnAttacksWhite', () => {
    const pawns =           fromBigInt(0b00000000_00000000_00000000_00000000_00001000_00000000_00000000_00000000n);
    const expectedAttacks = fromBigInt(0b00000000_00000000_00000000_00010100_00000000_00000000_00000000_00000000n);
    const pawnAttacks: BB64Long = Bitboard.pawnAttacks(pawns, Side.WHITE);
    expect(pawnAttacks.equals(expectedAttacks)).toBe(true); // Compare using equals method
  });

  test('testPawnAttacksBlack', () => {
    const pawns =           fromBigInt(0b00000000_00000000_00000000_01000001_00000000_00000000_00000000_00000000n);
    const expectedAttacks = fromBigInt(0b00000000_00000000_00000000_00000000_10100010_00000000_00000000_00000000n);
    const pawnAttacks = Bitboard.pawnAttacks(pawns, Side.BLACK);
    expect(pawnAttacks.equals(expectedAttacks)).toBe(true); // Use equals method
  });

  test('testGetKnightAttacks', () => {
    const knightPosition = 38;
    const expectedAttacks = fromBigInt(0b00000000_10100000_00010000_00000000_00010000_10100000_00000000_00000000n);
    const knightAttacks = Bitboard.getKnightAttacks(knightPosition);
    expect(knightAttacks.equals(expectedAttacks)).toBe(true);
  });

  test('testGetKingAttacks', () => {
    const kingPosition = 28;
    const expectedAttacks = fromBigInt(0b00000000_00000000_00000000_00111000_00101000_00111000_00000000_00000000n);
    const kingAttacks = Bitboard.getKingAttacks(kingPosition);
    expect(kingAttacks.equals(expectedAttacks)).toBe(true);
  });

  test('testGetRookAttacks', () => {
    const rookPosition = 29;
    const occupied =        fromBigInt(0b01010101_01110011_00001000_00101000_00100010_11010001_01010100_01110101n);
    const expectedAttacks = fromBigInt(0b00000000_00000000_00000000_00100000_11011110_00100000_00100000_00100000n);
    const rookAttacks = Bitboard.getRookAttacks(rookPosition, occupied);
    // console.info(bitboardToString(expectedAttacks));
    // console.info(bitboardToString(rookAttacks));
    expect(rookAttacks.equals(expectedAttacks)).toBe(true);
  });

  test('testGetRookAttacks2', () => {
    const rookPosition = 0;
    const occupied =        fromBigInt(0b11111101_11111111_00000001_00000000_00000000_00100000_11111111_10111111n);
    const expectedAttacks = fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000001_00000010n);
    const rookAttacks = Bitboard.getRookAttacks(rookPosition, occupied);
    // console.info(bitboardToFormattedBinary(occupied));
    // console.info(bitboardToString(rookAttacks));
    expect(rookAttacks.equals(expectedAttacks)).toBe(true);
  });

  test('testGetBishopAttacks', () => {
    const bishopPosition = 27;
    const occupied = fromBigInt(0b01010101_01110011_00001000_00101000_00100010_11010001_01010100_01110101n);
    const expectedAttacks = fromBigInt(0b00000000_01000001_00100010_00010100_00000000_00010100_00000010_00000001n);
    const bishopAttacks = Bitboard.getBishopAttacks(bishopPosition, occupied);
    expect(bishopAttacks.equals(expectedAttacks)).toBe(true);
  });

  test('testGetBishopAttacks2', () => {
    const bishopPosition = 2;
    const occupied = fromBigInt(0b00100000_10000001_00001010_00101110_00000100_00100001_00000000_10000000n);
    const expectedAttacks = stringToBitboard(
      `. . . . . . . . 
. . . . . . . . 
. . . . . . . X 
. . . . . . X . 
. . . . . X . . 
. . . . X . . . 
. X . X . . . . 
. . . . . . . .`) ;
    const bishopAttacks = Bitboard.getBishopAttacks(bishopPosition, occupied);
    // console.info(bitboardToString(bishopAttacks));
    expect(bitboardToString(bishopAttacks)).toEqual(bitboardToString(expectedAttacks));
  });

  test('attacks differ', () => {
    const expectedAttacks1 = stringToBitboard(
      `. . . . . . . . 
. . . . . . . . 
. . . . . . . X 
. . . . . . X . 
. . . . . X . . 
. . . . X . . . 
. X . X . . . . 
. . . . . . . .`) ;
    const expectedAttacks2 = stringToBitboard(
      `. . . . . . . . 
. . . . . . . . 
. . . . . . . . 
. . . . . . . . 
. . . . . X . . 
. . . . X . . . 
. X . X . . . . 
. . . . . . . .`) ;
    expect(expectedAttacks1).not.toEqual(expectedAttacks2);
  });

  test('testGetQueenAttacks', () => {
    const queenPosition = 28;
    const occupied = fromBigInt(0b01010101_01110011_00001000_00101000_00100010_11010001_01010100_01110101n);
    const expectedAttacks = fromBigInt(0b00000000_00010000_00010000_00111000_00101110_00111000_01000100_00000000n);

    const queenAttacks = Bitboard.getQueenAttacks(queenPosition, occupied);

    expect(queenAttacks.equals(expectedAttacks)).toBe(true);
  });

  // test('testBetweenSquares', () => {
  //   const squareA = 0;  // a1
  //   const squareB = 7;  // h1
  //   const expectedBetween = new BB64Long(
  //     0b01111110, 0b00000000_00000000_00000000_00000000
  //   );
  //   expect(Bitboard.between(squareA, squareB)).toEqual(expectedBetween);
  // });

  test('testPushWhite', () => {
    const pawn =         fromBigInt(0b00000000_00000000_00000000_00000000_00000100_00000000_00000000_00000000n);
    const expectedPush = fromBigInt(0b00000000_00000000_00000000_00000100_00000000_00000000_00000000_00000000n);
    const pushedPawn = Bitboard.pawnPush(pawn, Side.WHITE);
    expect(pushedPawn).toEqual(expectedPush);
  });

  test('testPushBlack', () => {
    const pawn =         fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000010_00000000n);
    const expectedPush = fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000010n);
    const pushedPawn = Bitboard.pawnPush(pawn, Side.BLACK);
    expect(pushedPawn).toEqual(expectedPush);
  });

  // test('testCastlingPiecesKingsideMask', () => {
  //   const expectedWhiteMask = WHITE_KING_SIDE_CASTLING_BIT_PATTERN;
  //   const expectedBlackMask = BLACK_KING_SIDE_CASTLING_BIT_PATTERN;
  //   expect(Bitboard.castlingPiecesKingsideMask(Side.WHITE)).toEqual(expectedWhiteMask);
  //   expect(Bitboard.castlingPiecesKingsideMask(Side.BLACK)).toEqual(expectedBlackMask);
  // });
  //
  // test('testCastlingPiecesQueensideMask', () => {
  //   const expectedWhiteMask = Bitboard.WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN;
  //   const expectedBlackMask = Bitboard.BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN;
  //   expect(Bitboard.castlingPiecesQueensideMask(Side.WHITE)).toEqual(expectedWhiteMask);
  //   expect(Bitboard.castlingPiecesQueensideMask(Side.BLACK)).toEqual(expectedBlackMask);
  // });
  //
  // test('testCastlingBlockersKingsideMask', () => {
  //   const expectedWhiteMask = Bitboard.WHITE_KING_SIDE_CASTLING_BLOCKERS_PATTERN;
  //   const expectedBlackMask = Bitboard.BLACK_KING_SIDE_CASTLING_BLOCKERS_PATTERN;
  //   expect(Bitboard.castlingBlockersKingsideMask(Side.WHITE)).toEqual(expectedWhiteMask);
  //   expect(Bitboard.castlingBlockersKingsideMask(Side.BLACK)).toEqual(expectedBlackMask);
  // });
  //
  // test('testCastlingBlockersQueensideMask', () => {
  //   const expectedWhiteMask = Bitboard.WHITE_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN;
  //   const expectedBlackMask = Bitboard.BLACK_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN;
  //   expect(Bitboard.castlingBlockersQueensideMask(Side.WHITE)).toEqual(expectedWhiteMask);
  //   expect(Bitboard.castlingBlockersQueensideMask(Side.BLACK)).toEqual(expectedBlackMask);
  // });
});

describe('getLineAttacks', () => {
  it('should correctly compute line attacks using obstruction difference', () => {
    // Given the occupancy and line attack mask
    const occupancy = fromBigInt(BigInt('6157274084021458037'));
    // console.info(bitboardToFormattedBinary(occupancy));
    const lineMask = new LineAttackMask(
      fromBigInt(BigInt('520093696')),
      fromBigInt(BigInt('3221225472')),
      fromBigInt(BigInt('3741319168'))
    );

    // Expected result as BigInt
    const expectedResult = BigInt(3724541952);

    // When we calculate the line attacks
    const result = getLineAttacks(occupancy, lineMask);

    // Then the result should match the expected value
    expect(result.asBigInt()).toBe(expectedResult);
  });
});

describe('getLineAttacks2', () => {
  it('should correctly compute line attacks using obstruction difference', () => {
    // Given the occupancy and line attack mask
    const occupancy = fromBigInt(BigInt('18302348510170775487'));
    // console.info(bitboardToFormattedBinary(occupancy));
    const lineMask = new LineAttackMask(
      fromBigInt(BigInt('0')),
      fromBigInt(BigInt('254')),
      fromBigInt(BigInt('254'))
    );

    // Expected result as BigInt
    const expectedResult = BigInt(2);

    // When we calculate the line attacks
    const result = getLineAttacks(occupancy, lineMask);

    // Then the result should match the expected value
    expect(result.asBigInt()).toBe(expectedResult);
  });
});
