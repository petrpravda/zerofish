import {Side} from './Side';

export class Square {
  // Constants representing directions on the chessboard
  public static readonly FORWARD: number = 8;
  public static readonly BACK: number = -8;
  public static readonly RIGHT: number = 1;
  public static readonly LEFT: number = -1;
  public static readonly FORWARD_RIGHT: number = Square.FORWARD + Square.RIGHT;
  public static readonly FORWARD_LEFT: number = Square.FORWARD + Square.LEFT;
  public static readonly DOUBLE_FORWARD: number = Square.FORWARD + Square.FORWARD;

  // Square definitions (based on index, starting from 0)
  public static readonly A1: number = 0;
  public static readonly B1: number = 1;
  public static readonly C1: number = 2;
  public static readonly D1: number = 3;
  public static readonly E1: number = 4;
  public static readonly F1: number = 5;
  public static readonly G1: number = 6;
  public static readonly H1: number = 7;

  public static readonly E4: number = 28;

  public static readonly A8: number = 56;
  public static readonly B8: number = 57;
  public static readonly C8: number = 58;
  public static readonly D8: number = 59;
  public static readonly E8: number = 60;
  public static readonly F8: number = 61;
  public static readonly G8: number = 62;
  public static readonly H8: number = 63;

  public static readonly NO_SQUARE: number = 64;

  // Get the name of the square based on its index (e.g., 0 -> "a1")
  public static getName(square: number): string {
    let file: string = String.fromCharCode((square & 0b111) + 'a'.charCodeAt(0));
    let rank: string = String.fromCharCode(((square & 0b111000) >>> 3) + '1'.charCodeAt(0));
    return file + rank;
  }

  // Get the square index based on UCI notation (e.g., "a1" -> 0)
  public static getSquareFromName(square: string): number {
    let file: number = square.charCodeAt(0) - 'a'.charCodeAt(0);
    let rank: number = square.charCodeAt(1) - '1'.charCodeAt(0);
    return (rank << 3) | file;
  }

  // Get the file character (a-h) from the square index
  public static getFile(square: number): string {
    return String.fromCharCode((square & 0b111) + 'a'.charCodeAt(0));
  }

  // Get the rank character (1-8) from the square index
  public static getRank(square: number): string {
    return String.fromCharCode(((square & 0b111000) >>> 3) + '1'.charCodeAt(0));
  }

  // Get the rank index (0-7) from the square index
  public static getRankIndex(square: number): number {
    return square >>> 3;
  }

  // Get the file index (0-7) from the square index
  public static getFileIndex(square: number): number {
    return square & 7;
  }

  // Get the diagonal index for a given square (0-14 range for 8x8 board)
  public static getDiagonalIndex(square: number): number {
    return 7 + Square.getRankIndex(square) - Square.getFileIndex(square);
  }

  // Get the anti-diagonal index for a given square (0-14 range for 8x8 board)
  public static getAntiDiagonalIndex(square: number): number {
    return Square.getRankIndex(square) + Square.getFileIndex(square);
  }

  // Get the direction based on the side (White or Black)
  public static direction(direction: number, side: number): number {
    return side === Side.WHITE ? direction : -direction;
  }
}
