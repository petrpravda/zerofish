export class Piece {
  public static readonly WHITE_PAWN = 0;
  public static readonly WHITE_KNIGHT = 1;
  public static readonly WHITE_BISHOP = 2;
  public static readonly WHITE_ROOK = 3;
  public static readonly WHITE_QUEEN = 4;
  public static readonly WHITE_KING = 5;
  public static readonly BLACK_PAWN = 8;
  public static readonly BLACK_KNIGHT = 9;
  public static readonly BLACK_BISHOP = 10;
  public static readonly BLACK_ROOK = 11;
  public static readonly BLACK_QUEEN = 12;
  public static readonly BLACK_KING = 13;
  public static readonly NONE = 14;

  public static readonly PIECES_COUNT = 15;

  public static flip(piece: number): number {
    return piece ^ 8;
  }

  public static typeOf(piece: number): number {
    return piece & 0b111;
  }

  public static sideOf(piece: number): number {
    return (piece & 0b1000) >>> 3;
  }

  public static makePiece(side: number, pieceType: number): number {
    return (side << 3) + pieceType;
  }

  public static getNotation(piece: number): string {
    switch (piece) {
      case Piece.WHITE_PAWN: return "P";
      case Piece.WHITE_KNIGHT: return "N";
      case Piece.WHITE_BISHOP: return "B";
      case Piece.WHITE_ROOK: return "R";
      case Piece.WHITE_QUEEN: return "Q";
      case Piece.WHITE_KING: return "K";
      case Piece.BLACK_PAWN: return "p";
      case Piece.BLACK_KNIGHT: return "n";
      case Piece.BLACK_BISHOP: return "b";
      case Piece.BLACK_ROOK: return "r";
      case Piece.BLACK_QUEEN: return "q";
      case Piece.BLACK_KING: return "k";
      default: return " ";
    }
  }
}
