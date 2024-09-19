import {BB64Long, zeroBB} from './BB64Long';
import {Piece} from './Piece';
import {Side, SideType, SideUtils} from './Side';
import {Bitboard} from './Bitboard';
import {Fen} from './Fen';
import {Move} from './Move';
import {PieceSquareTable} from './PieceSquareTable';
import {Zobrist} from './Zobrist';
import {PieceType} from './PieceType';
import {Square} from './Square';

export class BoardState {
  static TOTAL_PHASE = 24;
  static PIECE_PHASES = [0, 1, 1, 2, 4, 0];

  ply: number;
  private history: number[];
  private piece_bb: BB64Long[] = new Array(Piece.PIECES_COUNT).fill(new BB64Long(0, 0));
  items: number[] = new Array(64).fill(Piece.NONE);
  hash: number = 0;
  fullMoveNormalized: number = 0;
  halfMoveClock: number = 0;
  phase: number = BoardState.TOTAL_PHASE;

  mg: number = 0;
  private eg: number = 0;

  private checkers: BB64Long = zeroBB();
  movements: BB64Long;
  enPassant: BB64Long;
  sideToPlay: SideType;

  constructor(
    items: number[],
    sideToPlay: SideType,
    movements: BB64Long,
    enPassantMask: BB64Long,
    halfMoveClock: number,
    fullMoveCount: number,
    maxSearchDepth: number
  ) {
    for (let i = 0; i < 64; i++) {
      const item = items[i];
      if (item !== Piece.NONE) {
        this.setPieceAt(item, i);
      } else {
        this.items[i] = Piece.NONE;
      }
    }

    this.sideToPlay = sideToPlay;

    if (sideToPlay === Side.BLACK)
      this.hash = this.hash ^ Zobrist.SIDE;

    this.enPassant = enPassantMask;
    if (!this.enPassant.empty()) {
      this.hash = this.hash ^ Zobrist.EN_PASSANT[this.enPassant.LSB() & 0b111];
    }

    this.movements = movements;

    this.halfMoveClock = halfMoveClock;
    this.fullMoveNormalized = (fullMoveCount - 1) * 2 + (sideToPlay === Side.WHITE ? 0 : 1);
    this.history = new Array(maxSearchDepth).fill(0);
    this.ply = 0;
  }

  static fromFen(fen: string): BoardState {
    return Fen.fromFen(fen, null);
  }

  static fromFenWithDepth(fen: string, maxSearchDepth: number): BoardState {
    return Fen.fromFen(fen, maxSearchDepth);
  }

  clone(): BoardState {
    const result = Object.create(this) as BoardState;
    result.piece_bb = this.piece_bb.map(bb => new BB64Long(bb.lower, bb.upper));
    result.items = [...this.items];
    result.history = [...this.history];
    return result;
  }

  pieceAt(square: number): number {
    return this.items[square];
  }

  pieceTypeAt(square: number): number {
    return Piece.typeOf(this.items[square]);
  }

  setPieceAt(piece: number, square: number): void {
    this.phase -= BoardState.PIECE_PHASES[Piece.typeOf(piece)];
    this.mg += PieceSquareTable.MGS[piece][square];
    this.eg += PieceSquareTable.EGS[piece][square];

    this.items[square] = piece;
    this.piece_bb[piece] = this.piece_bb[piece].OR(new BB64Long(1, 0).SHL(square));

    this.hash = this.hash ^ Zobrist.ZOBRIST_TABLE[piece][square];
  }

  removePiece(square: number): void {
    const piece = this.items[square];
    this.phase += BoardState.PIECE_PHASES[Piece.typeOf(piece)];
    this.mg -= PieceSquareTable.MGS[piece][square];
    this.eg -= PieceSquareTable.EGS[piece][square];

    this.hash = this.hash ^ Zobrist.ZOBRIST_TABLE[piece][square];

    this.piece_bb[piece] = this.piece_bb[piece].AND_NOT(new BB64Long(1, 0).SHL(square));
    this.items[square] = Piece.NONE;
  }

  movePieceQuiet(fromSq: number, toSq: number): void {
    const piece = this.items[fromSq];
    this.mg += PieceSquareTable.MGS[piece][toSq] - PieceSquareTable.MGS[piece][fromSq];
    this.eg += PieceSquareTable.EGS[piece][toSq] - PieceSquareTable.EGS[piece][fromSq];

    this.hash = this.hash ^ Zobrist.ZOBRIST_TABLE[piece][fromSq] ^ Zobrist.ZOBRIST_TABLE[piece][toSq];

    this.piece_bb[piece] = this.piece_bb[piece].XOR(new BB64Long(1, 0).SHL(fromSq)).XOR(new BB64Long(1, 0).SHL(toSq));
    this.items[toSq] = piece;
    this.items[fromSq] = Piece.NONE;
  }

