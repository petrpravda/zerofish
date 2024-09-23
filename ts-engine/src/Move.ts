import {Square} from './Square';
import {Piece} from './Piece';
import {BoardState} from './BoardState';

export class Move {

  /**
   * Move Class Data Organization
   * -----------------------------
   * +---------------------------+
   * |        Move Bits           |
   * +---------------------------+
   * |  15   14   13   12 | 11 10 9  8  7  6  | 5  4  3  2  1  0 |
   * | [ FLAGS (4 bits) ] | [ FROM (6 bits) ] | [ TO (6 bits) ]  |
   * +---------------------------+
   *
   * Flags Bitmask:
   * +---------------------------+
   * |  15        14   13   12   | 11 10 9 8 7 6 5 4 3 2 1 0 |
   * | [ FLAG BITS ] [   UNUSED   ] | [     FLAGS     ]       |
   * +---------------------------+
   * - QUIET                    : 0b0000_000000000000
   * - DOUBLE_PUSH              : 0b0001_000000000000
   * - OO (Kingside Castling)   : 0b0010_000000000000
   * - OOO (Queenside Castling) : 0b0011_000000000000
   * - CAPTURE                  : 0b0100_000000000000
   * - EN_PASSANT               : 0b0101_000000000000
   * - PROMOTION                : 0b1000_000000000000
   * - PR_KNIGHT                : 0b1000_000000000000
   * - PR_BISHOP                : 0b1001_000000000000
   * - PR_ROOK                  : 0b1010_000000000000
   * - PR_QUEEN                 : 0b1011_000000000000
   * - PC_KNIGHT                : 0b1100_000000000000
   * - PC_BISHOP                : 0b1101_000000000000
   * - PC_ROOK                  : 0b1110_000000000000
   * - PC_QUEEN                 : 0b1111_000000000000
   * - FLAGS_MASK               : 0b1111_000000000000
   * - NULL                     : 0b0111_000000000000
   *
   * Bit Fields:
   * - `TO`: The destination square (6 bits).
   * - `FROM`: The origin square (6 bits).
   * - `FLAGS`: The move type and additional information (4 bits).
   */

  static readonly QUIET: number        = 0b0000_000000000000;
  static readonly DOUBLE_PUSH: number  = 0b0001_000000000000;
  static readonly OO: number           = 0b0010_000000000000;
  static readonly OOO: number          = 0b0011_000000000000;
  static readonly CAPTURE: number      = 0b0100_000000000000;
  static readonly EN_PASSANT: number   = 0b0101_000000000000;
  static readonly PROMOTION: number    = 0b1000_000000000000;
  static readonly PR_KNIGHT: number    = 0b1000_000000000000;
  static readonly PR_BISHOP: number    = 0b1001_000000000000;
  static readonly PR_ROOK: number      = 0b1010_000000000000;
  static readonly PR_QUEEN: number     = 0b1011_000000000000;
  static readonly PC_KNIGHT: number    = 0b1100_000000000000;
  static readonly PC_BISHOP: number    = 0b1101_000000000000;
  static readonly PC_ROOK: number      = 0b1110_000000000000;
  static readonly PC_QUEEN: number     = 0b1111_000000000000;
  static readonly FLAGS_MASK: number   = 0b1111_000000000000;
  static readonly NULL: number         = 0b0111_000000000000;

  static readonly NULL_MOVE: Move = new Move(0, 0, Move.NULL);

  private bits: number;

  constructor(bitsOrFrom: number, to?: number, flags?: number) {
    if (typeof to === 'number') {
      if (typeof flags === 'number') {
        this.bits = flags | (bitsOrFrom << 6) | to;
      } else {
        this.bits = (bitsOrFrom << 6) | to;
      }
    } else {
      this.bits = bitsOrFrom;
    }
  }

  to(): number {
    return this.bits & 0x3f;
  }

  from(): number {
    return (this.bits >>> 6) & 0x3f;
  }

  flags(): number {
    return this.bits & Move.FLAGS_MASK;
  }

  bitsValue(): number {
    return this.bits;
  }

  isPromotion(): boolean {
    return (this.bits & Move.PROMOTION) !== 0;
  }

  isCapture(): boolean {
    return (this.bits & Move.CAPTURE) !== 0;
  }

  equals(other: Move): boolean {
    return other !== null && this.bits === other.bitsValue();
  }

  uci(): string {
    const promo = (() => {
      switch (this.flags()) {
        case Move.PC_BISHOP:
        case Move.PR_BISHOP:
          return 'b';
        case Move.PC_KNIGHT:
        case Move.PR_KNIGHT:
          return 'n';
        case Move.PC_ROOK:
        case Move.PR_ROOK:
          return 'r';
        case Move.PC_QUEEN:
        case Move.PR_QUEEN:
          return 'q';
        default:
          return '';
      }
    })();

    return Square.getName(this.from()) + Square.getName(this.to()) + promo;
  }

  toString(): string {
    return this.isNullMove() ? 'NULL_MOVE' : this.uci();
  }

  isNullMove(): boolean {
    return this.flags() === Move.NULL;
  }

  static fromUciString(str: string, state: BoardState): Move {
    const fromSq = Square.getSquareFromName(str.substring(0, 2));
    const toSq = Square.getSquareFromName(str.substring(2, 4));
    let typeStr = '';
    let capturingPromotion = false;

    if (str.length > 4) {
      typeStr = str.substring(4);
      if (state.pieceAt(toSq) !== Piece.NONE) {
        capturingPromotion = true;
      }
    }

    let flags = (() => {
      switch (typeStr) {
        case 'q':
          return Move.PR_QUEEN;
        case 'n':
          return Move.PR_KNIGHT;
        case 'b':
          return Move.PR_BISHOP;
        case 'r':
          return Move.PR_ROOK;
        default:
          return Move.QUIET;
      }
    })();

    if (capturingPromotion) {
      flags |= Move.CAPTURE;
    }

    const moveWithPromoUci = new Move(fromSq, toSq, flags).uci();
    return state.generateLegalMoves()
        .find((m: Move) => m.uci() === moveWithPromoUci) ||
      (() => {
        throw new Error(`Cannot find legal ${moveWithPromoUci} in \n${state.toString()}\n${state.generateLegalMoves()}`);
      })();
  }

  getPieceType(): number {
    return ((this.flags() >>> 12) & 0b11) + 1;
  }

  getPieceTypeForSide(sideToPlay: number): number {
    return this.getPieceType() + sideToPlay * 8;
  }

  isCastling(): boolean {
    return this.flags() === Move.OO || this.flags() === Move.OOO;
  }
}
