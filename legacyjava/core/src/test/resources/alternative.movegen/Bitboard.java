package org.miniyaga.board;

import java.util.Arrays;
import java.util.Iterator;
import java.util.stream.IntStream;

import static org.miniyaga.board.Bitboard.MoveDirection.dir;

public class Bitboard {
    // https://www.chessprogramming.org/Square_Mapping_Considerations

    private static final long LEFT_PAWN_ATTACK_MASK = 0b11111110_11111110_11111110_11111110_11111110_11111110_11111110_11111110L;
    private static final long RIGHT_PAWN_ATTACK_MASK = 0b1111111_01111111_01111111_01111111_01111111_01111111_01111111_01111111L;

    public static final long[] PAWN_DOUBLE_MOVE_LINES = {
            0b00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000L,
            0b00000000_00000000_11111111_00000000_00000000_00000000_00000000_00000000L,
    };
    // Patterns to check, whether the fields between king and rook are empty
    public static final long BLACK_KING_SIDE_CASTLING_BIT_PATTERN =
            0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;
    public static final long BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN =
            0b00001110_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;

    public static final long WHITE_KING_SIDE_CASTLING_BIT_PATTERN =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000L;
    public static final long WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001110L;

    public static final LineAttackMask[] LINE_MASKS = calculateLinePatterns();

    private static final MoveDirection[] KNIGHT_MOVE_DIRECTIONS = new MoveDirection[] {dir(-2, -1), dir(-2, 1), dir(2, -1), dir(2, 1),
            dir(-1, -2), dir(-1, 2), dir(1, -2), dir(1, 2)};
    private static final MoveDirection[] KING_MOVE_DIRECTIONS = new MoveDirection[] {dir(0, -1), dir(1, -1), dir(1, 0), dir(1, 1),
            dir(0, 1), dir(-1, 1), dir(-1, 0), dir(-1, -1)};
    // public static final MoveDirection[] BISHOP_MOVE_DIRECTIONS = new MoveDirection[] {dir(-1, -1), dir(1, -1), dir(1, 1), dir(-1, 1)};

    private final static long[] KNIGHT_ATTACKS = generateAttacks(KNIGHT_MOVE_DIRECTIONS);
    private final static long[] KING_ATTACKS = generateAttacks(KING_MOVE_DIRECTIONS);

    public record SquarePosition(int file, int rank) {
        public static SquarePosition fromSquareIndex(int square) {
            return new SquarePosition(square % 8, square / 8);
        }

        public int toSquareIndex() {
            return this.file + this.rank * 8;
        }

        public SquarePosition moveInDirection(MoveDirection direction) {
            return new SquarePosition(this.file + direction.x, this.rank + direction.y);
        }

        public boolean isOnBoard() {
            return this.file >= 0 && this.file < 8 && this.rank >= 0 && this.rank < 8;
        }
    }

    public record MoveDirection(int x, int y) {
        public static MoveDirection dir(int x, int y) {
            return new MoveDirection(x, y);
        }
    }

    public static int lsb(long bb){
        return Long.numberOfTrailingZeros(bb);
    }

    public static long extractLsb(long bb){
        return bb & (bb - 1);
    }

    public static Iterable<Integer> iter(long target_bb) {
        return new BitboardIter(target_bb);
    }

    public static class BitboardIter implements Iterable<Integer> {
        private long target_bb;

        public BitboardIter(long target_bb) {
            this.target_bb = target_bb;
        }

        @Override
        public Iterator<Integer> iterator() {
            return new Iterator<>() {
                @Override
                public boolean hasNext() {
                    return target_bb != 0;
                }

                @Override
                public Integer next() {
                    int toSq = Bitboard.lsb(target_bb);
                    target_bb = Bitboard.extractLsb(target_bb);
                    return toSq;
                }
            };
        }
    }

    public static void printBitboard(long bb){
        System.out.println(bitboardToString(bb));
    }

    public static String bitboardToString(long bb){
        StringBuilder result = new StringBuilder();
        for (int rank = 56; rank >= 0; rank -= 8){
            for (int file = 0; file < 8; file++){
                result.append(((bb >>> (rank + file)) & 1) == 1 ? "X" : ".").append(" ");
            }
            result.append("\n");
        }
        return result.toString();
    }

//    public long get_vertical_attacks(long occupied, int pos) {
//        return getLineAttacks(occupied, LINE_MASKS[pos + Directions.Vertical.ordinal() * 64]);
//    }