  movePiece(fromSq: number, toSq: number): void {
    this.removePiece(toSq);
    this.movePieceQuiet(fromSq, toSq);
  }

  computeHash(): number {
    return this.hash;
  }

  bitboardOf(piece: number): BB64Long {
    return this.piece_bb[piece];
  }

  bitboardOfSideAndType(side: SideType, pieceType: number): BB64Long {
    return this.piece_bb[Piece.makePiece(side, pieceType)];
  }

  diagonalSliders(side: SideType): BB64Long {
    return side === Side.WHITE
      ? this.piece_bb[Piece.WHITE_BISHOP].OR(this.piece_bb[Piece.WHITE_QUEEN])
      : this.piece_bb[Piece.BLACK_BISHOP].OR(this.piece_bb[Piece.BLACK_QUEEN]);
  }

  orthogonalSliders(side: SideType): BB64Long {
    return side === Side.WHITE
      ? this.piece_bb[Piece.WHITE_ROOK].OR(this.piece_bb[Piece.WHITE_QUEEN])
      : this.piece_bb[Piece.BLACK_ROOK].OR(this.piece_bb[Piece.BLACK_QUEEN]);
  }

  allPieces(side: SideType): BB64Long {
    return side === Side.WHITE
      ? this.piece_bb[Piece.WHITE_PAWN]
        .OR(this.piece_bb[Piece.WHITE_KNIGHT])
        .OR(this.piece_bb[Piece.WHITE_BISHOP])
        .OR(this.piece_bb[Piece.WHITE_ROOK])
        .OR(this.piece_bb[Piece.WHITE_QUEEN])
        .OR(this.piece_bb[Piece.WHITE_KING])
      : this.piece_bb[Piece.BLACK_PAWN]
        .OR(this.piece_bb[Piece.BLACK_KNIGHT])
        .OR(this.piece_bb[Piece.BLACK_BISHOP])
        .OR(this.piece_bb[Piece.BLACK_ROOK])
        .OR(this.piece_bb[Piece.BLACK_QUEEN])
        .OR(this.piece_bb[Piece.BLACK_KING]);
  }

  allPiecesOnBoard(): BB64Long {
    return this.allPieces(Side.WHITE).OR(this.allPieces(Side.BLACK));
  }

  attackersFrom(square: number, occ: BB64Long, side: SideType): BB64Long {
    return side === Side.WHITE
      ? Bitboard.pawnAttacksFromSquare(square, Side.BLACK).AND(this.piece_bb[Piece.WHITE_PAWN])
        .OR(Bitboard.getKnightAttacks(square).AND(this.piece_bb[Piece.WHITE_KNIGHT]))
        .OR(Bitboard.getBishopAttacks(square, occ).AND(this.piece_bb[Piece.WHITE_BISHOP].OR(this.piece_bb[Piece.WHITE_QUEEN])))
        .OR(Bitboard.getRookAttacks(square, occ).AND(this.piece_bb[Piece.WHITE_ROOK].OR(this.piece_bb[Piece.WHITE_QUEEN])))
      : Bitboard.pawnAttacksFromSquare(square, Side.WHITE).AND(this.piece_bb[Piece.BLACK_PAWN])
        .OR(Bitboard.getKnightAttacks(square).AND(this.piece_bb[Piece.BLACK_KNIGHT]))
        .OR(Bitboard.getBishopAttacks(square, occ).AND(this.piece_bb[Piece.BLACK_BISHOP].OR(this.piece_bb[Piece.BLACK_QUEEN])))
        .OR(Bitboard.getRookAttacks(square, occ).AND(this.piece_bb[Piece.BLACK_ROOK].OR(this.piece_bb[Piece.BLACK_QUEEN])));
  }

  attackersFromIncludingKings(square: number, occ: BB64Long, side: SideType): BB64Long {
    return side === Side.WHITE
      ? Bitboard.pawnAttacksFromSquare(square, Side.BLACK).AND(this.piece_bb[Piece.WHITE_PAWN])
        .OR(Bitboard.getKingAttacks(square).AND(this.piece_bb[Piece.WHITE_KING]))
        .OR(Bitboard.getKnightAttacks(square).AND(this.piece_bb[Piece.WHITE_KNIGHT]))
        .OR(Bitboard.getBishopAttacks(square, occ).AND(this.piece_bb[Piece.WHITE_BISHOP].OR(this.piece_bb[Piece.WHITE_QUEEN])))
        .OR(Bitboard.getRookAttacks(square, occ).AND(this.piece_bb[Piece.WHITE_ROOK].OR(this.piece_bb[Piece.WHITE_QUEEN])))
      : Bitboard.pawnAttacksFromSquare(square, Side.WHITE).AND(this.piece_bb[Piece.BLACK_PAWN])
        .OR(Bitboard.getKingAttacks(square).AND(this.piece_bb[Piece.BLACK_KING]))
        .OR(Bitboard.getKnightAttacks(square).AND(this.piece_bb[Piece.BLACK_KNIGHT]))
        .OR(Bitboard.getBishopAttacks(square, occ).AND(this.piece_bb[Piece.BLACK_BISHOP].OR(this.piece_bb[Piece.BLACK_QUEEN])))
        .OR(Bitboard.getRookAttacks(square, occ).AND(this.piece_bb[Piece.BLACK_ROOK].OR(this.piece_bb[Piece.BLACK_QUEEN])));
  }

