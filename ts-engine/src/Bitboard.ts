import {BB64Long, BB_ZERO, fromBigInt, idxBB, zeroBB} from './BB64Long';
import {Side, SideType} from './Side';
import {Square} from './Square';
import {PieceType} from './PieceType';

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


  static pawnAttacks(pawns: BB64Long, side: SideType): BB64Long {
    if (side === Side.WHITE) {
      // Left attack shift by 7 and right attack shift by 9
      const leftAttacks = pawns.AND(LEFT_PAWN_ATTACK_MASK).SHL(7);
      const rightAttacks = pawns.AND(RIGHT_PAWN_ATTACK_MASK).SHL(9);
      return leftAttacks.OR(rightAttacks);
    } else {
      // For black, shift by 9 to the right and by 7 to the right
      const leftAttacks = pawns.AND(LEFT_PAWN_ATTACK_MASK).SHR(9);
      const rightAttacks = pawns.AND(RIGHT_PAWN_ATTACK_MASK).SHR(7);
      return leftAttacks.OR(rightAttacks);
    }
  }

  static pawnPush(pawn: BB64Long, side: SideType): BB64Long {
    return side == Side.WHITE ? pawn.SHL(8) : pawn.SHR(8);
  }

  static castlingPiecesKingsideMask(side: SideType): BB64Long {
    return side == Side.WHITE ? WHITE_KING_SIDE_CASTLING_BIT_PATTERN : BLACK_KING_SIDE_CASTLING_BIT_PATTERN;
  }

  static castlingPiecesQueensideMask(side: SideType): BB64Long {
    return side == Side.WHITE ? WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN : BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN;
  }

  static pawnAttacksFromSquare(square: number, side: number): BB64Long {
    const bb = zeroBB();
    bb.setBit(square);

    if (side === Side.WHITE) {
      return Bitboard.whiteLeftPawnAttacks(bb).OR(Bitboard.whiteRightPawnAttacks(bb));
    } else {
      return Bitboard.blackLeftPawnAttacks(bb).OR(Bitboard.blackRightPawnAttacks(bb));
    }
  }

  static whiteLeftPawnAttacks(pawns: BB64Long): BB64Long {
    return pawns.AND(LEFT_PAWN_ATTACK_MASK).SHL(7);
  }

  static whiteRightPawnAttacks(pawns: BB64Long): BB64Long {
    return pawns.AND(RIGHT_PAWN_ATTACK_MASK).SHL(9);
  }

  static blackLeftPawnAttacks(pawns: BB64Long): BB64Long {
    return pawns.AND(LEFT_PAWN_ATTACK_MASK).SHR(9);
  }

  static blackRightPawnAttacks(pawns: BB64Long): BB64Long {
    return pawns.AND(RIGHT_PAWN_ATTACK_MASK).SHR(7);
  }

  static castlingBlockersKingsideMask(side: SideType): BB64Long {
    return side === Side.WHITE ? WHITE_KING_SIDE_CASTLING_BLOCKERS_PATTERN :
      BLACK_KING_SIDE_CASTLING_BLOCKERS_PATTERN;
  }

  static castlingBlockersQueensideMask(side: SideType): BB64Long {
    return side === Side.WHITE ? WHITE_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN :
      BLACK_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN;
  }

  static ignoreOOODanger(side: SideType): BB64Long {
    return side === Side.WHITE ? OOO_DANGER_WHITE : OOO_DANGER_BLACK;
  }

  static between(sq1: number, sq2: number): BB64Long {
    return BB_SQUARES_BETWEEN[sq1][sq2];
  }

  static attacks(pieceType: number, square: number, occ: BB64Long): BB64Long {
    switch (pieceType) {
      case PieceType.ROOK:
        return Bitboard.getRookAttacks(square, occ);
      case PieceType.BISHOP:
        return Bitboard.getBishopAttacks(square, occ);
      case PieceType.QUEEN:
        return Bitboard.getBishopAttacks(square, occ).OR(Bitboard.getRookAttacks(square, occ));
      case PieceType.KING:
        return Bitboard.getKingAttacks(square);
      case PieceType.KNIGHT:
        return Bitboard.getKnightAttacks(square);
      default:
        return BB_ZERO;
    }
  }

  static line(sq1: number, sq2: number): BB64Long {
    return BB_LINES[sq1][sq2];
  }
}

const OOO_DANGER_WHITE = fromBigInt(0x2n);
const OOO_DANGER_BLACK = fromBigInt(0x200000000000000n);

// Constants converted to BigInt using `fromBigInt` (assuming fromBigInt handles the conversion)
export const LEFT_PAWN_ATTACK_MASK = fromBigInt(0b11111110_11111110_11111110_11111110_11111110_11111110_11111110_11111110n);
export const RIGHT_PAWN_ATTACK_MASK = fromBigInt(0b1111111_01111111_01111111_01111111_01111111_01111111_01111111_01111111n);

// Exporting arrays of BigInt for patterns like PAWN_DOUBLE_PUSH_LINES and PAWN_RANKS
export const PAWN_DOUBLE_PUSH_LINES = [
  fromBigInt(0b00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000n),
  fromBigInt(0b00000000_00000000_11111111_00000000_00000000_00000000_00000000_00000000n),
];

export const PAWN_RANKS = [
  fromBigInt(0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000n),
  fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000n),
];

export const PAWN_FINAL_RANKS = fromBigInt(0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_11111111n);

