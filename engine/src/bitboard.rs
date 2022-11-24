#![allow(dead_code)]

use std::time::Instant;
use crate::bitboard::Direction::{AntiDiagonal, Diagonal, Horizontal, Vertical};
use crate::piece::PieceType;
use crate::side::{Side, WHITE};
use crate::square::Square;

struct Dir(i8, i8);

const KNIGHT_MOVE_DIRECTIONS: [Dir; 8] = [Dir(-2, -1), Dir(-2, 1), Dir(2, -1), Dir(2, 1),
    Dir(-1, -2), Dir(-1, 2), Dir(1, -2), Dir(1, 2)];
const KING_MOVE_DIRECTIONS: [Dir; 8] = [Dir(0, -1), Dir(1, -1), Dir(1, 0), Dir(1, 1),
    Dir(0, 1), Dir(-1, 1), Dir(-1, 0), Dir(-1, -1)];

struct SquarePosition {
    file: i8,
    rank: i8
}

impl SquarePosition {
    fn from_square_index(square: u8) -> Self {
        SquarePosition{ file: (square % 8) as i8, rank: (square / 8) as i8 }
    }

    fn to_square_index(&self) -> u8 {
        (self.file + self.rank * 8) as u8
    }

    fn move_in_direction(&self, direction: &Dir) -> SquarePosition {
        SquarePosition{ file: self.file + direction.0, rank: self.rank + direction.1 }
    }

    fn is_on_board(&self) -> bool {
        return self.file >= 0 && self.file < 8 && self.rank >= 0 && self.rank < 8;
    }
}

pub struct BitIter(pub u64);

impl Iterator for BitIter {
    type Item = u32; // TODO change to u8
    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 {
            return None;
        }

        let pos = self.0.trailing_zeros();
        self.0 ^= 1 << pos as u64;
        Some(pos)
    }
}

#[repr(usize)]
enum Direction {
    Horizontal = 0,
    Vertical = 1,
    Diagonal = 2,
    AntiDiagonal = 3,
}

const MAX_FIELD_DISTANCE: i32 = 7; // maximum distance between two fields on the board

pub const DIRECTIONS: [usize; 4] = [
    Horizontal as usize,
    Vertical as usize,
    Diagonal as usize,
    AntiDiagonal as usize,
];

#[derive(Copy, Clone)]
struct LinePatterns {
    lower: u64,
    upper: u64,
    combined: u64
}

const DIRECTION_COL_OFFSET: [i32; 4] = [-1, 0, 1, -1]; // TODO unify with dir()
const DIRECTION_ROW_OFFSET: [i32; 4] = [0, -1, -1, -1];

const fn calc_line_patterns() -> [LinePatterns; 64 * 4] {
    let mut patterns: [LinePatterns; 64 * 4] = [LinePatterns{lower: 0, upper: 0, combined: 0}; 64 * 4];

    let mut index = 0;
    let mut dir_index = 0;
    while dir_index < DIRECTIONS.len() {
        let diri = DIRECTIONS[dir_index];
        let mut pos = 0;
        while pos < 64 {
            let lower = calc_pattern(pos, DIRECTION_COL_OFFSET[diri], DIRECTION_ROW_OFFSET[diri]);
            let upper = calc_pattern(pos, -DIRECTION_COL_OFFSET[diri], -DIRECTION_ROW_OFFSET[diri]);
            let combined = upper | lower;
            patterns[index] = LinePatterns{lower, upper, combined};
            index += 1;
            pos += 1;
        }
        dir_index += 1;
    }

    patterns
}

const fn calc_pattern(pos: i32, dir_col: i32, dir_row: i32) -> u64 {
    let mut col = pos % 8;
    let mut row = pos / 8;

    let mut pattern: u64 = 0;

    let mut i = 1;
    while i <= MAX_FIELD_DISTANCE {
        col += dir_col;
        row += dir_row;

        if col < 0 || col > 7 || row < 0 || row > 7 {
            break;
        }

        let pattern_index = row * 8 + col;
        pattern |= 1 << pattern_index as u64;

        i += 1;
    }

    pattern
}

pub struct Bitboard {
    king_attacks: [u64; 64],
    knight_attacks: [u64; 64],
    line_masks: [LinePatterns; 64 * 4],
    bb_squares_between: [[u64; 64]; 64],
    bb_lines: [[u64; 64]; 64],
}

