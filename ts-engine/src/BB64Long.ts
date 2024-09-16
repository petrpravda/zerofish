// inspired by https://github.com/wlivengood/Winston/tree/master

// Utility Functions for 32-bit unsigned integer operations
export function U32(v: number): number {
  return v >>> 0;
}

// Count the number of set bits (population count) in a 32-bit integer
export function popcnt32(v: number): number {
  v = U32(v);
  v -= (v >>> 1) & 0x55555555;
  v = (v & 0x33333333) + ((v >>> 2) & 0x33333333);
  return (((v + (v >>> 4)) & 0xF0F0F0F) * 0x1010101) >>> 24;
}

// Clear the least significant bit (LSB)
export function popLSB32(v: number): number {
  v = U32(v);
  return U32(v & (v - 1));
}

// Get the index of the least significant bit
export function getLSB32(v: number): number {
  v = U32(v);
  return popcnt32((v & -v) - 1);
}

// Bitboard class for handling 64-bit board represented as two 32-bit integers
export class BB64Long {
  lower: number;
  upper: number;

  // constructor(lower: number, upper: number) {
  //   this.lower = U32(lower);
  //   this.upper = U32(upper);
  // }

  constructor(lower: number, upper: number);
  constructor(bblong: bigint);
  constructor(lowerOrBigInt: number | bigint, upper?: number) {
    if (typeof lowerOrBigInt === "bigint") {
      // Handle the case where the constructor is called with a bigint
      this.lower = U32(Number(lowerOrBigInt & BigInt(0xFFFFFFFF))); // Extract lower 32 bits
      this.upper = U32(Number((lowerOrBigInt >> BigInt(32)) & BigInt(0xFFFFFFFF))); // Extract upper 32 bits
    } else if (typeof upper === "number") {
      // Handle the case where the constructor is called with two 32-bit numbers
      this.lower = U32(lowerOrBigInt);
      this.upper = U32(upper);
    } else {
      throw new Error("Invalid constructor parameters");
    }
  }

  asBigInt(): bigint {
    return (BigInt(this.upper) << BigInt(32)) | BigInt(this.lower);
  }

  // Check if the bitboard is empty
  empty(): boolean {
    return !this.lower && !this.upper;
  }

  // Check if the bit at a specific index is zero
  isZero(idx: number): boolean {
    idx = U32(idx);
    return idx < 32 ? !(this.lower & (1 << idx)) : !(this.upper & (1 << (idx - 32)));
  }

  // Check if the bit at a specific index is one
  isOne(idx: number): boolean {
    return !this.isZero(idx);
  }

  // Set a bit at a specific index
  setBit(idx: number): this {
    idx = U32(idx);
    if (idx < 32) this.lower = U32(this.lower | (1 << idx));
    else this.upper = U32(this.upper | (1 << (idx - 32)));
    return this;
  }

  // Clear a bit at a specific index
  clearBit(idx: number): this {
    idx = U32(idx);
    if (idx < 32) this.lower = U32(this.lower & ~(1 << idx));
    else this.upper = U32(this.upper & ~(1 << (idx - 32)));
    return this;
  }

  // Count the number of set bits
  popcnt(): number {
    return popcnt32(this.lower) + popcnt32(this.upper);
  }

  // Clear the least significant bit
  popLSB(): this {
    if (this.lower) this.lower = popLSB32(this.lower);
    else this.upper = popLSB32(this.upper);
    return this;
  }

  // Get the index of the least significant bit
  LSB(): number {
    return this.lower ? getLSB32(this.lower) : 32 + getLSB32(this.upper);
  }

  // Clear the least significant bit and return its previous index
  popRetLSB(): number {
    const idx = this.LSB();
    this.popLSB();
    return idx;
  }

  // Perform a bitwise AND with another bitboard
  AND(other: BB64Long): this {
    this.lower = U32(this.lower & other.lower);
    this.upper = U32(this.upper & other.upper);
    return this;
  }

  // Perform a bitwise AND NOT with another bitboard
  AND_NOT(other: BB64Long): this {
    this.lower = U32(this.lower & ~other.lower);
    this.upper = U32(this.upper & ~other.upper);
    return this;
  }

  // Perform a bitwise OR with another bitboard
  OR(other: BB64Long): this {
    this.lower = U32(this.lower | other.lower);
    this.upper = U32(this.upper | other.upper);
    return this;
  }

