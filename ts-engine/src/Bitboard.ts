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

  static getBishopAttacks(bishopPosition: number, occupied: BB64Long): BB64Long {
    return getLineAttacks(occupied, LINE_MASKS[DirectionUtils.maskIndex(Directions.Diagonal, bishopPosition)])
      .OR(getLineAttacks(occupied, LINE_MASKS[DirectionUtils.maskIndex(Directions.AntiDiagonal, bishopPosition)]));
  }

  static getQueenAttacks(queenPosition: number, occupied: BB64Long): BB64Long {
    const rookAttacks = Bitboard.getRookAttacks(queenPosition, occupied);
    const bishopAttacks = Bitboard.getBishopAttacks(queenPosition, occupied);

    return rookAttacks.OR(bishopAttacks);
  }
}

export function getLineAttacks(occupied: BB64Long, patterns: LineAttackMask): BB64Long {
  const lower = patterns.lower.copy().AND(occupied); // lower part of occupied & patterns.lower
  // console.info(bitboardToFormattedBinary(occupied));
  // console.info(bitboardToFormattedBinary(patterns.lower));
  // console.info(bitboardToFormattedBinary(lower));
  const upper = patterns.upper.copy().AND(occupied); // upper part of occupied & patterns.upper

  lower.maskMostSignificantBit().subtract1().NOT().AND(patterns.lower);
  // console.info(bitboardToFormattedBinary(lower));
  // console.info(bitboardToFormattedBinary(patterns.upper));
  // console.info(bitboardToFormattedBinary(upper));
  upper.subtract1().SHL(1).AND(patterns.upper);

  return lower.OR(upper);
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

  toString(): string {
    return `LineAttackMask[lower=${this.lower.asBigInt()}, upper=${this.upper.asBigInt()}, combined=${this.combined.asBigInt()}]`;
  }
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

//console.info(LINE_MASKS.map(mask => mask.toString()));
// LINE_MASKS.forEach(mask => console.info(mask.toString()));

