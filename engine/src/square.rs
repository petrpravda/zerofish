// const BOARD_FLIPPING: u8 = 0x38;

pub struct Square {

}

impl Square {
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
}
