use std::fmt;
use crate::bitboard::BitIter;
use crate::square::Square;

pub struct Move {
    bits: u32,
    sort_score: u32
}

impl Move {
    pub const QUIET: u8 = 0b0000;
    pub const DOUBLE_PUSH: u8 = 0b0001;
    pub const OO: u8 = 0b0010;
    pub const OOO: u8 = 0b0011;
    pub const CAPTURE: u8 = 0b0100;
    pub const EN_PASSANT: u8 = 0b0101;
    pub const PROMOTION: u8 = 0b1000;
    pub const PR_KNIGHT: u8 = 0b1000;
    pub const PR_BISHOP: u8 = 0b1001;
    pub const PR_ROOK: u8 = 0b1010;
    pub const PR_QUEEN: u8 = 0b1011;
    pub const PC_KNIGHT: u8 = 0b1100;
    pub const PC_BISHOP: u8 = 0b1101;
    pub const PC_ROOK: u8 = 0b1110;
    pub const PC_QUEEN: u8 = 0b1111;
    pub const NULL: u8 = 0b0111;

    pub const NULL_MOVE: Move = Move { bits: 0, sort_score: 0 };

    pub fn new() -> Self { Self{bits: 0, sort_score: 0 }}
    pub fn newFromBits(m: u32) -> Self { Self{bits: m, sort_score: 0 }}
    pub fn newFromTo(from: u8, to: u8) -> Self { Self{bits: (from as u32) << 6 | (to as u32), sort_score: 0 }}
    pub fn newFromFlags(from: u8, to: u8, flags: u8) -> Self {
        Self{bits: (flags as u32) << 12 | (from as u32) << 6 | (to as u32), sort_score: 0 }}

    pub fn to(&self) -> u8 {
        (self.bits & 0x3f) as u8
    }

    pub fn from(&self) -> u8 {
        ((self.bits >> 6) & 0x3f) as u8
    }

    pub fn flags(&self) -> u8 {
        ((self.bits >> 12) & 0xf) as u8
    }

    pub fn score(&self) -> u32 {
        self.sort_score
    }

    pub fn is_promotion(&self) -> bool {
        ((self.bits >> 12) as u8 & Move::PROMOTION) != 0
    }

    // @Override
    // public boolean equals(Object other) {
    // if (other != null && getClass() == other.getClass())
    // return this.bits == ((Move)other).bits();
    // return false;
    // }

    pub fn uci(&self) -> String {
        let promo = match self.flags() {
            Move::PC_BISHOP | Move::PR_BISHOP => "b",
            Move::PC_KNIGHT | Move::PR_KNIGHT => "n",
            Move::PC_ROOK | Move::PR_ROOK => "r",
            Move::PC_QUEEN | Move::PR_QUEEN => "q",
            _ => ""
        };
    //     String promo = switch (this.flags()) {
    //     case Move.PC_BISHOP, Move.PR_BISHOP -> "b";
    //     case Move.PC_KNIGHT, Move.PR_KNIGHT -> "n";
    //     case Move.PC_ROOK, Move.PR_ROOK -> "r";
    //     case Move.PC_QUEEN, Move.PR_QUEEN -> "q";
    //     default -> "";
    // };
    //    Square.getName(this.from()) + Square.getName(this.to()) + promo;
        format!("{}{}{}",
               Square::get_name(self.from() as usize),
               Square::get_name(self.to() as usize),
               promo)
    }

    // public void addToScore(int score){
    // sort_score += score;
    // }
    //
    // @Override
    // public String toString() {
    // return uci();
    // }
    //
    // public static List<Move> parseUciMoves(List<String> moves) {
    // return moves.stream()
    // .map(Move::fromUciString)
    // .collect(Collectors.toList());
    // }
    //
    // public static Move fromUciString(String str) {
    // int fromSq = Square.getSquareFromName(str.substring(0, 2));
    // int toSq = Square.getSquareFromName(str.substring(2, 4));
    // String typeStr = "";
    // if (str.length() > 4)
    // typeStr = str.substring(4);
    //
    // Move move;
    //
    // if (typeStr.equals("q"))
    // move = new Move(fromSq, toSq, Move.PR_QUEEN);
    // else if (typeStr.equals("n"))
    // move = new Move(fromSq, toSq, Move.PR_KNIGHT);
    // else if (typeStr.equals("b"))
    // move = new Move(fromSq, toSq, Move.PR_BISHOP);
    // else if (typeStr.equals("r"))
    // move = new Move(fromSq, toSq, Move.PR_ROOK);
    // else
    // move = new Move(fromSq, toSq, Move.QUIET);
    //
    // return move;
    // }
    //
    // public static Move fromFirstUciSubstring(String movesDelimitedWithSpace) {
    // String[] moves = movesDelimitedWithSpace.split(" ");
    // return fromUciString(moves[0]);
    // }
    //
    // public int getPieceType() {
    // return (flags() & 0b11) + 1;
    // }
    //
    // public int getPieceTypeForSide(int sideToPlay) {
    // return this.getPieceType() + sideToPlay * 8;
    // }
    //
    // public boolean isCastling() {
    // return this.flags() == Move.OO || this.flags() == Move.OOO;
    // }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}


pub struct MoveList {
    pub moves: Vec<Move>,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: Vec::with_capacity(64)
        }
    }

    pub fn makeQuiets(&mut self, from: u8, targets: u64) {
        for to in BitIter(targets) {
            self.moves.push(Move::newFromFlags(from, to as u8, Move::QUIET));
        }
    }

    pub fn makeCaptures(&mut self, from: u8, targets: u64) {
        for to in BitIter(targets) {
            self.moves.push(Move::newFromFlags(from, to as u8, Move::CAPTURE));
        }
    }

    pub fn makePromotionCaptures(&mut self, from: u8, targets: u64) {
        for to in BitIter(targets) {
            self.moves.push(Move::newFromFlags(from, to as u8, Move::PC_QUEEN));
            self.moves.push(Move::newFromFlags(from, to as u8, Move::PC_KNIGHT));
            self.moves.push(Move::newFromFlags(from, to as u8, Move::PC_ROOK));
            self.moves.push(Move::newFromFlags(from, to as u8, Move::PC_BISHOP));
        }
    }

    pub fn makeDoublePushes(&mut self, from: u8, targets: u64) {
        for to in BitIter(targets) {
            self.moves.push(Move::newFromFlags(from, to as u8, Move::DOUBLE_PUSH));
        }
    }

    pub fn add(&mut self, mowe: Move) {
        self.moves.push(mowe);
    }

    pub fn to_string(&self) -> String {
        format!("length: {}", self.moves.len())
    }
}

impl fmt::Display for MoveList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ucis = self.moves.iter().map(|m| m.uci()).collect::<Vec<String>>().join(" ");
        write!(f, "Move list: {}", ucis)
    }
}
