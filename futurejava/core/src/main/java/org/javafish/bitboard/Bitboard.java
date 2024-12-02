package org.javafish.bitboard;

import org.javafish.board.PieceType;
import org.javafish.board.Side;
import org.javafish.board.Square;

import java.util.Arrays;
import java.util.stream.IntStream;

import static org.javafish.bitboard.Bitboard.MoveDirection.dir;
import static org.javafish.board.Square.A1;
import static org.javafish.board.Square.H8;

public class Bitboard {
    // https://www.chessprogramming.org/Square_Mapping_Considerations

    public static final long FULL_BOARD = 0b11111111_11111111_11111111_11111111_11111111_11111111_11111111_11111111L;

    private static final long LEFT_PAWN_ATTACK_MASK = 0b11111110_11111110_11111110_11111110_11111110_11111110_11111110_11111110L;
    private static final long RIGHT_PAWN_ATTACK_MASK = 0b1111111_01111111_01111111_01111111_01111111_01111111_01111111_01111111L;

    public static final long LIGHT_SQUARES = 0x55AA55AA55AA55AAL;
    public static final long DARK_SQUARES = 0xAA55AA55AA55AA55L;

    public static final long[] PAWN_DOUBLE_PUSH_LINES = {
            0b00000000_00000000_00000000_00000000_00000000_11111111_00000000_00000000L,
            0b00000000_00000000_11111111_00000000_00000000_00000000_00000000_00000000L,
    };
    public static final long[] PAWN_RANKS = {
            0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000L,
            0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000L,
    };
    public static final long PAWN_FINAL_RANKS = 0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_11111111L;

    // Patterns to check, whether the fields between king and rook are empty
    public static final long BLACK_KING_SIDE_CASTLING_BLOCKERS_PATTERN =
            0b01100000_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;
    public static final long BLACK_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN =
            0b00001110_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;

    public static final long WHITE_KING_SIDE_CASTLING_BLOCKERS_PATTERN =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01100000L;
    public static final long WHITE_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001110L;


    // Patterns to check, whether king and rook squares are not are empty
    public static final long BLACK_KING_SIDE_CASTLING_BIT_PATTERN =
            0b10010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;
    public static final long BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN =
            0b00010001_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;

    public static final long WHITE_KING_SIDE_CASTLING_BIT_PATTERN =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10010000L;
    public static final long WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010001L;

    public static final long WHITE_KINGS_ROOK_MASK =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000000L;
    public static final long WHITE_QUEENS_ROOK_MASK =
            0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00000001L;
    public static final long BLACK_QUEENS_ROOK_MASK =
            0b00000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;
    public static final long BLACK_KINGS_ROOK_MASK =
            0b10000000_00000000_00000000_00000000_00000000_00000000_00000000_00000000L;

    public static final long WHITE_OUTPOST_MASK =
            0b00000000_11111111_11111111_11111111_11111111_00000000_00000000_00000000L;
    public static final long BLACK_OUTPOST_MASK =
            0b00000000_00000000_00000000_11111111_11111111_11111111_11111111_00000000L;

    public static final long LONG_DIAGONALS[] = {
                    0b00000001_00000010_00000100_00001000_00010000_00100000_01000000_10000000L,
                    0b10000000_01000000_00100000_00010000_00001000_00000100_00000010_00000001L};

    public static final int WHITE_KING_INITIAL_SQUARE =
            Long.numberOfTrailingZeros(0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000L);
    public static final int BLACK_KING_INITIAL_SQUARE =
            Long.numberOfTrailingZeros(0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000L);

    public static final long BACK_ROWS = 0b11111111_00000000_00000000_00000000_00000000_00000000_00000000_11111111L;
    public static final long FILE_A = 0b00000001_00000001_00000001_00000001_00000001_00000001_00000001_00000001L;

    public static final LineAttackMask[] LINE_MASKS = calculateLinePatterns();

