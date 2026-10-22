//! Bit `n` is square `n`: a1 = 0, h1 = 7, a8 = 56, h8 = 63.
//! See https://www.chessprogramming.org/Square_Mapping_Considerations

use std::sync::LazyLock;

use crate::random::JavaRandom;
use crate::types::*;

const FILE_A: u64 = 0x0101_0101_0101_0101;
const FILE_H: u64 = FILE_A << 7;
const RANK_1: u64 = 0xFF;

/// Indexed by side: the rank reached by a pawn's first single push.
pub const PAWN_DOUBLE_PUSH_LINES: [u64; 2] = [RANK_1 << 16, RANK_1 << 40];
/// Indexed by side: the rank a pawn promotes from.
pub const PAWN_RANKS: [u64; 2] = [RANK_1 << 48, RANK_1 << 8];
pub const PAWN_FINAL_RANKS: u64 = RANK_1 | RANK_1 << 56;

// squares between king and rook that must be empty
const WHITE_KING_SIDE_CASTLING_BLOCKERS: u64 = 0x60; // f1 g1
const WHITE_QUEEN_SIDE_CASTLING_BLOCKERS: u64 = 0x0E; // b1 c1 d1
// king and rook squares that must not have been moved from or to
const WHITE_KING_SIDE_CASTLING_PIECES: u64 = 0x90; // e1 h1
const WHITE_QUEEN_SIDE_CASTLING_PIECES: u64 = 0x11; // a1 e1

pub const WHITE_KINGS_ROOK_MASK: u64 = 1 << 7;
pub const WHITE_QUEENS_ROOK_MASK: u64 = 1;
pub const BLACK_KINGS_ROOK_MASK: u64 = 1 << 63;
pub const BLACK_QUEENS_ROOK_MASK: u64 = 1 << 56;

pub const WHITE_KING_INITIAL_SQUARE: usize = 4;
pub const BLACK_KING_INITIAL_SQUARE: usize = 60;

/// (dx, dy) is the step towards lower squares: horizontal, vertical, diagonal, anti-diagonal.
const DIRECTIONS: [(i32, i32); 4] = [(-1, 0), (0, -1), (1, -1), (-1, -1)];

#[derive(Clone, Copy, Default)]
struct LineMask {
    lower: u64,
    upper: u64,
    both: u64,
}

#[derive(Clone, Copy, Default)]
struct Magic {
    mask: u64,
    magic: u64,
    shift: u32,
    offset: usize,
}

struct Tables {
    line_masks: Vec<LineMask>,
    rook: [Magic; 64],
    bishop: [Magic; 64],
    attacks: Vec<u64>,
    knight: [u64; 64],
    king: [u64; 64],
    between: Vec<u64>,
    line: Vec<u64>,
}

static TABLES: LazyLock<Tables> = LazyLock::new(build_tables);

/// Forces table construction, so the first search does not pay for it.
pub fn init() {
    LazyLock::force(&TABLES);
}

fn build_tables() -> Tables {
    let mut line_masks = Vec::with_capacity(4 * 64);
    for &(dx, dy) in &DIRECTIONS {
        for sq in 0..64 {
            let lower = ray(sq, dx, dy, 7);
            let upper = ray(sq, -dx, -dy, 7);
            line_masks.push(LineMask { lower, upper, both: lower | upper });
        }
    }
    let mut tables = Tables {
        line_masks,
        rook: [Magic::default(); 64],
        bishop: [Magic::default(); 64],
        attacks: Vec::new(),
        knight: leaper_attacks(&[(-2, -1), (-2, 1), (2, -1), (2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2)]),
        king: leaper_attacks(&[(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)]),
        between: vec![0; 64 * 64],
        line: vec![0; 64 * 64],
    };

    // fancy magic bitboards, filled from the obstruction-difference reference
    let mut random = JavaRandom::new(0x5EED);
    let edges_ranks = RANK_1 | RANK_1 << 56;
    let edges_files = FILE_A | FILE_H;
    for sq in 0..64 {
        let rank = RANK_1 << (sq & 56);
        let file = FILE_A << (sq & 7);
        let rook_mask = reference_rook(&tables.line_masks, sq, 0) & !(edges_ranks & !rank) & !(edges_files & !file);
        let bishop_mask = reference_bishop(&tables.line_masks, sq, 0) & !(edges_ranks | edges_files);
        tables.rook[sq] = find_magic(&mut tables, sq, rook_mask, true, &mut random);
        tables.bishop[sq] = find_magic(&mut tables, sq, bishop_mask, false, &mut random);
    }

    for a in 0..64 {
        for b in 0..64 {
            tables.between[a * 64 + b] = shared_attacks(&tables, a, b, 1 << a | 1 << b);
            tables.line[a * 64 + b] = shared_attacks(&tables, a, b, 0);
        }
    }
    tables
}