  isEmptyAt(square: number): boolean {
    return this.items[square] === Piece.NONE;
  }

  doMove(move: Move): BoardState {
    return BoardState.performMove(move, this);
  }

  doMoveUCI(uciMove: string): BoardState {
    const move = this.generateLegalMoves().find(m => m.toString() === uciMove);
    if (!move) throw new Error("Invalid UCI move");
    return BoardState.performMove(move, this);
  }

  doNullMove(): BoardState {
    return this.performNullMove(this);
  }

  private performNullMove(oldBoardState: BoardState): BoardState {
    const state = oldBoardState.clone();
    state.halfMoveClock += 1;
    state.clearEnPassant();
    state.sideToPlay = SideUtils.flip(state.sideToPlay);
    state.hash ^= Zobrist.SIDE;
    return state;
  }

  public static performMove(move: Move, oldBoardState: BoardState): BoardState {
    const state = oldBoardState.clone();
    state.fullMoveNormalized += 1;
    state.halfMoveClock += 1;
    state.history[state.ply++] = move.bitsValue();
    state.movements = state.movements.OR(new BB64Long(1 << move.to(), 0)).OR(new BB64Long(1 << move.from(), 0));

    if (Piece.typeOf(state.items[move.from()]) === PieceType.PAWN) {
      state.halfMoveClock = 0;
    }

    state.clearEnPassant();

    switch (move.flags()) {
      case Move.QUIET:
        state.movePieceQuiet(move.from(), move.to());
        break;
      case Move.DOUBLE_PUSH:
        state.movePieceQuiet(move.from(), move.to());
        state.enPassant = new BB64Long(1 << (move.from() + Square.direction(Square.FORWARD, state.sideToPlay)), 0);
        state.hash ^= Zobrist.EN_PASSANT[state.enPassant.LSB() & 0b111];
        break;
      case Move.OO:
        if (state.sideToPlay === Side.WHITE) {
          state.movePieceQuiet(Square.E1, Square.G1);
          state.movePieceQuiet(Square.H1, Square.F1);
        } else {
          state.movePieceQuiet(Square.E8, Square.G8);
          state.movePieceQuiet(Square.H8, Square.F8);
        }
        break;
      case Move.OOO:
        if (state.sideToPlay === Side.WHITE) {
          state.movePieceQuiet(Square.E1, Square.C1);
          state.movePieceQuiet(Square.A1, Square.D1);
        } else {
          state.movePieceQuiet(Square.E8, Square.C8);
          state.movePieceQuiet(Square.A8, Square.D8);
        }
        break;
      case Move.EN_PASSANT:
        state.movePieceQuiet(move.from(), move.to());
        state.removePiece(move.to() + Square.direction(Square.BACK, state.sideToPlay));
        break;
      case Move.PR_KNIGHT:
      case Move.PR_BISHOP:
      case Move.PR_ROOK:
      case Move.PR_QUEEN:
        state.removePiece(move.from());
        state.setPieceAt(Piece.makePiece(state.sideToPlay, move.getPieceType()), move.to());
        break;
      case Move.PC_KNIGHT:
      case Move.PC_BISHOP:
      case Move.PC_ROOK:
      case Move.PC_QUEEN:
        state.removePiece(move.from());
        state.removePiece(move.to());
        state.setPieceAt(Piece.makePiece(state.sideToPlay, move.getPieceType()), move.to());
        break;
      case Move.CAPTURE:
        state.halfMoveClock = 0;
        state.movePiece(move.from(), move.to());
        break;
    }

    state.sideToPlay = SideUtils.flip(state.sideToPlay);
    state.hash ^= Zobrist.SIDE;

    return state;
  }

  private clearEnPassant(): void {
    if (this.enPassant && !this.enPassant.empty()) {
      const lsb = this.enPassant.LSB();
      this.hash ^= Zobrist.EN_PASSANT[lsb & 0b111];
      this.enPassant = new BB64Long(0, 0);
    }
  }

  generateLegalMoves(): Move[] {
    throw Error("TBD");
  }

}
