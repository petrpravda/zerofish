export class PieceType {
  public static readonly PAWN: number = 0;
  public static readonly KNIGHT: number = 1;
  public static readonly BISHOP: number = 2;
  public static readonly ROOK: number = 3;
  public static readonly QUEEN: number = 4;
  public static readonly KING: number = 5;

  /**
   * Converts a SAN character (e.g., 'N' for knight, 'B' for bishop) to the corresponding PieceType.
   *
   * @param pieceCode The SAN character representing the piece ('N', 'B', 'R', 'Q', 'K').
   * @returns The corresponding PieceType value, or throws an error if the input is invalid.
   */
  static fromSan(pieceCode: string): number | undefined {
    switch (pieceCode.toUpperCase()) {
      case 'P':
        return PieceType.PAWN;
      case 'N':
        return PieceType.KNIGHT;
      case 'B':
        return PieceType.BISHOP;
      case 'R':
        return PieceType.ROOK;
      case 'Q':
        return PieceType.QUEEN;
      case 'K':
        return PieceType.KING;
      default:
        return undefined;
    }
  }

  static toSan(pieceType: number): string | undefined {
    switch (pieceType) {
      case PieceType.PAWN:
        return ''; // Pawns are represented without a letter in SAN
      case PieceType.KNIGHT:
        return 'N';
      case PieceType.BISHOP:
        return 'B';
      case PieceType.ROOK:
        return 'R';
      case PieceType.QUEEN:
        return 'Q';
      case PieceType.KING:
        return 'K';
      default:
        return undefined;
    }
  }
}
