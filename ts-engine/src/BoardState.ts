import {BB64Long, BB_ZERO, bitboardToString, idxBB, zeroBB} from './BB64Long';
import {Piece} from './Piece';
import {Side, SideType, SideUtils} from './Side';
import {Bitboard, PAWN_DOUBLE_PUSH_LINES, PAWN_FINAL_RANKS, PAWN_RANKS} from './Bitboard';
import {Fen} from './Fen';
import {Move} from './Move';
import {PieceSquareTable} from './PieceSquareTable';
import {Zobrist} from './Zobrist';
import {PieceType} from './PieceType';
// import {Square} from './Square';
// import {MoveList} from './MoveList';
import {Constants} from './Constants';
import {MoveList, Square } from './index';

export class BoardState {
  static TOTAL_PHASE = 24;
  static PIECE_PHASES = [0, 1, 1, 2, 4, 0];

  private piece_bb: BB64Long[] = new Array(Piece.PIECES_COUNT).fill(new BB64Long(0, 0));
  items: number[] = new Array(64).fill(Piece.NONE);
  hash: number = 0;
  fullMoveNormalized: number = 0;
  halfMoveClock: number = 0;
  phase: number = BoardState.TOTAL_PHASE;

  mg: number = 0;
  private eg: number = 0;

  //private checkers: BB64Long = zeroBB();
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
    //this.history = new Array(maxSearchDepth).fill(0);
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
    //result.history = [...this.history];
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
    this.piece_bb[piece] = this.piece_bb[piece].OR(idxBB(square));