// Castling blockers pattern constants
export const BLACK_KING_SIDE_CASTLING_BLOCKERS_PATTERN = fromBigInt(0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000n);
export const BLACK_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN = fromBigInt(0b00001110_00000000_00000000_00000000_00000000_00000000_00000000_00000000n);
export const WHITE_KING_SIDE_CASTLING_BLOCKERS_PATTERN = fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000n);
export const WHITE_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN = fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001110n);

// Castling bit patterns
export const BLACK_KING_SIDE_CASTLING_BIT_PATTERN = fromBigInt(0b10010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000n);
export const BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN = fromBigInt(0b00010001_00000000_00000000_00000000_00000000_00000000_00000000_00000000n);
export const WHITE_KING_SIDE_CASTLING_BIT_PATTERN = fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10010000n);
export const WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN = fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010001n);

// Rook masks
export const WHITE_KINGS_ROOK_MASK = fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000000n);
export const WHITE_QUEENS_ROOK_MASK = fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001n);
export const BLACK_QUEENS_ROOK_MASK = fromBigInt(0b00000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000n);
export const BLACK_KINGS_ROOK_MASK = fromBigInt(0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000n);

// Function to calculate the position of the most significant bit (MSB)
function MSB(bigIntValue: BigInt): number {
  // You would implement the MSB calculation here (or use a library function)
  // This is a placeholder function that should return the position of the highest bit set.
  return 0; // Implement actual logic based on your requirements
}

// Initial king square values calculated using MSB
export const WHITE_KING_INITIAL_SQUARE = fromBigInt(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000n).LSB();
export const BLACK_KING_INITIAL_SQUARE = fromBigInt(0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000n).LSB();


export function getLineAttacks(occupied: BB64Long, patterns: LineAttackMask): BB64Long {
  const lower = patterns.lower.AND(occupied); // lower part of occupied & patterns.lower
  // console.info(bitboardToFormattedBinary(occupied));
  // console.info(bitboardToFormattedBinary(patterns.lower));
  // console.info(bitboardToFormattedBinary(lower));
  const upper = patterns.upper.AND(occupied); // upper part of occupied & patterns.upper

  const lowerSlide = lower.maskMostSignificantBit().subtract1().NOT().AND(patterns.lower);
  // console.info(bitboardToFormattedBinary(lower));
  // console.info(bitboardToFormattedBinary(patterns.upper));
  // console.info(bitboardToFormattedBinary(upper));
  const upperSlide = upper.maskLeastSignificantBit().subtract1().SHL(1).AND(patterns.upper);

  return lowerSlide.OR(upperSlide);
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

function generateSquaresBetween(): BB64Long[][] {
  const squaresBetween: BB64Long[][] = Array.from({ length: 64 }, () => Array(64).fill(0n));

  for (let sq1 = Square.A1; sq1 <= Square.H8; sq1++) {
    for (let sq2 = Square.A1; sq2 <= Square.H8; sq2++) {
      const sqs = idxBB(sq1).OR(idxBB(sq2));

      if (Square.getFileIndex(sq1) === Square.getFileIndex(sq2) || Square.getRankIndex(sq1) === Square.getRankIndex(sq2)) {
        squaresBetween[sq1][sq2] =
          Bitboard.getRookAttacks(sq1, sqs).AND(Bitboard.getRookAttacks(sq2, sqs));
      } else if (Square.getDiagonalIndex(sq1) === Square.getDiagonalIndex(sq2) || Square.getAntiDiagonalIndex(sq1) === Square.getAntiDiagonalIndex(sq2)) {
        squaresBetween[sq1][sq2] =
          Bitboard.getBishopAttacks(sq1, sqs).AND(Bitboard.getBishopAttacks(sq2, sqs));
      }
    }
  }

  return squaresBetween;
}

function createBBLines(): BB64Long[][] {
  const bbLines: BB64Long[][] = Array.from({ length: 64 }, () => Array(64).fill(BB_ZERO));

  for (let sq1 = Square.A1; sq1 <= Square.H8; sq1++) {
    for (let sq2 = Square.A1; sq2 <= Square.H8; sq2++) {
      if (Square.getFileIndex(sq1) === Square.getFileIndex(sq2) || Square.getRankIndex(sq1) === Square.getRankIndex(sq2)) {
        bbLines[sq1][sq2] = Bitboard.getRookAttacks(sq1, BB_ZERO).AND(Bitboard.getRookAttacks(sq2, BB_ZERO));
      } else if (Square.getDiagonalIndex(sq1) === Square.getDiagonalIndex(sq2) || Square.getAntiDiagonalIndex(sq1) === Square.getAntiDiagonalIndex(sq2)) {
        bbLines[sq1][sq2] = Bitboard.getBishopAttacks(sq1, BB_ZERO).AND(Bitboard.getBishopAttacks(sq2, BB_ZERO));
      }
    }
  }

  return bbLines;
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
        .reduce((acc, pos) => acc.OR(fromBigInt(BigInt(1) << BigInt(pos.toSquareIndex()))), zeroBB());
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
const BB_SQUARES_BETWEEN: BB64Long[][] = generateSquaresBetween();
const BB_LINES: BB64Long[][] = createBBLines();
const KNIGHT_ATTACKS: BB64Long[] = generateAttacks(KNIGHT_MOVE_DIRECTIONS);
const KING_ATTACKS: BB64Long[] = generateAttacks(KING_MOVE_DIRECTIONS);

//console.info(LINE_MASKS.map(mask => mask.toString()));
// LINE_MASKS.forEach(mask => console.info(mask.toString()));