    public final static long[][] BB_SQUARES_BETWEEN = new long[64][64];
    static {
        long sqs;
        for (int sq1 = A1; sq1 <= H8; sq1++){
            for (int sq2 = A1; sq2 <= H8; sq2++){
                sqs = 1L << sq1 | 1L << sq2;
                if (Square.getFileIndex(sq1) == Square.getFileIndex(sq2) || Square.getRankIndex(sq1) == Square.getRankIndex(sq2)) {
                    BB_SQUARES_BETWEEN[sq1][sq2] =
                            getRookAttacks(sq1, sqs) & getRookAttacks(sq2, sqs);
                }
                else if (Square.getDiagonalIndex(sq1) == Square.getDiagonalIndex(sq1) || Square.getAntiDiagonalIndex(sq1) == Square.getAntiDiagonalIndex(sq2)) {
                    BB_SQUARES_BETWEEN[sq1][sq2] =
                            getBishopAttacks(sq1, sqs) & getBishopAttacks(sq2, sqs);
                }
            }
        }
    }

    public final static long[][] BB_LINES = new long[64][64];
    static {
        for (int sq1 = A1; sq1 <= H8; sq1++){
            for (int sq2 = A1; sq2 <= H8; sq2++){
                if (Square.getFileIndex(sq1) == Square.getFileIndex(sq2) || Square.getRankIndex(sq1) == Square.getRankIndex(sq2))
                    BB_LINES[sq1][sq2] =
                            getRookAttacks(sq1, 0) & getRookAttacks(sq2, 0);
                else if (Square.getDiagonalIndex(sq1) == Square.getDiagonalIndex(sq2) || Square.getAntiDiagonalIndex(sq1) == Square.getAntiDiagonalIndex(sq2))
                    BB_LINES[sq1][sq2] =
                            getBishopAttacks(sq1, 0) & getBishopAttacks(sq2, 0);
            }
        }
    }


    private static final MoveDirection[] KNIGHT_MOVE_DIRECTIONS = new MoveDirection[] {dir(-2, -1), dir(-2, 1), dir(2, -1), dir(2, 1),
            dir(-1, -2), dir(-1, 2), dir(1, -2), dir(1, 2)};
    private static final MoveDirection[] KING_MOVE_DIRECTIONS = new MoveDirection[] {dir(0, -1), dir(1, -1), dir(1, 0), dir(1, 1),
            dir(0, 1), dir(-1, 1), dir(-1, 0), dir(-1, -1)};
    // public static final MoveDirection[] BISHOP_MOVE_DIRECTIONS = new MoveDirection[] {dir(-1, -1), dir(1, -1), dir(1, 1), dir(-1, 1)};

    private final static long[] KNIGHT_ATTACKS = generateAttacks(KNIGHT_MOVE_DIRECTIONS);
    private final static long[] KING_ATTACKS = generateAttacks(KING_MOVE_DIRECTIONS);

    public static long push(long l, int side) {
        return side == Side.WHITE ? l << 8 : l >>> 8;
    }

    public static long stringToBitboard(String bbString) {
        // Normalize the string by removing spaces and newlines
        String cleanString = bbString.replaceAll("\\s+", "");

        // Check if the input string is the correct length (64 characters for an 8x8 board)
        if (cleanString.length() != 64) {
            throw new IllegalArgumentException("Invalid bitboard string length. Expected 64 characters.");
        }

        long bitboard = 0L;

        // Iterate over the string, flipping only ranks (rows), but not files (columns)
        for (int rank = 0; rank < 8; rank++) {
            for (int file = 0; file < 8; file++) {
                char charAtPosition = cleanString.charAt(rank * 8 + file);

                // Calculate the correct bit index by flipping the rank
                int flippedRank = 7 - rank;  // Flip rank, but not file
                int bitIndex = flippedRank * 8 + file;

                if (charAtPosition == 'X') {
                    // Set the bit in the bitboard at the correct position
                    bitboard |= (1L << bitIndex);
                } else if (charAtPosition != '.') {
                    throw new IllegalArgumentException("Invalid character in bitboard string. Only 'X' and '.' are allowed.");
                }
            }
        }

        // Return the populated bitboard as a long value
        return bitboard;
    }

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

    public static long between(int sq1, int sq2){
        return BB_SQUARES_BETWEEN[sq1][sq2];
    }

    public static long line(int sq1, int sq2){
        return BB_LINES[sq1][sq2];
    }

    public static long extractLsb(long bb){
        return bb & (bb - 1);
    }

