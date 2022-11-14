  pub type Piece = u8;

  pub const WHITE_PAWN: Piece = 0;
  pub const WHITE_KNIGHT: Piece = 1;
  pub const WHITE_BISHOP: Piece = 2;
  pub const WHITE_ROOK: Piece = 3;
  pub const WHITE_QUEEN: Piece = 4;
  pub const WHITE_KING: Piece = 5;
  pub const BLACK_PAWN: Piece = 8;
  pub const BLACK_KNIGHT: Piece = 9;
  pub const BLACK_BISHOP: Piece = 10;
  pub const BLACK_ROOK: Piece = 11;
  pub const BLACK_QUEEN: Piece = 12;
  pub const BLACK_KING: Piece = 13;
  pub const NONE: Piece = 14;

  pub const PIECES_COUNT: usize = 15;

  pub fn parse_piece(c: char) -> Piece {
    match c {
      'P' => WHITE_PAWN,
      'N' => WHITE_KNIGHT,
      'B' => WHITE_BISHOP,
      'R' => WHITE_ROOK,
      'Q' => WHITE_QUEEN,
      'K' => WHITE_KING,
      'p' => BLACK_PAWN,
      'n' => BLACK_KNIGHT,
      'b' => BLACK_BISHOP,
      'r' => BLACK_ROOK,
      'q' => BLACK_QUEEN,
      'k' => BLACK_KING,
      _ => NONE,
    }
  }
//
//     public static int flip(int piece) { return piece ^ 8; }
//
//     public static int typeOf(int piece){
//         return piece & 0b111;
//     }
//
//     public static int sideOf(int piece){
//         return (piece & 0b1000) >>> 3;
//     }
//
//     public static int makePiece(int side, int pieceType){
//         return (side << 3) + pieceType;
//     }
//
//     public static String getNotation(int piece){
//         return switch (piece) {
//             case WHITE_PAWN -> "P";
//             case WHITE_KNIGHT -> "N";
//             case WHITE_BISHOP -> "B";
//             case WHITE_ROOK -> "R";
//             case WHITE_QUEEN -> "Q";
//             case WHITE_KING -> "K";
//             case BLACK_PAWN -> "p";
//             case BLACK_KNIGHT -> "n";
//             case BLACK_BISHOP -> "b";
//             case BLACK_ROOK -> "r";
//             case BLACK_QUEEN -> "q";
//             case BLACK_KING -> "k";
//             default -> " ";
//         };
//
//     }
