package org.javafish.board;

import java.util.Optional;

public enum EnumPieceType {
    PAWN("P"),
    KNIGHT("N"),
    BISHOP("B"),
    ROOK("R"),
    QUEEN("Q"),
    KING("K");


    private final String code;

    EnumPieceType(String code) {
        this.code = code;
    }

//    public static EnumPieceType fromFen(char c) {
//        String string = String.valueOf(c).toUpperCase();
//        for (EnumPieceType pieceType : EnumPieceType.values()) {
//            if (pieceType.code.equals(string)) {
//                return pieceType;
//            }
//        }
//        return null;
//    }

    public static Optional<EnumPieceType> fromSan(char c) {
        String string = String.valueOf(c);
        for (EnumPieceType pieceType : EnumPieceType.values()) {
            if (pieceType.code.equals(string)) {
                return Optional.of(pieceType);
            }
        }
        return Optional.empty();
    }

//    public static String toFen(int piece) {
//        int piece_id = Math.abs(piece);
//        EnumPieceType pieceType = EnumPieceType.values()[piece_id];
//        return piece < 0 ? pieceType.name().toLowerCase() : pieceType.name();
//    }

//    public String getBoardChar() {
//        return this == EMPTY ? " " : this.name();
//    }
}
