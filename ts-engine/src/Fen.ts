import {BoardState} from './BoardState';
import {Piece} from './Piece';
import {Side} from './Side';
import {
  Bitboard,
  BLACK_KING_INITIAL_SQUARE, BLACK_KINGS_ROOK_MASK, BLACK_QUEENS_ROOK_MASK,
  WHITE_KING_INITIAL_SQUARE,
  WHITE_KINGS_ROOK_MASK,
  WHITE_QUEENS_ROOK_MASK
} from './Bitboard';
import {Square} from './Square';
import {zeroBB} from './BB64Long';

export const START_POS = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

export class Fen {
  private static readonly REGEX_FEN_FREE = /Fen: (.*)/;
  public static readonly MAX_SEARCH_DEPTH = 100;
  private static readonly REGEX_EXPAND = /[2-8]/g;
  private static readonly ONES = "11111111";

  public static toFen(state: BoardState): string {
    let fen = "";
    let count = 0;
    let rankCounter = 1;
    let sqCount = 0;

    for (let rank = 7; rank >= 0; rank--) {
      for (let file = 0; file <= 7; file++) {
        const square = (rank << 3) + file;
        const piece = state.items[square];
        if (piece !== Piece.NONE) {
          if (count > 0) {
            fen += count;
          }
          fen += Piece.getNotation(piece);
          count = 0;
        } else {
          count++;
        }
        if ((sqCount + 1) % 8 === 0) {
          if (count > 0) {
            fen += count;
            count = 0;
          }
          if (rankCounter < 8) {
            fen += "/";
          }
          rankCounter++;
        }
        sqCount++;
      }
    }

    fen += state.sideToPlay === Side.WHITE ? " w" : " b";

    let rights = "";
    if (Bitboard.castlingPiecesKingsideMask(Side.WHITE).AND(state.movements).empty()) rights += "K";
    if (Bitboard.castlingPiecesQueensideMask(Side.WHITE).AND(state.movements).empty()) rights += "Q";
    if (Bitboard.castlingPiecesKingsideMask(Side.BLACK).AND(state.movements).empty()) rights += "k";
    if (Bitboard.castlingPiecesQueensideMask(Side.BLACK).AND(state.movements).empty()) rights += "q";

    fen += rights === "" ? " -" : ` ${rights}`;

    if (!state.enPassant.empty()) {
      fen += ` ${Square.getName(state.enPassant.LSB())}`;
    } else {
      fen += " -";
    }

    fen += ` ${state.halfMoveClock} ${(state.fullMoveNormalized / 2) + 1}`;

    return fen;
  }

  public static expandFenPieces(fenPieces: string): string {
    return fenPieces.replace(Fen.REGEX_EXPAND, (match) => {
      const countOfSpaces = parseInt(match);
      return Fen.ONES.substring(0, countOfSpaces);
    });
  }

  public static fromFen(fen: string, maxSearchDepth: number | null = null): BoardState {
    const fenParts = fen.split(/\s+/);

    const squares = Fen.expandFenPieces(fenParts[0]);
    const squaresList = squares.split("/").reverse();
    const items = squaresList.flatMap(line =>
      Array.from(line).map(char => {
        switch (char) {
          case '1': return Piece.NONE;
          case 'P': return Piece.WHITE_PAWN;
          case 'N': return Piece.WHITE_KNIGHT;
          case 'B': return Piece.WHITE_BISHOP;
          case 'R': return Piece.WHITE_ROOK;
          case 'Q': return Piece.WHITE_QUEEN;
          case 'K': return Piece.WHITE_KING;
          case 'p': return Piece.BLACK_PAWN;
          case 'n': return Piece.BLACK_KNIGHT;
          case 'b': return Piece.BLACK_BISHOP;
          case 'r': return Piece.BLACK_ROOK;
          case 'q': return Piece.BLACK_QUEEN;
          case 'k': return Piece.BLACK_KING;
          default: throw new Error(`Character "${char}" not known.`);
        }
      })
    );

    let entry = zeroBB();
    const castlingFlags = fenParts[2];
    if (!castlingFlags.includes("K") || items[WHITE_KING_INITIAL_SQUARE] !== Piece.WHITE_KING ||
      items[WHITE_KINGS_ROOK_MASK.LSB()] !== Piece.WHITE_ROOK) {
      entry = entry.OR(WHITE_KINGS_ROOK_MASK);
    }
    if (!castlingFlags.includes("Q") || items[WHITE_KING_INITIAL_SQUARE] !== Piece.WHITE_KING ||
      items[WHITE_QUEENS_ROOK_MASK.LSB()] !== Piece.WHITE_ROOK) {
      entry = entry.OR(WHITE_QUEENS_ROOK_MASK);
    }
    if (!castlingFlags.includes("k") || items[BLACK_KING_INITIAL_SQUARE] !== Piece.BLACK_KING ||
      items[BLACK_KINGS_ROOK_MASK.LSB()] !== Piece.BLACK_ROOK) {
      entry = entry.OR(BLACK_KINGS_ROOK_MASK);
    }
    if (!castlingFlags.includes("q") || items[BLACK_KING_INITIAL_SQUARE] !== Piece.BLACK_KING ||
      items[BLACK_QUEENS_ROOK_MASK.LSB()] !== Piece.BLACK_ROOK) {
      entry = entry.OR(BLACK_QUEENS_ROOK_MASK);
    }

    const enpassant = fenParts[3];
    const enPassantMask = zeroBB();
    if (enpassant.length == 2) {
      enPassantMask.setBit(Square.getSquareFromName(enpassant));
    }

    const halfMoveClock = parseInt(fenParts[4]);
    const fullMoveCount = parseInt(fenParts[5]);

    const sideToPlay = fenParts[1].toLowerCase() === "w" ? Side.WHITE : Side.BLACK;

    return new BoardState(items, sideToPlay, entry, enPassantMask, halfMoveClock, fullMoveCount, maxSearchDepth ?? Fen.MAX_SEARCH_DEPTH);
  }

  // public static fromFenFree(freeFen: string): BoardPosition {
  //   const match = Fen.REGEX_FEN_FREE.exec(freeFen);
  //   if (match) {
  //     return BoardPosition.fromFen(match[1]);
  //   } else {
  //     throw new Error(`${freeFen} doesn't contain 'Fen: '`);
  //   }
  // }
}
