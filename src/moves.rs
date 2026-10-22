use std::fmt;

use crate::bitboard::lsb;
use crate::types::square_name;

/// 16 bits: flags (4) | from (6) | to (6).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Move(pub u16);

impl Move {
    pub const QUIET: u16 = 0x0000;
    pub const DOUBLE_PUSH: u16 = 0x1000;
    pub const OO: u16 = 0x2000;
    pub const OOO: u16 = 0x3000;
    pub const CAPTURE: u16 = 0x4000;
    pub const EN_PASSANT: u16 = 0x5000;
    pub const NULL: u16 = 0x7000;
    pub const PROMOTION: u16 = 0x8000;
    pub const PR_KNIGHT: u16 = 0x8000;
    pub const PR_BISHOP: u16 = 0x9000;
    pub const PR_ROOK: u16 = 0xA000;
    pub const PR_QUEEN: u16 = 0xB000;
    pub const PC_KNIGHT: u16 = 0xC000;
    pub const PC_BISHOP: u16 = 0xD000;
    pub const PC_ROOK: u16 = 0xE000;
    pub const PC_QUEEN: u16 = 0xF000;
    pub const FLAGS_MASK: u16 = 0xF000;

    pub const NULL_MOVE: Move = Move(Move::NULL);

    #[inline(always)]
    pub const fn new(from: usize, to: usize, flags: u16) -> Move {
        Move(flags | (from << 6) as u16 | to as u16)
    }

    #[inline(always)]
    pub const fn from(self) -> usize {
        ((self.0 >> 6) & 0x3f) as usize
    }

    #[inline(always)]
    pub const fn to(self) -> usize {
        (self.0 & 0x3f) as usize
    }

    #[inline(always)]
    pub const fn flags(self) -> u16 {
        self.0 & Move::FLAGS_MASK
    }

    #[inline(always)]
    pub const fn bits(self) -> u16 {
        self.0
    }

    #[inline(always)]
    pub const fn is_promotion(self) -> bool {
        self.0 & Move::PROMOTION != 0
    }

    #[inline(always)]
    pub const fn is_null(self) -> bool {
        self.flags() == Move::NULL
    }

    #[inline(always)]
    pub const fn is_castling(self) -> bool {
        self.flags() == Move::OO || self.flags() == Move::OOO
    }

    /// The promotion piece type, meaningful for promotions only.
    #[inline(always)]
    pub const fn piece_type(self) -> usize {
        (((self.flags() >> 12) & 0b11) + 1) as usize
    }

    pub fn uci(self) -> String {
        let promo = match self.flags() {
            Move::PC_BISHOP | Move::PR_BISHOP => "b",
            Move::PC_KNIGHT | Move::PR_KNIGHT => "n",
            Move::PC_ROOK | Move::PR_ROOK => "r",
            Move::PC_QUEEN | Move::PR_QUEEN => "q",
            _ => "",
        };
        format!("{}{}{}", square_name(self.from()), square_name(self.to()), promo)
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_null() {
            f.write_str("NULL_MOVE")
        } else {
            f.write_str(&self.uci())
        }
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

pub const MAX_MOVES: usize = 256;

#[derive(Clone)]
pub struct MoveList {
    moves: [Move; MAX_MOVES],
    len: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self::new()
    }
}

impl MoveList {
    pub fn new() -> Self {
        MoveList { moves: [Move(0); MAX_MOVES], len: 0 }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline(always)]
    pub fn get(&self, index: usize) -> Move {
        self.moves[index]
    }

    #[inline(always)]
    pub fn add(&mut self, mv: Move) {
        self.moves[self.len] = mv;
        self.len += 1;
    }

    pub fn as_slice(&self) -> &[Move] {
        &self.moves[..self.len]
    }

    pub fn iter(&self) -> impl Iterator<Item = Move> + '_ {
        self.as_slice().iter().copied()
    }

    pub fn contains(&self, mv: Move) -> bool {
        self.as_slice().contains(&mv)
    }

    #[inline(always)]
    fn make(&mut self, from: usize, mut to: u64, flags: u16) {
        while to != 0 {
            self.add(Move::new(from, lsb(to), flags));
            to &= to - 1;
        }
    }

    #[inline(always)]
    pub fn make_q(&mut self, from: usize, to: u64) {
        self.make(from, to, Move::QUIET);
    }

    #[inline(always)]
    pub fn make_c(&mut self, from: usize, to: u64) {
        self.make(from, to, Move::CAPTURE);
    }

    #[inline(always)]
    pub fn make_dp(&mut self, from: usize, to: u64) {
        self.make(from, to, Move::DOUBLE_PUSH);
    }

    pub fn make_pc(&mut self, from: usize, mut to: u64) {
        while to != 0 {
            let sq = lsb(to);
            to &= to - 1;
            self.add(Move::new(from, sq, Move::PC_KNIGHT));
            self.add(Move::new(from, sq, Move::PC_BISHOP));
            self.add(Move::new(from, sq, Move::PC_ROOK));
            self.add(Move::new(from, sq, Move::PC_QUEEN));
        }
    }
}

impl fmt::Debug for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
