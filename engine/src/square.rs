// const BOARD_FLIPPING: u8 = 0x38;

pub struct Square {

}

impl Square {
    pub const A1: u8 =  0;
    pub const B1: u8 =  1;
    pub const C1: u8 =  2;
    pub const D1: u8 =  3;
    pub const E1: u8 =  4;
    pub const F1: u8 =  5;
    pub const G1: u8 =  6;
    pub const H1: u8 =  7;
    pub const A8: u8 = 56;
    pub const B8: u8 = 57;
    pub const C8: u8 = 58;
    pub const D8: u8 = 59;
    pub const E8: u8 = 60;
    pub const F8: u8 = 61;
    pub const G8: u8 = 62;
    pub const H8: u8 = 63;
    pub const NO_SQUARE: u8 = 64;

    pub fn get_square_from_name(square: &str) -> u8 {
        let file = square.chars().nth(0).unwrap() as u8 - b'a';
        let rank = square.chars().nth(1).unwrap() as u8 - b'1';
        return rank << 3 | file;
    }

    pub fn get_name(square: usize) -> String {
        let file = char::from((square as u8 & 0b111) + 97);
        let rank = char::from(((square as u8 & 0b111111) >> 3) + 49);
        let chars = [file, rank];
        chars.iter().collect::<String>()
    }

    pub fn getFileIndex(square: u8) -> u8 {
        square & 7
    }

    pub fn getRankIndex(square: u8) -> u8 {
        square >> 3
    }

    pub fn getDiagonalIndex(square: u8) -> u8 {
        7 + Square::getRankIndex(square) - Square::getFileIndex(square)
    }

    pub fn getAntiDiagonalIndex(square: u8) -> u8 {
        Square::getRankIndex(square) + Square::getFileIndex(square)
    }

}