    this.hash = this.hash ^ Zobrist.ZOBRIST_TABLE[piece][square];
  }

  removePiece(square: number): void {
    const piece = this.items[square];
    this.phase += BoardState.PIECE_PHASES[Piece.typeOf(piece)];
    this.mg -= PieceSquareTable.MGS[piece][square];
    this.eg -= PieceSquareTable.EGS[piece][square];

    this.hash = this.hash ^ Zobrist.ZOBRIST_TABLE[piece][square];

    this.piece_bb[piece] = this.piece_bb[piece].AND_NOT(idxBB(square));
    this.items[square] = Piece.NONE;
  }

  movePieceQuiet(fromSq: number, toSq: number): void {
    const piece = this.items[fromSq];
    this.mg += PieceSquareTable.MGS[piece][toSq] - PieceSquareTable.MGS[piece][fromSq];
    this.eg += PieceSquareTable.EGS[piece][toSq] - PieceSquareTable.EGS[piece][fromSq];

    this.hash = this.hash ^ Zobrist.ZOBRIST_TABLE[piece][fromSq] ^ Zobrist.ZOBRIST_TABLE[piece][toSq];

    this.piece_bb[piece] = this.piece_bb[piece].XOR(new BB64Long(1, 0).SHL(fromSq)).XOR(idxBB(toSq));
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
    // state.history[state.ply++] = move.bitsValue();
    state.movements = state.movements.OR(idxBB(move.to())).OR(idxBB(move.from()));

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
        state.enPassant = idxBB(move.from() + Square.direction(Square.FORWARD, state.sideToPlay));
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
      this.enPassant = BB_ZERO;
    }
  }

  isKingAttacked(): boolean {
    const us = this.sideToPlay;
    const them = SideUtils.flip(us);
    const ourKing = this.bitboardOfSideAndType(us, PieceType.KING).LSB();  // Gets the least significant bit position (king square)

    if (!Bitboard.pawnAttacks(idxBB(ourKing), us).AND(this.bitboardOfSideAndType(them, PieceType.PAWN)).empty()) {
      return true;
    }

    const knightAttacks = Bitboard.getKnightAttacks(ourKing);
    const knights = this.bitboardOfSideAndType(them, PieceType.KNIGHT);
    if (!knightAttacks.AND(knights).empty()) {
      return true;
    }

    const usBb = this.allPieces(us);
    const themBb = this.allPieces(them);
    const all = usBb.OR(themBb);

    const theirDiagonalSliders = this.diagonalSliders(them);
    const theirOrthogonalSliders = this.orthogonalSliders(them);

    if (!Bitboard.getRookAttacks(ourKing, all).AND(theirOrthogonalSliders).empty()) {
      return true;
    }

    return !Bitboard.getBishopAttacks(ourKing, all).AND(theirDiagonalSliders).empty();
  }

  attackedPieces(side: number): BB64Long {
    const workingState = this.sideToPlay === side ? this.doNullMove() : this;
    const quiescence = workingState.generateLegalQuiescence();
    const attackingMoves = quiescence.filter((m: Move) => workingState.pieceAt(m.to()) !== Piece.NONE);

    let result = BB_ZERO;
    for (const move of attackingMoves) {
      result = result.OR(idxBB(move.to()));
    }
    return result;
  }

  attackedPiecesUndefended(side: SideType): BB64Long {
    const sideThem = SideUtils.flip(side);
    const usBb = this.allPieces(side);
    const themBb = this.allPieces(sideThem);
    const all = usBb.OR(themBb);

    let attackedPieces = this.attackedPieces(side);
    let attackedUndefendedPieces = BB_ZERO;

    while (!attackedPieces.empty()) {
      const square = attackedPieces.LSB();
      let attackingPieces = this.attackersFromIncludingKings(square, all, sideThem);

      while (!attackingPieces.empty()) {
        const attackingSquare = attackingPieces.LSB();
        const allWithoutAttacker = all.AND_NOT(idxBB(attackingSquare));
        const defendingPieces = this.attackersFromIncludingKings(square, allWithoutAttacker, side);

        if (defendingPieces.empty()) {
          attackedUndefendedPieces = attackedUndefendedPieces.OR(idxBB(square));
        }

        attackingPieces = attackingPieces.popLSB();
      }
      attackedPieces = attackedPieces.popLSB();
    }

    return attackedUndefendedPieces;
  }

  smallestAttackerWithKing(square: number, side: SideType): number {
    return this.smallestAttacker(square, side, true);
  }

  smallestAttacker(square: number, side: SideType, withAttackingKing: boolean): number {
    const us = SideUtils.flip(side);
    const them = side;

    const pawns = Bitboard.pawnAttacks(idxBB(square), us).AND(this.bitboardOfSideAndType(them, PieceType.PAWN));
    if (!pawns.empty()) return pawns.LSB();

    const knights = Bitboard.getKnightAttacks(square).AND(this.bitboardOfSideAndType(them, PieceType.KNIGHT));
    if (!knights.empty()) return knights.LSB();

    const usBb = this.allPieces(us);
    const themBb = this.allPieces(them);
    const all = usBb.OR(themBb);

    const bishopAttacks = Bitboard.getBishopAttacks(square, all);
    const bishops = bishopAttacks.AND(this.bitboardOfSideAndType(them, PieceType.BISHOP));
    if (!bishops.empty()) return bishops.LSB();

    const rookAttacks = Bitboard.getRookAttacks(square, all);
    const rooks = rookAttacks.AND(this.bitboardOfSideAndType(them, PieceType.ROOK));
    if (!rooks.empty()) return rooks.LSB();

    const queens = bishopAttacks.OR(rookAttacks).AND(this.bitboardOfSideAndType(them, PieceType.QUEEN));
    if (!queens.empty()) return queens.LSB();

    if (withAttackingKing) {
      const kings = Bitboard.getKingAttacks(square).AND(this.bitboardOfSideAndType(them, PieceType.KING));
      if (!kings.empty()) return kings.LSB();
    }

    return Square.NO_SQUARE;
  }

  public generateLegalMovesInternal(onlyQuiescence: boolean): MoveList {
    const moves = new MoveList();
    const us = this.sideToPlay;
    const them = SideUtils.flip(this.sideToPlay);

    const usBb = this.allPieces(us);
    const themBb = this.allPieces(them);
    const all = usBb.OR(themBb);

    let ourKingBb = this.bitboardOfSideAndType(us, PieceType.KING);
    const ourKing = ourKingBb.LSB();
    const theirKing = this.bitboardOfSideAndType(them, PieceType.KING).LSB();

    const ourBishopsAndQueens = this.diagonalSliders(us);
    const theirBishopsAndQueens = this.diagonalSliders(them);
    const ourRooksAndQueens = this.orthogonalSliders(us);
    const theirRooksAndQueens = this.orthogonalSliders(them);

    // Calculate squares that are under attack
    let underAttack = Bitboard.pawnAttacks(this.bitboardOfSideAndType(them, PieceType.PAWN), them)
      .OR(Bitboard.getKingAttacks(theirKing));

    let b1 = this.bitboardOfSideAndType(them, PieceType.KNIGHT);
    while (!b1.empty()) {
      underAttack = underAttack.OR(Bitboard.getKnightAttacks(b1.LSB()));
      b1 = b1.popLSB();
    }

    b1 = theirBishopsAndQueens;
    while (!b1.empty()) {
      underAttack = underAttack.OR(Bitboard.getBishopAttacks(b1.LSB(), all.XOR(idxBB(ourKing))));
      b1 = b1.popLSB();
    }

    b1 = theirRooksAndQueens;
    while (!b1.empty()) {
      underAttack = underAttack.OR(Bitboard.getRookAttacks(b1.LSB(), all.XOR(idxBB(ourKing))));
      b1 = b1.popLSB();
    }

    const kingAttacks = Bitboard.getKingAttacks(ourKing); // TODO inline back
    b1 = kingAttacks.AND_NOT(usBb.OR(underAttack));

    moves.makeQ(ourKing, b1.AND_NOT(themBb));
    moves.makeC(ourKing, b1.AND(themBb));

    let captureMask: BB64Long;
    let quietMask: BB64Long;

    // Check for knight and pawn checkers
    let checkers = Bitboard.getKnightAttacks(ourKing).AND(this.bitboardOfSideAndType(them, PieceType.KNIGHT))
      .OR(Bitboard.pawnAttacksFromSquare(ourKing, us).AND(this.bitboardOfSideAndType(them, PieceType.PAWN)));

    // Check for sliding piece threats
    let candidates = Bitboard.getRookAttacks(ourKing, themBb).AND(theirRooksAndQueens)
      .OR(Bitboard.getBishopAttacks(ourKing, themBb).AND(theirBishopsAndQueens));

    let pinned = BB_ZERO;
    while (!candidates.empty()) {
      const attackingSquare = candidates.LSB();
      candidates = candidates.popLSB();
      let piecesBetweenKingAndAttacker = Bitboard.between(ourKing, attackingSquare).AND(usBb);

      if (piecesBetweenKingAndAttacker.empty()) {
        checkers = checkers.XOR(idxBB(attackingSquare));
      } else if (piecesBetweenKingAndAttacker.popcnt() === 1) {
        pinned = pinned.OR(piecesBetweenKingAndAttacker);
      }
    }

    const notPinned = pinned.NOT();
    switch (checkers.popcnt()) {
      case 2:
        return moves;  // Two checkers, king move only

      case 1: {
        const checkerSquare = checkers.LSB();
        switch (this.pieceTypeAt(checkerSquare)) {
          case PieceType.PAWN:
            if (checkers.equals(idxBB(us === Side.WHITE ? this.enPassant.SHR(8).LSB() : this.enPassant.SHL(8).LSB()))) {
              let enPassantSquare = this.enPassant.LSB();
              let nonPinnedPawnAttacks = Bitboard.pawnAttacks(this.enPassant, them).AND(this.bitboardOfSideAndType(us, PieceType.PAWN)).AND(notPinned);
              while (!nonPinnedPawnAttacks.empty()) {
                moves.push(new Move(nonPinnedPawnAttacks.LSB(), enPassantSquare, Move.EN_PASSANT));
                nonPinnedPawnAttacks = nonPinnedPawnAttacks.popLSB();
              }
            }
          // Intentional fall-through
          case PieceType.KNIGHT: {
            let piecesAttackingChecker = this.attackersFrom(checkerSquare, all, us).AND(notPinned);
            while (!piecesAttackingChecker.empty()) {
              const sq = piecesAttackingChecker.LSB();
              piecesAttackingChecker = piecesAttackingChecker.popLSB();
              if (this.pieceTypeAt(sq) === PieceType.PAWN && !idxBB(sq).AND(PAWN_FINAL_RANKS).empty()) {
                moves.makePC(sq, idxBB(checkerSquare));
              } else {
                moves.makeC(sq, idxBB(checkerSquare));
              }
            }
            return moves;
          }
          default:
            captureMask = checkers;
            quietMask = Bitboard.between(ourKing, checkerSquare);
            break;
        }
        break;
      }
      default:
        captureMask = themBb;
        quietMask = all.NOT();

        if (!this.enPassant.empty()) {
          const enPassantSquare = this.enPassant.LSB();
          let nonPinnedPawnAttacks = Bitboard.pawnAttacks(this.enPassant, them)
            .AND(this.bitboardOfSideAndType(us, PieceType.PAWN)).AND(notPinned);
          while (!nonPinnedPawnAttacks.empty()) {
            let pawnAttackSquare = nonPinnedPawnAttacks.LSB();
            nonPinnedPawnAttacks = nonPinnedPawnAttacks.popLSB();

            let themWoEp = themBb.XOR(idxBB(us === Side.WHITE ? this.enPassant.SHR(8).LSB() : this.enPassant.SHL(8).LSB()));
            let usBbEpMove = usBb.XOR(idxBB(pawnAttackSquare)).XOR(idxBB(enPassantSquare));
            let kingAttackersAfterEp = Bitboard.getRookAttacks(ourKing, themWoEp.OR(usBbEpMove)).AND(theirRooksAndQueens)
              .OR(Bitboard.getBishopAttacks(ourKing, themWoEp.OR(usBbEpMove)).AND(theirBishopsAndQueens));

            if (kingAttackersAfterEp.empty()) {
              moves.push(new Move(pawnAttackSquare, enPassantSquare, Move.EN_PASSANT));
            }
          }
        }

        if (!onlyQuiescence) {
          if ((this.movements.AND(Bitboard.castlingPiecesKingsideMask(us)).empty()) &&
            (all.OR(underAttack).AND(Bitboard.castlingBlockersKingsideMask(us)).empty())) {
            moves.push(new Move(us === Side.WHITE ? Square.E1 : Square.E8, us === Side.WHITE ? Square.G1 : Square.G8, Move.OO));
          }

          if ((this.movements.AND(Bitboard.castlingPiecesQueensideMask(us)).empty()) &&
            (all.OR(underAttack.AND(Bitboard.ignoreOOODanger(us).NOT())).AND(Bitboard.castlingBlockersQueensideMask(us)).empty())) {
            moves.push(new Move(us === Side.WHITE ? Square.E1 : Square.E8, us === Side.WHITE ? Square.C1 : Square.C8, Move.OOO));
          }
        }

        let pinnedRookBishopQueen = notPinned.NOT().AND(this.bitboardOfSideAndType(us, PieceType.KNIGHT).NOT());
        while (!pinnedRookBishopQueen.empty()) {
          const square = pinnedRookBishopQueen.LSB();
          pinnedRookBishopQueen = pinnedRookBishopQueen.popLSB();

          const attacksToKing = Bitboard.attacks(this.pieceTypeAt(square), square, all).AND(Bitboard.line(ourKing, square));
          if (!onlyQuiescence) {
            moves.makeQ(square, attacksToKing.AND(quietMask));
          }
          moves.makeC(square, attacksToKing.AND(captureMask));
        }

        let pinnedPawn = notPinned.NOT().AND(this.bitboardOfSideAndType(us, PieceType.PAWN));
        while (!pinnedPawn.empty()) {
          const square = pinnedPawn.LSB();
          pinnedPawn = pinnedPawn.popLSB();

          if (!idxBB(square).AND(PAWN_FINAL_RANKS).empty()) {
            const pawnCaptures = Bitboard.pawnAttacksFromSquare(square, us).AND(captureMask).AND(Bitboard.line(ourKing, square));
            moves.makePC(square, pawnCaptures);
          } else {
            const pawnCaptures = Bitboard.pawnAttacksFromSquare(square, us).AND(themBb).AND(Bitboard.line(ourKing, square));
            moves.makeC(square, pawnCaptures);

            if (!onlyQuiescence) {
              const singlePush = Bitboard.pawnPush(idxBB(square), us).AND_NOT(all).AND(Bitboard.line(ourKing, square));
              const doublePush = Bitboard.pawnPush(singlePush.AND(PAWN_DOUBLE_PUSH_LINES[us]), us).AND_NOT(all).AND(Bitboard.line(ourKing, square));

              moves.makeQ(square, singlePush);
              moves.makeDP(square, doublePush);
            }
          }
        }
        break;
    }

// Create a lambda function for generateMovesForPiece
    const generateMovesForPieceLambda = (
      pieceBitboard: BB64Long,
      getAttacksFunction: (position: number) => BB64Long
    ) => {
      while (!pieceBitboard.empty()) {
        const piecePosition = pieceBitboard.LSB();  // Long.numberOfTrailingZeros equivalent
        pieceBitboard = pieceBitboard.popLSB();     // Bitboard.extractLsb equivalent

        const attacks = getAttacksFunction(piecePosition);
        // console.info(bitboardToString(attacks));
        moves.makeC(piecePosition, attacks.AND(captureMask)); // Capture moves
        if (!onlyQuiescence) {
          moves.makeQ(piecePosition, attacks.AND(quietMask)); // Quiet moves
        }
      }
    };

// Generate attacks for knights, bishops, rooks, and queens
    generateMovesForPieceLambda(
      this.bitboardOfSideAndType(us, PieceType.KNIGHT).AND(notPinned),
      (sq) => Bitboard.getKnightAttacks(sq)
    );
    generateMovesForPieceLambda(
      ourBishopsAndQueens.AND(notPinned),
      (sq) => Bitboard.getBishopAttacks(sq, all)
    );
    // console.info(bitboardToString(all));
    // console.info(bitboardToString(quietMask));
    generateMovesForPieceLambda(
      ourRooksAndQueens.AND(notPinned),
      (sq) => Bitboard.getRookAttacks(sq, all)
    );

// Handle pawn moves
    let pawnBitboard = this.bitboardOfSideAndType(us, PieceType.PAWN).AND(notPinned).AND_NOT(PAWN_RANKS[us]);

    if (!onlyQuiescence) {
      // Single pawn pushes
      let singlePushTargets = (us === Side.WHITE) ? pawnBitboard.SHL(8) : pawnBitboard.SHR(8);
      singlePushTargets = singlePushTargets.AND_NOT(all);

      // Double pawn pushes
      let doublePushTargets = Bitboard.pawnPush(singlePushTargets.AND(PAWN_DOUBLE_PUSH_LINES[us]), us).AND(quietMask);
      singlePushTargets = singlePushTargets.AND(quietMask);

      while (!singlePushTargets.empty()) {
        const square = singlePushTargets.LSB();
        singlePushTargets = singlePushTargets.popLSB();
        moves.push(new Move(square - Square.direction(Square.FORWARD, us), square, Move.QUIET));
      }

      while (!doublePushTargets.empty()) {
        const square = doublePushTargets.LSB();
        doublePushTargets = doublePushTargets.popLSB();
        moves.push(new Move(square - Square.direction(Square.DOUBLE_FORWARD, us), square, Move.DOUBLE_PUSH));
      }
    }

// Pawn attacks
    let leftPawnAttacks = ((us === Side.WHITE)
      ? Bitboard.whiteLeftPawnAttacks(pawnBitboard)
      : Bitboard.blackRightPawnAttacks(pawnBitboard)).AND(captureMask);

    let rightPawnAttacks = ((us === Side.WHITE)
      ? Bitboard.whiteRightPawnAttacks(pawnBitboard)
      : Bitboard.blackLeftPawnAttacks(pawnBitboard)).AND(captureMask);

    while (!leftPawnAttacks.empty()) {
      const s = leftPawnAttacks.LSB();
      leftPawnAttacks = leftPawnAttacks.popLSB();
      moves.push(new Move(s - Square.direction(Square.FORWARD_LEFT, us), s, Move.CAPTURE));
    }

    while (!rightPawnAttacks.empty()) {
      const s = rightPawnAttacks.LSB();
      rightPawnAttacks = rightPawnAttacks.popLSB();
      moves.push(new Move(s - Square.direction(Square.FORWARD_RIGHT, us), s, Move.CAPTURE));
    }

// Handle pawns on the promotion rank
    pawnBitboard = this.bitboardOfSideAndType(us, PieceType.PAWN).AND(notPinned).AND(PAWN_RANKS[us]);

    if (!pawnBitboard.empty()) {
      if (!onlyQuiescence) {
        let singlePushTargets = (us === Side.WHITE) ? pawnBitboard.SHL(8) : pawnBitboard.SHR(8);
        singlePushTargets = singlePushTargets.AND(quietMask);
        while (!singlePushTargets.empty()) {
          const square = singlePushTargets.LSB();
          singlePushTargets = singlePushTargets.popLSB();
          moves.push(new Move(square - Square.direction(Square.FORWARD, us), square, Move.PR_QUEEN));
          moves.push(new Move(square - Square.direction(Square.FORWARD, us), square, Move.PR_ROOK));
          moves.push(new Move(square - Square.direction(Square.FORWARD, us), square, Move.PR_KNIGHT));
          moves.push(new Move(square - Square.direction(Square.FORWARD, us), square, Move.PR_BISHOP));
        }
      }

      leftPawnAttacks = ((us === Side.WHITE)
        ? Bitboard.whiteLeftPawnAttacks(pawnBitboard)
        : Bitboard.blackRightPawnAttacks(pawnBitboard)).AND(captureMask);

      rightPawnAttacks = ((us === Side.WHITE)
        ? Bitboard.whiteRightPawnAttacks(pawnBitboard)
        : Bitboard.blackLeftPawnAttacks(pawnBitboard)).AND(captureMask);

      while (!leftPawnAttacks.empty()) {
        const s = leftPawnAttacks.LSB();
        leftPawnAttacks = leftPawnAttacks.popLSB();
        moves.push(new Move(s - Square.direction(Square.FORWARD_LEFT, us), s, Move.PC_QUEEN));
        moves.push(new Move(s - Square.direction(Square.FORWARD_LEFT, us), s, Move.PC_ROOK));
        moves.push(new Move(s - Square.direction(Square.FORWARD_LEFT, us), s, Move.PC_KNIGHT));
        moves.push(new Move(s - Square.direction(Square.FORWARD_LEFT, us), s, Move.PC_BISHOP));
      }

      while (!rightPawnAttacks.empty()) {
        const s = rightPawnAttacks.LSB();
        rightPawnAttacks = rightPawnAttacks.popLSB();
        moves.push(new Move(s - Square.direction(Square.FORWARD_RIGHT, us), s, Move.PC_QUEEN));
        moves.push(new Move(s - Square.direction(Square.FORWARD_RIGHT, us), s, Move.PC_ROOK));
        moves.push(new Move(s - Square.direction(Square.FORWARD_RIGHT, us), s, Move.PC_KNIGHT));
        moves.push(new Move(s - Square.direction(Square.FORWARD_RIGHT, us), s, Move.PC_BISHOP));
      }
    }

    return moves;
  }

  public generateLegalMoves(): MoveList {
    return this.generateLegalMovesInternal(false);
  }

  public generateLegalQuiescence(): MoveList {
    return this.generateLegalMovesInternal(true);
  }

  public toString(): string {
    let result = Constants.CHESSBOARD_LINE;
    for (let i = 56; i >= 0; i -= 8) {
      for (let j = 0; j < 8; j++) {
        const piece = this.items[i + j];
        const notation = Piece.getNotation(piece);
        result += `| ${notation} `;
      }
      result += `|\n${Constants.CHESSBOARD_LINE}`;
    }
    result += `Fen: ${Fen.toFen(this)}`;
    return result;
  }

  public toFen(): string {
    return Fen.toFen(this);
  }

  // Getter for mid-game score
  public getMg(): number {
    return this.mg;
  }

  // Getter for end-game score
  public getEg(): number {
    return this.eg;
  }

  // Interpolated score based on game phase
  public interpolatedScore(): number {
    const phase = (this.phase * 256 + (BoardState.TOTAL_PHASE / 2)) / BoardState.TOTAL_PHASE;
    return (this.getMg() * (256 - phase) + this.getEg() * phase) / 256;
  }

  // // Check if the current side is in check
  // public isInCheck(): boolean {
  //   this.generateLegalMoves();
  //   //return !this.checkers.empty();
  // }

  // Check if the current side is in checkmate
  public isInCheckMate(): boolean {
    return this.generateLegalMoves().length === 0;
  }

  // Check if a given move is a capture
  public isCapture(move: string): boolean {
    const parsedMove = Move.fromUciString(move, this);
    return this.pieceAt(parsedMove.to()) !== Piece.NONE;
  }

  // // Record type for score and pieces taken
  // public static ScoreOutcome = class {
  //   constructor(public score: number, public piecesTaken: number) {}
  // };
  //
  // /**
  //  * @param square battle square
  //  * @param side perspective of score, starting move
  //  * @return score in basic material values, the higher, the better, no matter if white or black
  //  */
  // public seeScore(square: number, side: number): InstanceType<typeof BoardState.ScoreOutcome> {
  //   let processedSide = side;
  //   let score = 0;
  //   let piecesTaken = 0;
  //   let evaluatedState = this.getSideToPlay() !== processedSide ? this.doNullMove() : this;
  //
  //   while (true) {
  //     const attacker = evaluatedState.smallestAttackerWithKing(square, processedSide);
  //     if (attacker === -1) { // NO_SQUARE assumed to be -1 in TS
  //       break;
  //     }
  //
  //     const possibleMoves = evaluatedState
  //       .generateLegalMoves(true)
  //       .filter(m => m.from() === attacker && m.to() === square);
  //
  //     if (possibleMoves.length === 0) {
  //       break;
  //     }
  //
  //     const pieceType = evaluatedState.pieceTypeAt(square);
  //     if (pieceType === PieceType.KING) {
  //       score = 0;
  //       break;
  //     }
  //
  //     score += evaluatedState.getBasicMaterialValue(square);
  //     piecesTaken++;
  //     processedSide = Side.flip(processedSide);
  //     evaluatedState = evaluatedState.doMove(possibleMoves[0]);
  //   }
  //
  //   return new BoardState.ScoreOutcome(-score * Side.multiplicator(side), piecesTaken);
  // }

  // Private method to get basic material value
  private getBasicMaterialValue(square: number): number {
    const piece = this.pieceAt(square);
    return PieceSquareTable.BASIC_MATERIAL_VALUE[Piece.typeOf(piece)] * (Piece.sideOf(piece) === Side.WHITE ? 1 : -1);
  }

  public generateUciMoves(): string[] {
    return this.generateLegalMoves().map(m => m.uci());
  }

  public hasNonPawnMaterial(side: SideType): boolean {
    const start = Piece.makePiece(side, PieceType.KNIGHT);
    const end = Piece.makePiece(side, PieceType.QUEEN);
    for (let piece = start; piece <= end; piece++) {
      if (!this.bitboardOf(piece).empty()) {
        return true;
      }
    }
    return false;
  }
}