impl Bitboard {
    pub fn new() -> Self {
        let start = Instant::now();
        // let WHITE_PAWN_FREEPATH: [u64; 64] = create_pawn_free_path_patterns(-1);
        // let WHITE_PAWN_FREEPATH: [u64; 64] = create_pawn_free_path_patterns(-1);
        // let BLACK_PAWN_FREEPATH: [u64; 64] = create_pawn_free_path_patterns(1);
        // let WHITE_PAWN_FREESIDES: [u64; 64] = create_pawn_free_sides_patterns(-1);
        // let BLACK_PAWN_FREESIDES: [u64; 64]= create_pawn_free_sides_patterns(1);
        // let WHITE_KING_SHIELD: [u64; 64]= create_king_shield_patterns(-1);
        // let BLACK_KING_SHIELD: [u64; 64]= create_king_shield_patterns(1);
        // let KING_DANGER_ZONE: [u64; 64]= create_king_danger_zone_patterns();
        // let line_masks: [LinePatterns; 64 * 4] = calc_line_patterns();
        //
        let mut result = Self {
            king_attacks: Bitboard::generate_attacks(KING_MOVE_DIRECTIONS),
            knight_attacks: Bitboard::generate_attacks(KNIGHT_MOVE_DIRECTIONS),
            line_masks: calc_line_patterns(),
            bb_squares_between: [[0; 64]; 64],
            bb_lines: [[0; 64]; 64],
            // BLACK_PAWN_FREEPATH,
            // WHITE_PAWN_FREESIDES,
            // BLACK_PAWN_FREESIDES,
            // WHITE_KING_SHIELD,
            // BLACK_KING_SHIELD,
            // KING_DANGER_ZONE,
            // line_masks
        };
        (result.bb_squares_between, result.bb_lines) = result.calc_squares_between();

        let end = Instant::now();

        let duration = end.duration_since(start);
        let msg = format!("Bitboard initialized in {} μs", duration.as_micros());
        //let _msgRef = msg.as_str();
        eprintln!("{}", msg);

        // let elapsed_time = start.elapsed();
        // eprintln!("Bitboard initialized in {} μs", elapsed_time.unwrap().as_micros());
        result
    }


    fn generate_attacks(move_directions: [Dir; 8]) -> [u64; 64] {
        let result = (0u8..64)
            .map(|square| SquarePosition::from_square_index(square))
            .map(|sp| {
                let res = move_directions.iter().map(|md| sp.move_in_direction(md))
                    .filter(|sp| sp.is_on_board())
                    .map(|sp| 1u64 << sp.to_square_index())
                    .reduce(|a, b| a|b)
                    .unwrap();
                return res;
            })
            .collect::<Vec<u64>>();
        let res2: [u64; 64] = result.try_into().unwrap();
        res2
    }


    //     pub const FULL_BOARD = 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111L;
    //
    pub const LEFT_PAWN_ATTACK_MASK: u64 = 0b11111110_11111110_11111110_11111110_11111110_11111110_11111110_11111110;
    pub const RIGHT_PAWN_ATTACK_MASK: u64 = 0b1111111_01111111_01111111_01111111_01111111_01111111_01111111_01111111;
    //
    //     pub const LIGHT_SQUARES = 0x55AA55AA55AA55AAL;
    //     pub const DARK_SQUARES = 0xAA55AA55AA55AA55L;
    //
    // //    WHITE_PAWN_FREEPATH = create_pawn_free_path_patterns(-1);
    // //    BLACK_PAWN_FREEPATH = create_pawn_free_path_patterns(1);
    //
    //
    pub const PAWN_DOUBLE_PUSH_LINES: [u64; 2] = [
            0b00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000,
            0b00000000_00000000_11111111_00000000_00000000_00000000_00000000_00000000,
    ];
    pub const PAWN_RANKS: [u64; 2] = [
            0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000,
            0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000,
    ];
    pub const PAWN_FINAL_RANKS: u64 = 0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_11111111;

    // Patterns to check, whether the fields between king and rook are empty
    pub const BLACK_KING_SIDE_CASTLING_BLOCKERS_PATTERN: u64 =
            0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
    pub const BLACK_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN: u64 =
            0b00001110_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

    pub const WHITE_KING_SIDE_CASTLING_BLOCKERS_PATTERN: u64 =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000;
    pub const WHITE_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN: u64 =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001110;