fn find_magic(tables: &mut Tables, sq: usize, mask: u64, rook: bool, random: &mut JavaRandom) -> Magic {
    let bits = mask.count_ones();
    let size = 1usize << bits;
    let mut occupancies = vec![0u64; size];
    let mut attacks = vec![0u64; size];
    let mut subset = 0u64;
    for index in 0..size {
        occupancies[index] = subset;
        attacks[index] = if rook {
            reference_rook(&tables.line_masks, sq, subset)
        } else {
            reference_bishop(&tables.line_masks, sq, subset)
        };
        subset = subset.wrapping_sub(mask) & mask;
    }
    let mut table = vec![0u64; size];
    let mut used = vec![false; size];
    loop {
        let magic = (random.next_long() & random.next_long() & random.next_long()) as u64;
        if (mask.wrapping_mul(magic) & 0xFF00_0000_0000_0000).count_ones() < 6 {
            continue;
        }
        used.fill(false);
        let mut ok = true;
        for index in 0..size {
            let key = (occupancies[index].wrapping_mul(magic) >> (64 - bits)) as usize;
            if !used[key] {
                used[key] = true;
                table[key] = attacks[index];
            } else if table[key] != attacks[index] {
                ok = false;
                break;
            }
        }
        if ok {
            let offset = tables.attacks.len();
            tables.attacks.extend_from_slice(&table);
            return Magic { mask, magic, shift: 64 - bits, offset };
        }
    }
}

/// https://www.chessprogramming.org/Obstruction_Difference
fn line_attacks(line_masks: &[LineMask], occupied: u64, direction: usize, square: usize) -> u64 {
    let mask = line_masks[direction * 64 + square];
    let lower = mask.lower & occupied;
    let upper = mask.upper & occupied;
    // all bits from the highest blocker below upwards (every bit if there is none)
    let m_ms1b = (i64::MIN >> (lower | 1).leading_zeros()) as u64;
    // lowest blocker above (0 if none)
    let ls1b = upper & upper.wrapping_neg();
    // adding the bit above ls1b carries out everything above it: blocker to blocker stays set
    mask.both & ls1b.wrapping_mul(2).wrapping_add(m_ms1b)
}

fn reference_bishop(line_masks: &[LineMask], sq: usize, occ: u64) -> u64 {
    line_attacks(line_masks, occ, 2, sq) | line_attacks(line_masks, occ, 3, sq)
}

fn reference_rook(line_masks: &[LineMask], sq: usize, occ: u64) -> u64 {
    line_attacks(line_masks, occ, 0, sq) | line_attacks(line_masks, occ, 1, sq)
}

/// Up to `max_steps` squares from `square` in direction (`dx`, `dy`), stopping at the board edge.
fn ray(square: usize, dx: i32, dy: i32, max_steps: i32) -> u64 {
    let file = (square % 8) as i32;
    let rank = (square / 8) as i32;
    let mut result = 0u64;
    for i in 1..=max_steps {
        let (f, r) = (file + i * dx, rank + i * dy);
        if !(0..8).contains(&f) || !(0..8).contains(&r) {
            break;
        }
        result |= 1 << (r * 8 + f);
    }
    result
}

/// Knights and kings: a single step in each of the given (dx, dy) directions.
fn leaper_attacks(steps: &[(i32, i32)]) -> [u64; 64] {
    let mut result = [0u64; 64];
    for (sq, attacks) in result.iter_mut().enumerate() {
        *attacks = steps.iter().fold(0, |acc, &(dx, dy)| acc | ray(sq, dx, dy, 1));
    }
    result
}

/// Squares attacked by both a and b along their common line; the occupancy decides whether the line passes the ends.
fn shared_attacks(tables: &Tables, a: usize, b: usize, occ: u64) -> u64 {
    if magic_attacks(tables, &tables.rook[a], 0) & 1 << b != 0 {
        return magic_attacks(tables, &tables.rook[a], occ) & magic_attacks(tables, &tables.rook[b], occ);
    }
    if magic_attacks(tables, &tables.bishop[a], 0) & 1 << b != 0 {
        return magic_attacks(tables, &tables.bishop[a], occ) & magic_attacks(tables, &tables.bishop[b], occ);
    }
    0
}

#[inline(always)]
fn magic_attacks(tables: &Tables, magic: &Magic, occ: u64) -> u64 {
    let index = ((occ & magic.mask).wrapping_mul(magic.magic) >> magic.shift) as usize;
    // SAFETY: the index is below 2^bits and every square's table holds exactly 2^bits entries
    unsafe { *tables.attacks.get_unchecked(magic.offset + index) }
}

#[inline(always)]
pub fn lsb(bb: u64) -> usize {
    bb.trailing_zeros() as usize
}

#[inline(always)]
pub fn push(bb: u64, side: usize) -> u64 {
    if side == WHITE {
        bb << 8
    } else {
        bb >> 8
    }
}

