//! Piece, side and square encodings shared by the whole engine.

pub const WHITE: usize = 0;
pub const BLACK: usize = 1;

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;

/// Pieces are `side << 3 | type`; slots 6, 7, 14 and 15 are unused by pieces.
pub const WHITE_PAWN: usize = 0;
pub const WHITE_KNIGHT: usize = 1;
pub const WHITE_BISHOP: usize = 2;
pub const WHITE_ROOK: usize = 3;
pub const WHITE_QUEEN: usize = 4;
pub const WHITE_KING: usize = 5;
pub const BLACK_PAWN: usize = 8;
pub const BLACK_KNIGHT: usize = 9;
pub const BLACK_BISHOP: usize = 10;
pub const BLACK_ROOK: usize = 11;
pub const BLACK_QUEEN: usize = 12;
pub const BLACK_KING: usize = 13;
pub const NONE: usize = 14;

#[inline(always)]
pub const fn flip(side: usize) -> usize {
    side ^ 1
}

#[inline(always)]
pub const fn type_of(piece: usize) -> usize {
    piece & 7
}

#[inline(always)]
pub const fn side_of(piece: usize) -> usize {
    (piece & 8) >> 3
}

#[inline(always)]
pub const fn make_piece(side: usize, piece_type: usize) -> usize {
    (side << 3) + piece_type
}

pub fn piece_notation(piece: usize) -> char {
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
        _ => ' ',
    }
}

/// SAN piece letter to piece type; pawns have no letter in SAN but `P` is accepted.
pub fn piece_type_from_san(c: char) -> Option<usize> {
    match c {
        'P' => Some(PAWN),
        'N' => Some(KNIGHT),
        'B' => Some(BISHOP),
        'R' => Some(ROOK),
        'Q' => Some(QUEEN),
        'K' => Some(KING),
        _ => None,
    }
}

// a1 = 0, h1 = 7, a8 = 56, h8 = 63
pub const A1: usize = 0;
pub const C1: usize = 2;
pub const D1: usize = 3;
pub const E1: usize = 4;
pub const F1: usize = 5;
pub const G1: usize = 6;
pub const H1: usize = 7;
pub const A8: usize = 56;
pub const E8: usize = 60;
pub const NO_SQUARE: usize = 64;

pub const FORWARD: i32 = 8;
pub const BACK: i32 = -8;
pub const FORWARD_LEFT: i32 = 7;
pub const FORWARD_RIGHT: i32 = 9;

/// A direction given for white, mirrored for black.
#[inline(always)]
pub const fn direction(direction: i32, side: usize) -> i32 {
    if side == WHITE {
        direction
    } else {
        -direction
    }
}

pub fn square_name(square: usize) -> String {
    let file = (b'a' + (square & 7) as u8) as char;
    let rank = (b'1' + (square >> 3) as u8) as char;
    format!("{file}{rank}")
}

pub fn square_from_name(name: &str) -> Option<usize> {
    let bytes = name.as_bytes();
    if bytes.len() < 2 || !(b'a'..=b'h').contains(&bytes[0]) || !(b'1'..=b'8').contains(&bytes[1]) {
        return None;
    }
    Some(((bytes[1] - b'1') as usize) << 3 | (bytes[0] - b'a') as usize)
}

pub fn file_char(square: usize) -> char {
    (b'a' + (square & 7) as u8) as char
}

pub fn rank_char(square: usize) -> char {
    (b'1' + (square >> 3) as u8) as char
}
