#![allow(unused)]

pub type Side = u8;

pub const WHITE: Side = 0;
pub const BLACK: Side = 1;

pub fn flip(side: Side) -> Side {
    BLACK ^ side
}


pub fn multiplicator(side: Side) -> i32 {
    if side == WHITE { 1 } else { -1 }
}

