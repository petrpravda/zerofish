import {BB64Long, BB_ONE, bitboardToString, makeBB, stringToBitboard, U32} from '../BB64Long';

describe('Bitboard', () => {
  test('constructor should initialize correctly', () => {
    const bb = new BB64Long(0xFFFFFFFF, 0x00000000);
    expect(bb.lower).toBe(U32(0xFFFFFFFF));
    expect(bb.upper).toBe(U32(0x00000000));
  });

  test('empty should return true for an empty bitboard', () => {
    const bb = new BB64Long(0, 0);
    expect(bb.empty()).toBe(true);
  });

  test('empty should return false for a non-empty bitboard', () => {
    const bb = new BB64Long(1, 0);
    expect(bb.empty()).toBe(false);
  });

  test('isZero should return true for zero bits', () => {
    const bb = new BB64Long(0, 0);
    expect(bb.isZero(0)).toBe(true);
    expect(bb.isZero(32)).toBe(true);
  });

  test('isZero should return false for set bits', () => {
    const bb = new BB64Long(1, 0);
    expect(bb.isZero(0)).toBe(false);
  });

  test('isOne should return true for set bits', () => {
    const bb = new BB64Long(1, 0);
    expect(bb.isOne(0)).toBe(true);
  });

  // test('setBit should set the correct bit', () => {
  //   const bb = new BB64Long(0, 0);
  //   bb.setBit(0);
  //   expect(bb.lower).toBe(1);
  //   bb.setBit(32);
  //   expect(bb.upper).toBe(1);
  // });
  //
  // test('clearBit should clear the correct bit', () => {
  //   const bb = new BB64Long(1, 1);
  //   bb.clearBit(0);
  //   expect(bb.lower).toBe(0);
  //   bb.clearBit(32);
  //   expect(bb.upper).toBe(0);
  // });

  test('popcnt should count the number of set bits', () => {
    const bb = new BB64Long(0xFFFFFFFF, 0xFFFFFFFF);
    expect(bb.popcnt()).toBe(64);
    const bb2 = new BB64Long(0xF0F0F0F0, 0x0F0F0F0F);
    expect(bb2.popcnt()).toBe(32);
  });


  test('popcnt should count the number of set bits', () => {
    const bb = new BB64Long(0xFFFFFFFF, 0xFFFFFFFF);
    expect(bb.popcnt()).toBe(64);
    const bb2 = new BB64Long(0xF0F0F0F0, 0x0F0F0F0F);
    expect(bb2.popcnt()).toBe(32);
  });

  test('popLSB should return a new bitboard with the least significant bit cleared', () => {
    const bb = new BB64Long(0b101, 0);
    const newBB = bb.popLSB();
    expect(newBB.lower).toBe(0b100);
    expect(bb.lower).toBe(0b101);  // Original remains unchanged
    const newBB2 = newBB.popLSB();
    expect(newBB2.lower).toBe(0);
    expect(newBB.lower).toBe(0b100);  // Original remains unchanged
  });

  test('LSB should return the index of the least significant bit', () => {
    const bb = new BB64Long(0b101, 0);
    expect(bb.LSB()).toBe(0);
    const newBB = bb.popLSB();
    expect(newBB.LSB()).toBe(2);
  });

  // test('popRetLSB should return the index and a new bitboard with LSB cleared', () => {
  //   const bb = new BB64Long(0b101, 0);
  //   const [index, newBB] = bb.popRetLSB();
  //   expect(index).toBe(0);
  //   const [index2, newBB2] = newBB.popRetLSB();
  //   expect(index2).toBe(2);
  // });

  test('AND should return a new bitboard with a bitwise AND result', () => {
    const bb1 = new BB64Long(0b1100, 0);
    const bb2 = new BB64Long(0b1010, 0);
    const newBB = bb1.AND(bb2);
    expect(newBB.lower).toBe(0b1000);
    expect(bb1.lower).toBe(0b1100);  // Original remains unchanged
  });

  test('AND_NOT should return a new bitboard with a bitwise AND NOT result', () => {
    const bb1 = new BB64Long(0b1100, 0);
    const bb2 = new BB64Long(0b1010, 0);
    const newBB = bb1.AND_NOT(bb2);
    expect(newBB.lower).toBe(0b0100);
    expect(bb1.lower).toBe(0b1100);  // Original remains unchanged
  });

  test('OR should return a new bitboard with a bitwise OR result', () => {
    const bb1 = new BB64Long(0b1100, 0);
    const bb2 = new BB64Long(0b1010, 0);
    const newBB = bb1.OR(bb2);
    expect(newBB.lower).toBe(0b1110);
    expect(bb1.lower).toBe(0b1100);  // Original remains unchanged
  });

  test('XOR should return a new bitboard with a bitwise XOR result', () => {
    const bb1 = new BB64Long(0b1100, 0);
    const bb2 = new BB64Long(0b1010, 0);
    const newBB = bb1.XOR(bb2);
    expect(newBB.lower).toBe(0b0110);
    expect(bb1.lower).toBe(0b1100);  // Original remains unchanged
  });

  test('NOT should return a new bitboard with a bitwise NOT result', () => {
    const bb = new BB64Long(0xFFFFFFFF, 0xFFFFFFFF);
    const newBB = bb.NOT();
    expect(newBB.lower).toBe(0);
    expect(newBB.upper).toBe(0);
    expect(bb.lower).toBe(0xFFFFFFFF);  // Original remains unchanged
    expect(bb.upper).toBe(0xFFFFFFFF);  // Original remains unchanged
  });

  test('SHL should return a new bitboard with bits shifted left', () => {
    const bb = new BB64Long(0b1, 0);
    const newBB = bb.SHL(1);
    expect(newBB.lower).toBe(0b10);
    expect(bb.lower).toBe(0b1);  // Original remains unchanged
  });

  test('SHR should return a new bitboard with bits shifted right', () => {
    const bb = new BB64Long(0b10, 0);
    const newBB = bb.SHR(1);
    expect(newBB.lower).toBe(0b1);
    expect(bb.lower).toBe(0b10);  // Original remains unchanged
  });

  test('SHIFT should return a new bitboard shifted correctly for positive and negative values', () => {
    const bb = new BB64Long(0b1, 0);
    const newBB = bb.SHIFT(1);
    expect(newBB.lower).toBe(0b10);
    expect(bb.lower).toBe(0b1);  // Original remains unchanged

    const newBB2 = newBB.SHIFT(-1);
    expect(newBB2.lower).toBe(0b1);
    expect(newBB.lower).toBe(0b10);  // Still unchanged
  });

  test('equals should return true for equal bitboards', () => {
    const bb1 = new BB64Long(0xFFFFFFFF, 0xFFFFFFFF);
    const bb2 = new BB64Long(0xFFFFFFFF, 0xFFFFFFFF);
    expect(bb1.equals(bb2)).toBe(true);
  });

  test('equals should return false for different bitboards', () => {
    const bb1 = new BB64Long(0xFFFFFFFF, 0xFFFFFFFF);
    const bb2 = new BB64Long(0x00000000, 0xFFFFFFFF);
    expect(bb1.equals(bb2)).toBe(false);
  });

  describe('bitboardToString', () => {
    it('should convert an empty bitboard to the correct string representation', () => {
      const bb = new BB64Long(0, 0);
      const expectedString =
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n";

      expect(bitboardToString(bb)).toBe(expectedString);
    });

    it('should convert a bitboard with some bits set to the correct string representation', () => {
      //console.info(bitboardToString(BB_ONE));

      // Set bits 0 and 63 (corner positions)
      const bb = makeBB(1, 0x80000000);
      const expectedString =
        ". . . . . . . X \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        "X . . . . . . . \n";

      expect(bitboardToString(bb)).toBe(expectedString);
    });
  });

  describe('stringToBitboard', () => {
    it('should convert a string representation to an empty bitboard', () => {
      const bbString =
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n";

      const expectedBB = new BB64Long(0, 0);
      expect(stringToBitboard(bbString)).toEqual(expectedBB);
    });

    it('should convert a string with some bits set to the correct bitboard', () => {
      const bbString =
        ". . . . . . . X \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        "X . . . . . . . \n";

      const expectedBB = makeBB(1, 0x80000000);
      expect(bbString).toEqual(bitboardToString(expectedBB));
    });

    it('should throw an error for invalid string input (wrong length)', () => {
      const invalidBBString = ". . . ."; // Too short to be a valid bitboard
      expect(() => stringToBitboard(invalidBBString)).toThrow("Invalid bitboard string length. Expected 64 characters.");
    });

    it('should throw an error for invalid characters', () => {
      const invalidBBString =
        "X . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . . \n" +
        ". . . . . . . A \n"; // Invalid character 'A'

      expect(() => stringToBitboard(invalidBBString)).toThrow("Invalid character in bitboard string. Only 'X' and '.' are allowed.");
    });
  });

  describe('Round-trip conversion', () => {
    it('should maintain bitboard consistency when converting between string and bitboard', () => {
      // Original bitboard
      const originalBB = makeBB(0xFFFFFFF0, 0xFFFFFFFF);

      // Convert bitboard to string and back to bitboard
      const bbString = bitboardToString(originalBB);
      const convertedBB = stringToBitboard(bbString);

      // The final bitboard should be equal to the original bitboard
      expect(bitboardToString(convertedBB)).toEqual(bitboardToString(originalBB));
    });
  });
});