/// Squares strictly between two squares on a common line, otherwise 0.
#[inline(always)]
pub fn between(sq1: usize, sq2: usize) -> u64 {
    TABLES.between[sq1 * 64 + sq2]
}

/// The whole line through two aligned squares without the squares themselves, otherwise 0.
#[inline(always)]
pub fn line(sq1: usize, sq2: usize) -> u64 {
    TABLES.line[sq1 * 64 + sq2]
}

#[inline(always)]
pub fn bishop_attacks(sq: usize, occ: u64) -> u64 {
    let tables = &*TABLES;
    magic_attacks(tables, &tables.bishop[sq], occ)
}

#[inline(always)]
pub fn rook_attacks(sq: usize, occ: u64) -> u64 {
    let tables = &*TABLES;
    magic_attacks(tables, &tables.rook[sq], occ)
}

#[inline(always)]
pub fn queen_attacks(sq: usize, occ: u64) -> u64 {
    bishop_attacks(sq, occ) | rook_attacks(sq, occ)
}

#[inline(always)]
pub fn knight_attacks(sq: usize) -> u64 {
    TABLES.knight[sq]
}

#[inline(always)]
pub fn king_attacks(sq: usize) -> u64 {
    TABLES.king[sq]
}

pub fn attacks(piece_type: usize, square: usize, occ: u64) -> u64 {
    match piece_type {
        ROOK => rook_attacks(square, occ),
        BISHOP => bishop_attacks(square, occ),
        QUEEN => queen_attacks(square, occ),
        KING => king_attacks(square),
        KNIGHT => knight_attacks(square),
        _ => 0,
    }
}

// A pawn attack is a diagonal step: 7 or 9 bits, masking the origin file stops wrap-around to the other edge.
#[inline(always)]
pub fn white_left_pawn_attacks(pawns: u64) -> u64 {
    (pawns & !FILE_A) << 7
}

#[inline(always)]
pub fn white_right_pawn_attacks(pawns: u64) -> u64 {
    (pawns & !FILE_H) << 9
}

#[inline(always)]
pub fn black_left_pawn_attacks(pawns: u64) -> u64 {
    (pawns & !FILE_A) >> 9
}

#[inline(always)]
pub fn black_right_pawn_attacks(pawns: u64) -> u64 {
    (pawns & !FILE_H) >> 7
}

#[inline(always)]
pub fn pawn_attacks_bb(pawns: u64, side: usize) -> u64 {
    if side == WHITE {
        white_left_pawn_attacks(pawns) | white_right_pawn_attacks(pawns)
    } else {
        black_left_pawn_attacks(pawns) | black_right_pawn_attacks(pawns)
    }
}

#[inline(always)]
pub fn pawn_attacks(square: usize, side: usize) -> u64 {
    pawn_attacks_bb(1 << square, side)
}

#[inline(always)]
pub fn castling_pieces_kingside_mask(side: usize) -> u64 {
    WHITE_KING_SIDE_CASTLING_PIECES << (56 * side)
}

#[inline(always)]
pub fn castling_pieces_queenside_mask(side: usize) -> u64 {
    WHITE_QUEEN_SIDE_CASTLING_PIECES << (56 * side)
}

#[inline(always)]
pub fn castling_blockers_kingside_mask(side: usize) -> u64 {
    WHITE_KING_SIDE_CASTLING_BLOCKERS << (56 * side)
}

#[inline(always)]
pub fn castling_blockers_queenside_mask(side: usize) -> u64 {
    WHITE_QUEEN_SIDE_CASTLING_BLOCKERS << (56 * side)
}

/// b1/b8 must be empty for long castling, but the king does not cross it, so it may be attacked.
#[inline(always)]
pub fn ignore_ooo_danger(side: usize) -> u64 {
    0x2 << (56 * side)
}

/// Prints the 8th rank first.
pub fn bitboard_to_string(bb: u64) -> String {
    let mut result = String::new();
    for row in 0..8 {
        for file in 0..8 {
            result.push_str(if (bb >> (56 - 8 * row + file)) & 1 == 1 { "X " } else { ". " });
        }
        result.push('\n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn magic_lookups_match_reference() {
        let tables = &*TABLES;
        let mut random = JavaRandom::new(7);
        for _ in 0..2000 {
            let occ = (random.next_long() & random.next_long()) as u64;
            for sq in 0..64 {
                assert_eq!(rook_attacks(sq, occ), reference_rook(&tables.line_masks, sq, occ));
                assert_eq!(bishop_attacks(sq, occ), reference_bishop(&tables.line_masks, sq, occ));
            }
        }
    }

    #[test]
    fn between_and_line() {
        assert_eq!(between(A1, H1), 0x7E);
        assert_eq!(between(A1, 9), 0);
        assert_eq!(line(A1, 9) & 1 << 63, 1 << 63);
        assert_eq!(line(A1, 17), 0);
    }
}