    public static long getLineAttacks(long occupied, LineAttackMask patterns) {
        //  https://www.chessprogramming.org/Obstruction_Difference
        long lower = patterns.lower & occupied;
        long upper = patterns.upper & occupied;
        long mMs1b = 0x8000000000000000L >> Long.numberOfLeadingZeros (lower | 1);
        long ls1b  = upper & -upper;
        long diff = 2*ls1b + mMs1b;
        return patterns.combined & diff;
    }

    public static long getLineAttacks(long occupied, long lower, long upper) {
        long combined = lower | upper;
        lower &= occupied;
        upper &= occupied;
        long mMs1b = 0x8000000000000000L >> Long.numberOfLeadingZeros (lower | 1);
        long ls1b  = upper & -upper;
        long diff = 2*ls1b + mMs1b;
        return combined & diff;
    }

    public record LineAttackMask(long lower, long upper, long combined) {
    }

    public enum Directions {
        Horizontal(-1, 0),
        Vertical(0, -1),
        Diagonal(1, -1),
        AntiDiagonal(-1, -1);

        private final int col;
        private final int row;

        Directions(int col, int row) {
            this.col = col;
            this.row = row;
        }

        public int maskIndex(int square) {
            return this.ordinal() * 64 + square;
        }
    }

    private static LineAttackMask[] calculateLinePatterns() {
        return Arrays.stream(Directions.values())
            .flatMap(dir -> IntStream.range(0, 64)
                    .mapToObj(square -> {
                        long lower = generateRay(square, dir.col, dir.row);
                        long upper = generateRay(square, -dir.col, -dir.row);
                        return new LineAttackMask(lower, upper, upper | lower);
                    })).toArray(LineAttackMask[]::new);
    }

    private static long generateRay(int pos, int directionHorizontal, int directionVertical) {
        int file = pos % 8;
        int rank = pos / 8;
        long pattern = 0;
        for (int i = 0; i < 7; i++) { // max steps
            file += directionHorizontal;
            rank += directionVertical;
            if (file < 0 || file > 7 || rank < 0 || rank > 7) {
                break;
            }
            pattern |= 1L << (rank * 8 | file);
        }

        return pattern;
    }

    public static long getBishopAttacks(int sq, long occ){
        return getLineAttacks(occ, LINE_MASKS[Directions.Diagonal.maskIndex(sq)])
        | getLineAttacks(occ, LINE_MASKS[Directions.AntiDiagonal.maskIndex(sq)]);
    }

    public static long getRookAttacks(int sq, long occ){
        return getLineAttacks(occ, LINE_MASKS[Directions.Horizontal.maskIndex(sq)])
                | getLineAttacks(occ, LINE_MASKS[Directions.Vertical.maskIndex(sq)]);
    }

    public static long getQueenAttacks(int sq, long occ){
        return getBishopAttacks(sq, occ) | getRookAttacks(sq, occ);
    }

    public static long getKnightAttacks(int sq){
        return KNIGHT_ATTACKS[sq];
    }


    public static long getKingAttacks(int sq){
        return KING_ATTACKS[sq];
    }

    public static long whiteLeftPawnAttacks(long pawns) {
        return (pawns & LEFT_PAWN_ATTACK_MASK) << 7;
    }

    public static long whiteRightPawnAttacks(long pawns) {
        return (pawns & RIGHT_PAWN_ATTACK_MASK) << 9;
    }

    public static long blackLeftPawnAttacks(long pawns) {
        return (pawns & LEFT_PAWN_ATTACK_MASK) >>> 9;
    }

    public static long blackRightPawnAttacks(long pawns) {
        return (pawns & RIGHT_PAWN_ATTACK_MASK) >>> 7;
    }

    private static long[] generateAttacks(MoveDirection[] moveDirections) {
        return IntStream.range(0, 64)
                .mapToObj(SquarePosition::fromSquareIndex)
                .mapToLong(position -> Arrays.stream(moveDirections).map(position::moveInDirection)
                    .filter(SquarePosition::isOnBoard)
                    .mapToLong(x -> 1L << x.toSquareIndex())
                    .reduce(0, (a, b) -> a | b))
                .toArray();
    }
}
