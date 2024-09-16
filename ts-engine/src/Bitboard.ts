import {BB64Long, zeroBB} from './BB64Long';

export class Bitboard {


  static getKnightAttacks(knightPosition: number): BB64Long {
    return KNIGHT_ATTACKS[knightPosition];
  }

  static getKingAttacks(kingPosition: number): BB64Long {
    return KING_ATTACKS[kingPosition];
  }

  static getRookAttacks(rookPosition: number, occupied: BB64Long): BB64Long {
    return getLineAttacks(occupied, LINE_MASKS[DirectionUtils.maskIndex(Directions.Horizontal, rookPosition)])
      .OR(getLineAttacks(occupied, LINE_MASKS[DirectionUtils.maskIndex(Directions.Vertical, rookPosition)]));
  }
}

/** BB64Long operations: In the Java code, there were bitwise operations on long types like &, |, and shifts. These have been replaced with corresponding methods on BB64Long such as .AND(), .OR(), .SHL(), and .LSB().
 MS1B (most significant bit): The code finds the most significant bit of lower. Since BB64Long doesn't have a direct numberOfLeadingZeros equivalent, the solution is to reverse-engineer it by calculating the index of the most significant bit (via 63 - LSB()).
 LS1B (least significant bit): For this, the code uses the LSB() method of BB64Long to get the least significant bit and performs a shift and bitwise OR as per the logic in the original Java code.
 Return: Finally, the method returns the result of a bitwise AND between patterns.combined and the calculated diff.
  **/
export function getLineAttacks(occupied: BB64Long, patterns: LineAttackMask): BB64Long {
  // https://www.chessprogramming.org/Obstruction_Difference
  const lower = patterns.lower.copy().AND(occupied); // lower part of occupied & patterns.lower
  const upper = patterns.upper.copy().AND(occupied); // upper part of occupied & patterns.upper

  // Find the most significant bit (MS1B) in the lower part
  const mMs1b = new BB64Long(0, 0);
  if (!lower.empty()) {
    const ms1bIdx = 63 - lower.LSB(); // MS1B index (highest bit set in lower)
    mMs1b.setBit(ms1bIdx);
  }

  // Get least significant bit (LS1B) in the upper part
  const ls1b = upper.copy();
  if (!upper.empty()) {
    ls1b.clearBit(upper.LSB()); // ls1b = upper & -upper in terms of BB64Long logic
  }

  // Difference calculation
  const diff = ls1b.copy().SHL(1).OR(mMs1b); // 2 * ls1b + mMs1b

  // Return the masked attacks using the combined mask
  return patterns.combined.copy().AND(diff);
}

export class MoveDirection {
  constructor(public x: number, public y: number) {}

  static dir(x: number, y: number): MoveDirection {
    return new MoveDirection(x, y);
  }
}

export class LineAttackMask {
  constructor(
    public lower: BB64Long,
    public upper: BB64Long,
    public combined: BB64Long
  ) {}
}

export enum Directions {
  Horizontal = 0,
  Vertical = 1,
  Diagonal = 2,
  AntiDiagonal = 3
}

export class DirectionUtils {
  private static directionOffsets = {
    [Directions.Horizontal]: { col: -1, row: 0 },
    [Directions.Vertical]: { col: 0, row: -1 },
    [Directions.Diagonal]: { col: 1, row: -1 },
    [Directions.AntiDiagonal]: { col: -1, row: -1 }
  };

  static getOffset(direction: Directions): { col: number, row: number } {
    return this.directionOffsets[direction];
  }

  static maskIndex(direction: Directions, square: number): number {
    return direction * 64 + square;
  }
}

export class SquarePosition {
  constructor(public file: number, public rank: number) {}

  static fromSquareIndex(square: number): SquarePosition {
    return new SquarePosition(square % 8, Math.floor(square / 8));
  }

  toSquareIndex(): number {
    return this.file + this.rank * 8;
  }

  moveInDirection(direction: MoveDirection): SquarePosition {
    return new SquarePosition(this.file + direction.x, this.rank + direction.y);
  }

  isOnBoard(): boolean {
    return this.file >= 0 && this.file < 8 && this.rank >= 0 && this.rank < 8;
  }
}


function calculateLinePatterns(): LineAttackMask[] {
  return (Object.keys(Directions) as Array<keyof typeof Directions>) // Cast to enum keys
    .filter(key => !isNaN(Number(key))) // Filter out the string keys to keep only enum values
    .map(key => Number(key)) // Convert string keys to their numeric values
    .flatMap((dir: Directions) => // `dir` is now of type Directions
      Array.from({ length: 64 }, (_, square) => {
        const { col, row } = DirectionUtils.getOffset(dir);
        const lower = generateRay(square, col, row);
        const upper = generateRay(square, -col, -row);
        const combined = new BB64Long(lower.lower | upper.lower, lower.upper | upper.upper);
        return new LineAttackMask(lower, upper, combined);
      })
    );
}

function generateRay(pos: number, directionHorizontal: number, directionVertical: number): BB64Long {
  let file = pos % 8;
  let rank = Math.floor(pos / 8);
  let pattern = new BB64Long(0, 0);

  for (let i = 0; i < 7; i++) {  // max steps
    file += directionHorizontal;
    rank += directionVertical;

    if (file < 0 || file > 7 || rank < 0 || rank > 7) {
      break;
    }

    const bitIndex = rank * 8 + file;
    pattern.setBit(bitIndex);
  }

  return pattern;
}

function generateAttacks(moveDirections: MoveDirection[]): BB64Long[] {
  return Array.from({ length: 64 }, (_, index) => SquarePosition.fromSquareIndex(index))
    .map(position => {
      return moveDirections
        .map(direction => position.moveInDirection(direction))
        .filter(pos => pos.isOnBoard())
        .reduce((acc, pos) => acc.OR(new BB64Long(BigInt(1) << BigInt(pos.toSquareIndex()))), zeroBB());
    });
}


const KNIGHT_MOVE_DIRECTIONS: MoveDirection[] = [
  MoveDirection.dir(-2, -1), MoveDirection.dir(-2, 1),
  MoveDirection.dir(2, -1), MoveDirection.dir(2, 1),
  MoveDirection.dir(-1, -2), MoveDirection.dir(-1, 2),
  MoveDirection.dir(1, -2), MoveDirection.dir(1, 2)
];

const KING_MOVE_DIRECTIONS: MoveDirection[] = [
  MoveDirection.dir(0, -1), MoveDirection.dir(1, -1),
  MoveDirection.dir(1, 0), MoveDirection.dir(1, 1),
  MoveDirection.dir(0, 1), MoveDirection.dir(-1, 1),
  MoveDirection.dir(-1, 0), MoveDirection.dir(-1, -1)
];

const LINE_MASKS: LineAttackMask[] = calculateLinePatterns();

const KNIGHT_ATTACKS: BB64Long[] = generateAttacks(KNIGHT_MOVE_DIRECTIONS);
const KING_ATTACKS: BB64Long[] = generateAttacks(KING_MOVE_DIRECTIONS);
