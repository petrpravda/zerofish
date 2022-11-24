use crate::side::Side;

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

  pub fn to_piece_char(piece: Piece) -> char {
    match piece {
      WHITE_PAWN => 'P',
      WHITE_KNIGHT => 'N',
      WHITE_BISHOP => 'B',
      WHITE_ROOK => 'R',
      WHITE_QUEEN => 'Q',
      WHITE_KING => 'K',
      BLACK_PAWN => 'p',
      BLACK_KNIGHT => 'n',
      BLACK_BISHOP => 'b',
      BLACK_ROOK => 'r',
      BLACK_QUEEN => 'q',
      BLACK_KING => 'k',
      NONE => ' ',
      _ => '?',
    }
  }

  pub type PieceType = u8;

  pub const PAWN: PieceType = 0;
  pub const KNIGHT: PieceType = 1;
  pub const BISHOP: PieceType = 2;
  pub const ROOK: PieceType = 3;
  pub const QUEEN: PieceType = 4;
  pub const KING: PieceType = 5;

//
//     public static int flip(int piece) { return piece ^ 8; }
//
    pub fn type_of(piece: Piece) -> PieceType {
         piece & 0b111
    }
//
//     public static int sideOf(int piece){
//         return (piece & 0b1000) >>> 3;
//     }
//
    pub fn make_piece(side: Side, piece_type: PieceType) -> Piece {
        return (side << 3) + piece_type;
    }

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