    // Patterns to check, whether king and rook squares are not are empty
    pub const BLACK_KING_SIDE_CASTLING_BIT_PATTERN: u64 =
            0b10010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
    pub const BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN: u64 =
            0b00010001_00000000_00000000_00000000_00000000_00000000_00000000_00000000;

    pub const WHITE_KING_SIDE_CASTLING_BIT_PATTERN: u64 =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10010000;
    pub const WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN: u64 =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010001;

    pub const WHITE_KINGS_ROOK_MASK: u64 =
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000000;
    pub const  WHITE_QUEENS_ROOK_MASK: u64 =
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001;
    pub const  BLACK_QUEENS_ROOK_MASK: u64 =
                0b00000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
    pub const  BLACK_KINGS_ROOK_MASK: u64 =
                0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000;
    //
    //     pub const WHITE_OUTPOST_MASK =
    //             0b00000000_11111111_11111111_11111111_11111111_00000000_00000000_00000000L;
    //     pub const BLACK_OUTPOST_MASK =
    //             0b00000000_00000000_00000000_11111111_11111111_11111111_11111111_00000000L;
    //
    //     pub const LONG_DIAGONALS[] = {
    //                     0b00000001_00000010_00000100_00001000_00010000_00100000_01000000_10000000L,
    //                     0b10000000_01000000_00100000_00010000_00001000_00000100_00000010_00000001L};
    //

        pub const WHITE_KING_INITIAL_SQUARE: usize = (0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000 as u64).trailing_zeros() as usize;
        pub const BLACK_KING_INITIAL_SQUARE: usize = (0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000 as u64).trailing_zeros() as usize;


    //
    //     pub const BACK_ROWS = 0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_11111111L;
    //     pub const FILE_A = 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001L;
    //
    //     public static final LineAttackMask[] line_masks = calculateLinePatterns();
    //
    //     public final static long[][] bb_squares_between = new long[64][64];
    //     static {
    //         long sqs;
    //         for (int sq1 = A1; sq1 <= H8; sq1++){
    //             for (int sq2 = A1; sq2 <= H8; sq2++){
    //                 sqs = 1L << sq1 | 1L << sq2;
    //                 if (Square.get_file_index(sq1) == Square.get_file_index(sq2) || Square.get_rank_index(sq1) == Square.get_rank_index(sq2)) {
    //                     bb_squares_between[sq1][sq2] =
    //                             getRookAttacks(sq1, sqs) & getRookAttacks(sq2, sqs);
    //                 }
    //                 else if (Square.get_diagonal_index(sq1) == Square.get_diagonal_index(sq1) || Square.get_anti_diagonal_index(sq1) == Square.get_anti_diagonal_index(sq2)) {
    //                     bb_squares_between[sq1][sq2] =
    //                             getBishopAttacks(sq1, sqs) & getBishopAttacks(sq2, sqs);
    //                 }
    //             }
    //         }
    //     }
    //
    //     public final static long[][] bb_lines = new long[64][64];
    //     static {
    //         for (int sq1 = A1; sq1 <= H8; sq1++){
    //             for (int sq2 = A1; sq2 <= H8; sq2++){
    //                 if (Square.get_file_index(sq1) == Square.get_file_index(sq2) || Square.get_rank_index(sq1) == Square.get_rank_index(sq2))
    //                     bb_lines[sq1][sq2] =
    //                             getRookAttacks(sq1, 0) & getRookAttacks(sq2, 0);
    //                 else if (Square.get_diagonal_index(sq1) == Square.get_diagonal_index(sq2) || Square.get_anti_diagonal_index(sq1) == Square.get_anti_diagonal_index(sq2))
    //                     bb_lines[sq1][sq2] =
    //                             getBishopAttacks(sq1, 0) & getBishopAttacks(sq2, 0);
    //             }
    //         }
    //     }
    //
    //
    //     private static final MoveDirection[] KNIGHT_MOVE_DIRECTIONS = new MoveDirection[] {dir(-2, -1), dir(-2, 1), dir(2, -1), dir(2, 1),
    //             dir(-1, -2), dir(-1, 2), dir(1, -2), dir(1, 2)};
    //     private static final MoveDirection[] KING_MOVE_DIRECTIONS = new MoveDirection[] {dir(0, -1), dir(1, -1), dir(1, 0), dir(1, 1),
    //             dir(0, 1), dir(-1, 1), dir(-1, 0), dir(-1, -1)};
    //     // public static final MoveDirection[] BISHOP_MOVE_DIRECTIONS = new MoveDirection[] {dir(-1, -1), dir(1, -1), dir(1, 1), dir(-1, 1)};
    //
    //     private final static long[] knight_attacks = generate_attacks(KNIGHT_MOVE_DIRECTIONS);
    //     private final static long[] king_attacks = generate_attacks(KING_MOVE_DIRECTIONS);

