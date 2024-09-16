import {BB64Long, U32} from '../BB64Long';

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

  test('setBit should set the correct bit', () => {
    const bb = new BB64Long(0, 0);
    bb.setBit(0);
    expect(bb.lower).toBe(1);
    bb.setBit(32);
    expect(bb.upper).toBe(1);
  });

  test('clearBit should clear the correct bit', () => {
    const bb = new BB64Long(1, 1);
    bb.clearBit(0);
    expect(bb.lower).toBe(0);
    bb.clearBit(32);
    expect(bb.upper).toBe(0);
  });

  test('popcnt should count the number of set bits', () => {
    const bb = new BB64Long(0xFFFFFFFF, 0xFFFFFFFF);
    expect(bb.popcnt()).toBe(64);
    const bb2 = new BB64Long(0xF0F0F0F0, 0x0F0F0F0F);
    expect(bb2.popcnt()).toBe(32);
  });

  test('popLSB should clear the least significant bit', () => {
    const bb = new BB64Long(0b101, 0);
    bb.popLSB();
    expect(bb.lower).toBe(0b100);
    bb.popLSB();
    expect(bb.lower).toBe(0);
  });

  test('LSB should return the index of the least significant bit', () => {
    const bb = new BB64Long(0b101, 0);
    expect(bb.LSB()).toBe(0);
    bb.popLSB();
    expect(bb.LSB()).toBe(2);
  });

  test('popRetLSB should return the index and clear the LSB', () => {
    const bb = new BB64Long(0b101, 0);
    expect(bb.popRetLSB()).toBe(0);
    expect(bb.popRetLSB()).toBe(2);
  });

  test('AND should perform a bitwise AND', () => {
    const bb1 = new BB64Long(0b1100, 0);
    const bb2 = new BB64Long(0b1010, 0);
    bb1.AND(bb2);
    expect(bb1.lower).toBe(0b1000);
  });

  test('AND_NOT should perform a bitwise AND NOT', () => {
    const bb1 = new BB64Long(0b1100, 0);
    const bb2 = new BB64Long(0b1010, 0);
    bb1.AND_NOT(bb2);
    expect(bb1.lower).toBe(0b0100);
  });

  test('OR should perform a bitwise OR', () => {
    const bb1 = new BB64Long(0b1100, 0);
    const bb2 = new BB64Long(0b1010, 0);
    bb1.OR(bb2);
    expect(bb1.lower).toBe(0b1110);
  });

  test('XOR should perform a bitwise XOR', () => {
    const bb1 = new BB64Long(0b1100, 0);
    const bb2 = new BB64Long(0b1010, 0);
    bb1.XOR(bb2);
    expect(bb1.lower).toBe(0b0110);
  });

  test('NOT should perform a bitwise NOT', () => {
    const bb = new BB64Long(0xFFFFFFFF, 0xFFFFFFFF);
    bb.NOT();
    expect(bb.lower).toBe(0);
    expect(bb.upper).toBe(0);
  });

  test('SHL should shift left', () => {
    const bb = new BB64Long(0b1, 0);
    bb.SHL(1);
    expect(bb.lower).toBe(0b10);
  });

  test('SHR should shift right', () => {
    const bb = new BB64Long(0b10, 0);
    bb.SHR(1);
    expect(bb.lower).toBe(0b1);
  });

  test('SHIFT should shift correctly for positive and negative values', () => {
    const bb = new BB64Long(0b1, 0);
    bb.SHIFT(1);
    expect(bb.lower).toBe(0b10);

    bb.SHIFT(-1);
    expect(bb.lower).toBe(0b1);
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

  test('copy should create a copy of the bitboard', () => {
    const bb1 = new BB64Long(0xFFFFFFFF, 0xFFFFFFFF);
    const bb2 = bb1.copy();
    expect(bb1.equals(bb2)).toBe(true);
    bb2.clearBit(0);
    expect(bb1.equals(bb2)).toBe(false);
  });
});
