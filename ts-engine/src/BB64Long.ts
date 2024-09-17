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

// Get the index of the most significant bit in a 32-bit unsigned integer
export function getMSB32(v: number): number {
  v = U32(v);

  if (v === 0) return -1; // Return -1 if no bits are set

  // 31 - number of leading zeros gives us the index of the MSB
  return 31 - Math.clz32(v);
}


// Bitboard class for handling 64-bit board represented as two 32-bit integers
export class BB64Long {
  readonly lower: number;
  readonly upper: number;
  // lower: number;
  // upper: number;

  constructor(lower: number, upper: number) {
    this.lower = U32(lower);
    this.upper = U32(upper);
  }

  asBigInt(): bigint {
    return (BigInt(this.upper) << BigInt(32)) | BigInt(this.lower);
  }

  empty(): boolean {
    return !this.lower && !this.upper;
  }

  isZero(idx: number): boolean {
    idx = U32(idx);
    return idx < 32 ? !(this.lower & (1 << idx)) : !(this.upper & (1 << (idx - 32)));
  }

  isOne(idx: number): boolean {
    return !this.isZero(idx);
  }

  // Set a bit at a specific index
  // setBit(idx: number): this {
  //   idx = U32(idx);
  //   if (idx < 32) this.lower = U32(this.lower | (1 << idx));
  //   else this.upper = U32(this.upper | (1 << (idx - 32)));
  //   return this;
  // }
  // Bypassing readonly with Object.defineProperty
  setBit(idx: number): this {
    idx = U32(idx);

    if (idx < 32) {
      // Bypass readonly and update the 'lower' value
      Object.defineProperty(this, 'lower', {
        value: U32(this.lower | (1 << idx)),
        writable: true,  // Optional if you want to make it writable again
        configurable: true,  // Allows redefining the property again later if needed
      });
    } else {
      // Bypass readonly and update the 'upper' value
      Object.defineProperty(this, 'upper', {
        value: U32(this.upper | (1 << (idx - 32))),
        writable: true,
        configurable: true,
      });
    }

    return this;
  }
  // // Clear a bit at a specific index
  // clearBit(idx: number): this {
  //   idx = U32(idx);
  //   if (idx < 32) this.lower = U32(this.lower & ~(1 << idx));
  //   else this.upper = U32(this.upper & ~(1 << (idx - 32)));
  //   return this;
  // }

  // Count the number of set bits
  popcnt(): number {
    return popcnt32(this.lower) + popcnt32(this.upper);
  }

  popLSB(): BB64Long {
    let newLower = this.lower;
    let newUpper = this.upper;

    if (newLower) {
      newLower = popLSB32(newLower);
    } else {
      newUpper = popLSB32(newUpper);
    }

    return new BB64Long(newLower, newUpper);
  }

  maskMostSignificantBit(): BB64Long {
    let newLower = this.lower;
    let newUpper = this.upper;

    if (newUpper !== 0) {
      const msbIndex = 31 - Math.clz32(newUpper);
      newUpper = U32(1 << msbIndex);
      newLower = U32(0);
    } else if (newLower !== 0) {
      const msbIndex = 31 - Math.clz32(newLower);
      newLower = U32(1 << msbIndex);
    }

    return new BB64Long(newLower, newUpper);
  }

  LSB(): number {
    return this.lower ? getLSB32(this.lower) : 32 + getLSB32(this.upper);
  }

  // MSB(): number {
  //   //return this.lower ? getLSB32(this.lower) : 32 + getLSB32(this.upper);
  //   return 333; // TODO TBD
  // }

  // countLeadingZeros64(): number {
  //   if (this.upper === 0) {
  //     return 32 + Math.clz32(this.lower);
  //   } else {
  //     return Math.clz32(this.upper);
  //   }
  // }


// // Clear the least significant bit and return its previous index
//   popRetLSB(): { index: number; board: BB64Long } {
//     const idx = this.LSB();
//     const updatedBoard = this.popLSB();
//     return { index: idx, board: updatedBoard };
//   }

// Perform a bitwise AND with another bitboard
  AND(other: BB64Long): BB64Long {
    return new BB64Long(U32(this.lower & other.lower), U32(this.upper & other.upper));
  }

// Perform a bitwise AND NOT with another bitboard
  AND_NOT(other: BB64Long): BB64Long {
    return new BB64Long(U32(this.lower & ~other.lower), U32(this.upper & ~other.upper));
  }

// Perform a bitwise OR with another bitboard
  OR(other: BB64Long): BB64Long {
    return new BB64Long(U32(this.lower | other.lower), U32(this.upper | other.upper));
  }

// Perform a bitwise XOR with another bitboard
  XOR(other: BB64Long): BB64Long {
    return new BB64Long(U32(this.lower ^ other.lower), U32(this.upper ^ other.upper));
  }

// Perform a bitwise NOT on the bitboard
  NOT(): BB64Long {
    return new BB64Long(U32(~this.lower), U32(~this.upper));
  }

// Shift left by a specified number of bits
  SHL(v: number): BB64Long {
    v = U32(v);
    let newLower, newUpper;

    if (v > 31) {
      newUpper = U32(this.lower << (v - 32));
      newLower = U32(0);
    } else if (v > 0) {
      newUpper = U32((this.upper << v) | (this.lower >>> (32 - v)));
      newLower = U32(this.lower << v);
    } else {
      newUpper = this.upper;
      newLower = this.lower;
    }

    return new BB64Long(newLower, newUpper);
  }

// Shift right by a specified number of bits
  SHR(v: number): BB64Long {
    v = U32(v);
    let newLower, newUpper;

    if (v > 31) {
      newLower = this.upper >>> (v - 32);
      newUpper = U32(0);
    } else if (v > 0) {
      newLower = U32((this.lower >>> v) | (this.upper << (32 - v)));
      newUpper = this.upper >>> v;
    } else {
      newLower = this.lower;
      newUpper = this.upper;
    }

    return new BB64Long(newLower, newUpper);
  }

// Shift left for positive values or right for negative values
  SHIFT(v: number): BB64Long {
    if (v > 63 || v < -63) {
      return new BB64Long(U32(0), U32(0));
    } else if (v > 0) {
      return this.SHL(v);
    } else if (v < 0) {
      return this.SHR(-v);
    }
    return this; // No shift needed
  }

  subtract1(): BB64Long {
    let newLower = this.lower;
    let newUpper = this.upper;

    // Check if the lower part will underflow when subtracting 1
    if (this.lower === 0) {
      // If the lower part is 0, it will underflow, so set it to 0xFFFFFFFF (wrap around)
      // and subtract 1 from the upper part
      newLower = U32(0xFFFFFFFF);
      newUpper = U32(this.upper - 1);
    } else {
      // If the lower part won't underflow, simply subtract 1 from the lower part
      newLower = U32(this.lower - 1);
    }

    // Return a new instance of BB64Long without mutating the original object
    return new BB64Long(newLower, newUpper);
  }


// Check if two bitboards are equal
  equals(other: BB64Long): boolean {
    return this.lower === other.lower && this.upper === other.upper;
  }
}

// Utility function to create a BB64Long from a bigint
export function fromBigInt(bblong: bigint): BB64Long {
  const lower = U32(Number(bblong & BigInt(0xFFFFFFFF))); // Extract lower 32 bits
  const upper = U32(Number((bblong >> BigInt(32)) & BigInt(0xFFFFFFFF))); // Extract upper 32 bits
  return new BB64Long(lower, upper);
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

// export function idxBB(index: number): BB64Long {
//   return zeroBB().setBit(index);
// }

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