    pub fn push(l: u64, side: Side) -> u64 {
        match side {
            WHITE => l << 8,
            _ => l >> 8
        }
    }

    //     public record SquarePosition(int file, int rank) {
    //         public static SquarePosition from_square_index(int square) {
    //             return new SquarePosition(square % 8, square / 8);
    //         }
    //
    //         public int to_square_index() {
    //             return this.file + this.rank * 8;
    //         }
    //
    //         public SquarePosition move_in_direction(MoveDirection direction) {
    //             return new SquarePosition(this.file + direction.x, this.rank + direction.y);
    //         }
    //
    //         public boolean is_on_board() {
    //             return this.file >= 0 && this.file < 8 && this.rank >= 0 && this.rank < 8;
    //         }
    //     }
    //
    //     public record MoveDirection(int x, int y) {
    //         public static MoveDirection dir(int x, int y) {
    //             return new MoveDirection(x, y);
    //         }
    //     }
    //
    //     public static int lsb(long bb){
    //         return Long.numberOfTrailingZeros(bb);
    //     }
    //
    pub fn between(&self, sq1: u8, sq2: u8) -> u64 {
        return self.bb_squares_between[sq1 as usize][sq2 as usize];
    }

    pub fn line(&self, sq1: u8, sq2: u8) -> u64 {
         self.bb_lines[sq1 as usize][sq2 as usize]
    }

    //     public static long extractLsb(long bb){
    //         return bb & (bb - 1);
    //     }
    //
        pub fn ignore_ooo_danger(side: Side) -> u64 {
            match side { WHITE => 0x2,
                _ => 0x200000000000000 }
        }

    //     public static String bitboardToString(long bb){
    //         StringBuilder result = new StringBuilder();
    //         for (int rank = 56; rank >= 0; rank -= 8){
    //             for (int file = 0; file < 8; file++){
    //                 result.append(((bb >>> (rank + file)) & 1) == 1 ? "X" : ".").append(" ");
    //             }
    //             result.append("\n");
    //         }
    //         return result.toString();
    //     }
    //

    fn get_line_attacks(occupied: u64, patterns: &LinePatterns) -> u64 {
        // Uses the obstruction difference algorithm to determine line attacks
        // https://www.chessprogramming.org/Obstruction_Difference
        let lower = patterns.lower & occupied;
        let upper = patterns.upper & occupied;
        let ms1b = 0x8000000000000000 >> ((lower | 1).leading_zeros() as u64);
        let ls1b = upper & upper.wrapping_neg();
        let odiff = ls1b.wrapping_shl(1).wrapping_sub(ms1b);
        patterns.combined & odiff

        // TODO
        // odiff = upper ^ (upper - ms1B); // Daniel Infuehr's improvement
        // return pMask->lineEx & odiff; // (pMask->lower | pMask->upper) & odiff;
    }

    //     public static long getLineAttacks(long occupied, LineAttackMask patterns) {
    //         //  https://www.chessprogramming.org/Obstruction_Difference
    //         long lower = patterns.lower & occupied;
    //         long upper = patterns.upper & occupied;
    //         long mMs1b = 0x8000000000000000L >> Long.numberOfLeadingZeros (lower | 1);
    //         long ls1b  = upper & -upper;
    //         long diff = 2*ls1b + mMs1b;
    //         return patterns.combined & diff;
    //     }
    //
    //     public static long getLineAttacks(long occupied, long lower, long upper) {
    //         long combined = lower | upper;
    //         lower &= occupied;
    //         upper &= occupied;
    //         long mMs1b = 0x8000000000000000L >> Long.numberOfLeadingZeros (lower | 1);
    //         long ls1b  = upper & -upper;
    //         long diff = 2*ls1b + mMs1b;
    //         return combined & diff;
    //     }
    //


