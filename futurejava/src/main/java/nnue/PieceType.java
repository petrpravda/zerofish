package nnue;

import java.util.EnumSet;
import java.util.Iterator;

public enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
    None;

    public int index() {
        return ordinal();
    }

    public static Iterator<PieceType> iter(PieceType start, PieceType end) {
        return EnumSet.range(start, end).iterator();
    }

    public static PieceType from(byte n) {
        for (PieceType pt : PieceType.values()) {
            if (pt.ordinal() == n) return pt;
        }
        throw new IllegalArgumentException("Invalid piece type value");
    }

    @Override
    public String toString() {
        return PIECE_TYPE_STR.substring(ordinal(), ordinal() + 1).trim();
    }

    public static final int N_PIECE_TYPES = 6;
    private static final String PIECE_TYPE_STR = "pnbrqk ";
}
