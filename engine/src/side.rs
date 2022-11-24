// #![allow(unused)]

pub type Side = u8;

pub const WHITE: Side = 0;
pub const BLACK: Side = 1;

pub fn flip(side: Side) -> Side {
    BLACK ^ side
}

// pub fn multiplicator(side: Side) -> i32 {
//     if side == WHITE { 1 } else { -1 }
// }

pub fn parse(side: char) -> Side {
    match side {
        'w' => WHITE,
        'b' => BLACK,
        _ => panic!()
    }
}

pub fn to_string(side: Side) -> char {
    match side {
        WHITE => 'w',
        BLACK => 'b',
        _ => panic!()
    }
}