    //     public record LineAttackMask(long lower, long upper, long combined) {
    //     }
    //
    //     public enum Directions {
    //         Horizontal(-1, 0),
    //         Vertical(0, -1),
    //         Diagonal(1, -1),
    //         AntiDiagonal(-1, -1);
    //
    //         private final int col;
    //         private final int row;
    //
    //         Directions(int col, int row) {
    //             this.col = col;
    //             this.row = row;
    //         }
    //
    //         public int maskIndex(int square) {
    //             return this.ordinal() * 64 + square;
    //         }
    //     }
    //
    //     private static LineAttackMask[] calculateLinePatterns() {
    //         return Arrays.stream(Directions.values())
    //             .flatMap(dir -> IntStream.range(0, 64)
    //                     .mapToObj(square -> {
    //                         long lower = generateRay(square, dir.col, dir.row);
    //                         long upper = generateRay(square, -dir.col, -dir.row);
    //                         return new LineAttackMask(lower, upper, upper | lower);
    //                     })).toArray(LineAttackMask[]::new);
    //     }
    //
    //     private static long generateRay(int pos, int directionHorizontal, int directionVertical) {
    //         int file = pos % 8;
    //         int rank = pos / 8;
    //         long pattern = 0;
    //         for (int i = 0; i < 7; i++) { // max steps
    //             file += directionHorizontal;
    //             rank += directionVertical;
    //             if (file < 0 || file > 7 || rank < 0 || rank > 7) {
    //                 break;
    //             }
    //             pattern |= 1L << (rank * 8 | file);
    //         }
    //
    //         return pattern;
    //     }
    //

    pub fn get_bishop_attacks(&self, sq: usize, occupied: u64) -> u64 {
        Bitboard::get_line_attacks(occupied, unsafe { self.line_masks.get_unchecked(sq + (Diagonal as usize * 64)) })
            | Bitboard::get_line_attacks(occupied, unsafe { self.line_masks.get_unchecked(sq as usize + (AntiDiagonal as usize * 64)) })
    }

    pub fn get_rook_attacks(&self, sq: usize, occupied: u64) -> u64 {
        Bitboard::get_line_attacks(occupied, unsafe { self.line_masks.get_unchecked(sq as usize + (Horizontal as usize * 64)) })
            | Bitboard::get_line_attacks(occupied, unsafe { self.line_masks.get_unchecked(sq as usize + (Vertical as usize * 64)) })
    }


    //     public static long getRookAttacks(int sq, long occ) {
    //         return getLineAttacks(occ, line_masks[Directions.Horizontal.maskIndex(sq)])
    //                 | getLineAttacks(occ, line_masks[Directions.Vertical.maskIndex(sq)]);
    //     }
    //
    //     public static long getRookFileAttacks(int sq, long occ) {
    //         return getLineAttacks(occ, line_masks[Directions.Vertical.maskIndex(sq)]);
    //     }
    //
    //     public static long getQueenAttacks(int sq, long occ) {
    //         return getBishopAttacks(sq, occ) | getRookAttacks(sq, occ);
    //     }
    //
        pub fn get_knight_attacks(&self, sq: usize) -> u64 {
            return self.knight_attacks[sq];
        }

        pub fn get_king_attacks(&self, sq: usize) -> u64 {
            return self.king_attacks[sq];
        }

        pub fn white_left_pawn_attacks(pawns: u64) -> u64 {
            return (pawns & Bitboard::LEFT_PAWN_ATTACK_MASK) << 7;
        }

        pub fn white_right_pawn_attacks(pawns: u64) -> u64 {
            return (pawns & Bitboard::RIGHT_PAWN_ATTACK_MASK) << 9;
        }

        pub fn black_left_pawn_attacks(pawns: u64) -> u64 {
            return (pawns & Bitboard::LEFT_PAWN_ATTACK_MASK) >> 9;
        }

        pub fn black_right_pawn_attacks(pawns: u64) -> u64 {
            return (pawns & Bitboard::RIGHT_PAWN_ATTACK_MASK) >> 7;
        }

        pub fn pawn_attacks_from_square(square: u8, side: Side) -> u64 {
            let bb = 1u64 << square;
            match side {
                WHITE => Bitboard::white_left_pawn_attacks(bb) | Bitboard::white_right_pawn_attacks(bb),
                _ => Bitboard::black_left_pawn_attacks(bb) | Bitboard::black_right_pawn_attacks(bb)
            }
        }

