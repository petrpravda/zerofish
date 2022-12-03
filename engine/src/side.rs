// #![allow(unused)]

// pub type Side = u8;
//
// pub const WHITE: Side = 0;
// pub const BLACK: Side = 1;
//
// pub fn flip(side: Side) -> Side {
//     BLACK ^ side
// }
//
// // pub fn multiplicator(side: Side) -> i32 {
// //     if side == WHITE { 1 } else { -1 }
// // }
//
// pub fn parse(side: char) -> Side {
//     match side {
//         'w' => WHITE,
//         'b' => BLACK,
//         _ => panic!()
//     }
// }
//
// pub fn to_string(side: Side) -> char {
//     match side {
//         WHITE => 'w',
//         BLACK => 'b',
//         _ => panic!()
//     }
// }


use std::fmt;
use std::ops::Not;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Side {
    WHITE,
    BLACK,
}

impl Side {
    #[inline(always)]
    pub fn index(self) -> usize {
        self as usize
    }

    #[inline(always)]
    pub fn multiplicator(&self) -> i8 {
        if *self == Self::WHITE {
            1
        } else {
            -1
        }
    }
}

impl From<u8> for Side {
    #[inline(always)]
    fn from(n: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(n) }
    }
}

impl Not for Side {
    type Output = Side;

    #[inline(always)]
    fn not(self) -> Self {
        Side::from((self as u8) ^ 1)
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if *self == Self::WHITE { "w" } else { "b" })
    }
}

impl TryFrom<char> for Side {
    type Error = &'static str;

    fn try_from(value: char) -> Result<Side, Self::Error> {
        match value {
            'w' => Ok(Side::WHITE),
            'b' => Ok(Side::BLACK),
            _ => Err("Allowed sides are 'w' and 'b'."),
        }
    }
}
