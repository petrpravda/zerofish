package nnue;

import java.util.EnumSet;
import java.util.Iterator;

public enum Piece {
    WhitePawn(0b0000),
    WhiteKnight(0b0001),
    WhiteBishop(0b0010),
    WhiteRook(0b0011),
    WhiteQueen(0b0100),
    WhiteKing(0b0101),
    BlackPawn(0b1000),
    BlackKnight(0b1001),
    BlackBishop(0b1010),
    BlackRook(0b1011),
    BlackQueen(0b1100),
    BlackKing(0b1101),
    None(0b1110);

    private final int value;
    private static final String PIECE_STR = "PNBRQK  pnbrqk ";

    Piece(int value) {
        this.value = value;
    }

    public int index() {
        return value - 2 * colorOf().ordinal();
    }

    public Piece flip() {
        return from((byte) (value ^ 0b1000));
    }

    public PieceType typeOf() {
        return PieceType.from((byte) (value & 0b111));
    }

    public Color colorOf() {
        return Color.from((byte) ((value & 0b1000) >> 3));
    }

    public static Piece makePiece(Color color, PieceType pt) {
        return from((byte) ((color.value() << 3) + pt.index()));
    }

    public static Iterator<Piece> iter(Piece start, Piece end) {
        return EnumSet.range(start, end).iterator();
    }

    public static Piece from(byte n) {
        for (Piece piece : Piece.values()) {
            if (piece.value == n) return piece;
        }
        throw new IllegalArgumentException("Invalid piece value");
    }

    public static Piece from(char c) {
        int index = PIECE_STR.indexOf(c);
        if (index == -1) throw new IllegalArgumentException("Piece symbols should be one of \"KQRBNPkqrbnp\"");
        return from((byte) index);
    }

    @Override
    public String toString() {
        return PIECE_STR.substring(value, value + 1).trim();
    }

    public static final int N_PIECES = 12;
}