    public static long ignoreOOODanger(int side){ // TODO prozkoumat co to je
        return side == Side.WHITE ? 0x2 : 0x200000000000000L;
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

    public static String bitboardToFormattedBinary(long bb) {
        return String.format("%64s", Long.toBinaryString(bb))
                .replace(' ', '0')
                .replaceAll("(.{8})(?=.)", "$1_")
                .replaceAll("^", "0b")
                .replaceAll("$", "L");
    }

    public static long getLineAttacks(long occupied, LineAttackMask patterns) {
        //  https://www.chessprogramming.org/Obstruction_Difference
        long lower = patterns.lower & occupied;
        long upper = patterns.upper & occupied;
        long mMs1b = 0x8000000000000000L >> Long.numberOfLeadingZeros (lower | 1);
        long ls1b  = upper & -upper;
        long diff = 2*ls1b + mMs1b;
        return patterns.combined & diff;
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

    public static long getBishopAttacks(int sq, long occ) {
        return getLineAttacks(occ, LINE_MASKS[Directions.Diagonal.maskIndex(sq)])
        | getLineAttacks(occ, LINE_MASKS[Directions.AntiDiagonal.maskIndex(sq)]);
    }

    public static long getRookAttacks(int sq, long occ) {
        return getLineAttacks(occ, LINE_MASKS[Directions.Horizontal.maskIndex(sq)])
                | getLineAttacks(occ, LINE_MASKS[Directions.Vertical.maskIndex(sq)]);
    }

    public static long getRookFileAttacks(int sq, long occ) {
        return getLineAttacks(occ, LINE_MASKS[Directions.Vertical.maskIndex(sq)]);
    }

    public static long getQueenAttacks(int sq, long occ) {
        return getBishopAttacks(sq, occ) | getRookAttacks(sq, occ);
    }

    public static long getKnightAttacks(int sq) {
        return KNIGHT_ATTACKS[sq];
    }


    public static long getKingAttacks(int sq) {
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

    public static long pawnAttacks(int square, int side) {
        long bb = 1L << square;
        if (side == Side.WHITE) {
            return whiteLeftPawnAttacks(bb) | whiteRightPawnAttacks(bb);
        } else {
            return blackLeftPawnAttacks(bb) | blackRightPawnAttacks(bb);
        }
    }

    public static long pawnAttacks(long pawns, int side){
        return side == Side.WHITE ? ((pawns & LEFT_PAWN_ATTACK_MASK) << 7) | ((pawns & RIGHT_PAWN_ATTACK_MASK) << 9) :
                ((pawns & LEFT_PAWN_ATTACK_MASK) >>> 9) | ((pawns & RIGHT_PAWN_ATTACK_MASK) >>> 7);
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


    public static long castlingPiecesKingsideMask(int side){
        return side == Side.WHITE ? WHITE_KING_SIDE_CASTLING_BIT_PATTERN : BLACK_KING_SIDE_CASTLING_BIT_PATTERN;
    }

    public static long castlingPiecesQueensideMask(int side){
        return side == Side.WHITE ? WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN : BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN;
    }

    public static long castlingBlockersKingsideMask(int side){
        return side == Side.WHITE ? WHITE_KING_SIDE_CASTLING_BLOCKERS_PATTERN :
                BLACK_KING_SIDE_CASTLING_BLOCKERS_PATTERN;
    }

    public static long castlingBlockersQueensideMask(int side){
        return side == Side.WHITE ? WHITE_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN :
                BLACK_QUEEN_SIDE_CASTLING_BLOCKERS_PATTERN;
    }



    // . . X . . . . .
    // . . X . . . . .
    // . . X . . . . .
    // . . X . . . . .
    // . . @ . . . . .
    // . . . . . . . .
    // . . . . . . . .
    // . . . . . . . .
//    private long[] create_pawn_free_path_patterns(int direction) {
//        long[] patterns = new long[64];
//        for (int pos = 0; pos < 64; pos++) {
//            int row = pos / 8;
//            int col = pos & 7;
//            long pattern = 0;
//
//            while (row >= 1 && row <= 6) {
//                row += direction;
//                pattern |= 1L << (row * 8 + col);
//            }
//            patterns[pos] = pattern;
//        }
//
//        return patterns;
//    }

    public static long attacks(int pieceType, int square, long occ){
        return switch (pieceType) {
            case PieceType.ROOK -> getRookAttacks(square, occ);
            case PieceType.BISHOP -> getBishopAttacks(square, occ);
            case PieceType.QUEEN -> getBishopAttacks(square, occ) | getRookAttacks(square, occ);
            case PieceType.KING -> getKingAttacks(square);
            case PieceType.KNIGHT -> getKnightAttacks(square);
            default -> 0L;
        };
    }
}