  // Perform a bitwise XOR with another bitboard
  XOR(other: BB64Long): this {
    this.lower = U32(this.lower ^ other.lower);
    this.upper = U32(this.upper ^ other.upper);
    return this;
  }

  // Perform a bitwise NOT on the bitboard
  NOT(): this {
    this.lower = U32(~this.lower);
    this.upper = U32(~this.upper);
    return this;
  }

  // Shift left by a specified number of bits
  SHL(v: number): this {
    v = U32(v);
    if (v > 31) {
      this.upper = U32(this.lower << (v - 32));
      this.lower = U32(0);
    } else if (v > 0) {
      this.upper = U32((this.upper << v) | (this.lower >>> (32 - v)));
      this.lower = U32(this.lower << v);
    }
    return this;
  }

  // Shift right by a specified number of bits
  SHR(v: number): this {
    v = U32(v);
    if (v > 31) {
      this.lower = this.upper >>> (v - 32);
      this.upper = U32(0);
    } else if (v > 0) {
      this.lower = U32((this.lower >>> v) | (this.upper << (32 - v)));
      this.upper >>>= v;
    }
    return this;
  }

  // Shift left for positive values or right for negative values
  SHIFT(v: number): this {
    if (v > 63 || v < -63) {
      this.lower = this.upper = U32(0);
    } else if (v > 0) {
      this.SHL(v);
    } else if (v < 0) {
      this.SHR(-v);
    }
    return this;
  }

  // Check if two bitboards are equal
  equals(other: BB64Long): boolean {
    return this.lower === other.lower && this.upper === other.upper;
  }

  // Create a copy of the bitboard
  copy(): BB64Long {
    return new BB64Long(this.lower, this.upper);
  }
}

// Bitboard utility functions
export function makeBB(low: number, high: number): BB64Long {
  return new BB64Long(low, high);
}

export function zeroBB(): BB64Long {
  return makeBB(0, 0);
}

export function oneBB(): BB64Long {
  return makeBB(0xFFFFFFFF, 0xFFFFFFFF);
}

export function lightBB(): BB64Long {
  return makeBB(0x55AA55AA, 0x55AA55AA);
}

export function darkBB(): BB64Long {
  return makeBB(0xAA55AA55, 0xAA55AA55);
}

export function fileBB(file: number): BB64Long {
  return makeBB(0x01010101, 0x01010101).SHL(file);
}

export function fileBBs(): BB64Long[] {
  const b: BB64Long[] = [];
  for (let i = 0; i < 8; ++i) b.push(fileBB(i));
  return b;
}

export function rankBB(rank: number): BB64Long {
  return makeBB(0xFF, 0).SHL(rank * 8);
}

export function rankBBs(): BB64Long[] {
  const b: BB64Long[] = [];
  for (let i = 0; i < 8; ++i) b.push(rankBB(i));
  return b;
}

export function idxBB(index: number): BB64Long {
  return zeroBB().setBit(index);
}

export function diagBB(diagonal: number): BB64Long {
  return makeBB(0x10204080, 0x01020408).AND(oneBB().SHIFT(diagonal * 8)).SHIFT(diagonal);
}

export function diagBBs(): BB64Long[] {
  const b: BB64Long[] = [];
  for (let i = -7; i < 8; ++i) b.push(diagBB(i));
  return b;
}

export function antiDiagBB(antidiagonal: number): BB64Long {
  return makeBB(0x08040201, 0x80402010).AND(oneBB().SHIFT(-antidiagonal * 8)).SHIFT(antidiagonal);
}

export function antiDiagBBs(): BB64Long[] {
  const b: BB64Long[] = [];
  for (let i = -7; i < 8; ++i) b.push(antiDiagBB(i));
  return b;
}

export function bitboardToString(bb: BB64Long): string {
  const bigIntBoard = bb.asBigInt();
  let result = "";

  for (let rank = 56; rank >= 0; rank -= 8) {
    for (let file = 0; file < 8; file++) {
      const bitIndex = BigInt(rank + file);
      const bit = (bigIntBoard >> bitIndex) & BigInt(1);
      result += bit === BigInt(1) ? "X" : ".";
      result += " ";
    }
    result += "\n";
  }

  return result;
}

export function bitboardToFormattedBinary(bb: BB64Long): string {
  const bigIntBoard = bb.asBigInt();
  let binaryString = bigIntBoard.toString(2).padStart(64, '0');
  binaryString = binaryString.replace(/(.{8})(?=.)/g, "$1_");
  return `0b${binaryString}n`;
}
