pub struct Square {

}

impl Square {
    pub fn get_square_from_name(square: &str) -> u8 {
        let mut chars = square.chars();
        let file = chars.nth(0).unwrap() as u8 - b'a';
        let rank = chars.nth(1).unwrap() as u8 - b'1';
        return rank << 3 | file;
    }
}