        pub fn pawn_attacks(pawns: u64, side: Side) -> u64 {
            match side {
                WHITE => ((pawns & Bitboard::LEFT_PAWN_ATTACK_MASK) << 7) | ((pawns & Bitboard::RIGHT_PAWN_ATTACK_MASK) << 9),
                _ => ((pawns & Bitboard::LEFT_PAWN_ATTACK_MASK) >> 9) | ((pawns & Bitboard::RIGHT_PAWN_ATTACK_MASK) >> 7)
            }
        }

        pub fn castling_pieces_kingside_mask(side: Side) -> u64 {
            match side {
                WHITE => Bitboard::WHITE_KING_SIDE_CASTLING_BIT_PATTERN,
                _ => Bitboard::BLACK_KING_SIDE_CASTLING_BIT_PATTERN
            }
        }

        pub fn castling_pieces_queenside_mask(side: Side) -> u64 {
            match side {
                WHITE => Bitboard::WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN,
                _ => Bitboard::BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN
            }
        }

        pub fn castling_blockers_kingside_mask(side: Side) -> u64 {
            match side { WHITE => Bitboard::WHITE_KING_SIDE_CASTLING_BLOCKERS_PATTERN,
                    _ => Bitboard::BLACK_KING_SIDE_CASTLING_BLOCKERS_PATTERN }
        }

        pub fn castling_blockers_queenside_mask(side: Side) -> u64 {
            match side { WHITE => Bitboard::WHITE_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN,
                _ => Bitboard::BLACK_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN }
        }


    //
    //     // . . X . . . . .
    //     // . . X . . . . .
    //     // . . X . . . . .
    //     // . . X . . . . .
    //     // . . @ . . . . .
    //     // . . . . . . . .
    //     // . . . . . . . .
    //     // . . . . . . . .
    // //    private long[] create_pawn_free_path_patterns(int direction) {
    // //        long[] patterns = new long[64];
    // //        for (int pos = 0; pos < 64; pos++) {
    // //            int row = pos / 8;
    // //            int col = pos & 7;
    // //            long pattern = 0;
    // //
    // //            while (row >= 1 && row <= 6) {
    // //                row += direction;
    // //                pattern |= 1L << (row * 8 + col);
    // //            }
    // //            patterns[pos] = pattern;
    // //        }
    // //
    // //        return patterns;
    // //    }

    pub fn attacks(&self, piece_type: PieceType, square: u8, occ: u64) -> u64 {
        match piece_type {
            PieceType::ROOK => self.get_rook_attacks(square as usize, occ),
            PieceType::BISHOP => self.get_bishop_attacks(square as usize, occ),
            PieceType::QUEEN => self.get_bishop_attacks(square as usize, occ) | self.get_rook_attacks(square as usize, occ),
            PieceType::KING => self.get_king_attacks(square as usize),
            PieceType::KNIGHT => self.get_knight_attacks(square as usize),
            _ => 0u64
        }
    }

    fn calc_squares_between(&self) -> ([[u64; 64]; 64], [[u64; 64]; 64]) {
        let mut result_between = [[0; 64]; 64];
        let mut result_lines = [[0; 64]; 64];
        for sq1 in Square::A1..=Square::H8 {
            for sq2 in Square::A1..=Square::H8 {
                let sqs = 1u64 << sq1 | 1u64 << sq2;
                if Square::get_file_index(sq1) == Square::get_file_index(sq2)
                    || Square::get_rank_index(sq1) == Square::get_rank_index(sq2) {
                    result_between[sq1 as usize][sq2 as usize] =
                        self.get_rook_attacks(sq1 as usize, sqs) & self.get_rook_attacks(sq2 as usize, sqs);
                    result_lines[sq1 as usize][sq2 as usize] = self.get_rook_attacks(sq1 as usize, 0) & self.get_rook_attacks(sq2 as usize, 0);
                }
                else if Square::get_diagonal_index(sq1) == Square::get_diagonal_index(sq1)
                    || Square::get_anti_diagonal_index(sq1) == Square::get_anti_diagonal_index(sq2) {
                    result_between[sq1 as usize][sq2 as usize] =
                        self.get_bishop_attacks(sq1 as usize, sqs) & self.get_bishop_attacks(sq2 as usize, sqs);
                    result_lines[sq1 as usize][sq2 as usize] = self.get_bishop_attacks(sq1 as usize, 0) & self.get_bishop_attacks(sq2 as usize, 0);
                }
            }
        }
        (result_between, result_lines)
    }
}
