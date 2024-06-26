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

    public static Optional<EnumPieceType> fromSan(char c) {
        String string = String.valueOf(c);
        for (EnumPieceType pieceType : EnumPieceType.values()) {
            if (pieceType.code.equals(string)) {
                return Optional.of(pieceType);
            }
        }
        return Optional.empty();
    }
}
