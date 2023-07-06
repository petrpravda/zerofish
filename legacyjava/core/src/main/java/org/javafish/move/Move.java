package org.javafish.move;

import org.javafish.board.BoardState;
import org.javafish.board.Piece;
import org.javafish.board.Square;

import java.util.List;
import java.util.stream.Collectors;

public class Move {
    public final static short
            QUIET =       (short) 0b0000000000000000,
            DOUBLE_PUSH = (short) 0b0001000000000000,
            OO =          (short) 0b0010000000000000,
            OOO =         (short) 0b0011000000000000,
            CAPTURE =     (short) 0b0100000000000000,
            EN_PASSANT =  (short) 0b0101000000000000,
            PROMOTION =   (short) 0b1000000000000000,
            PR_KNIGHT =   (short) 0b1000000000000000,
            PR_BISHOP =   (short) 0b1001000000000000,
            PR_ROOK =     (short) 0b1010000000000000,
            PR_QUEEN =    (short) 0b1011000000000000,
            PC_KNIGHT =   (short) 0b1100000000000000,
            PC_BISHOP =   (short) 0b1101000000000000,
            PC_ROOK =     (short) 0b1110000000000000,
            PC_QUEEN =    (short) 0b1111000000000000,
            FLAGS_MASK =  (short) 0b1111000000000000,
            NULL =        (short) 0b0111000000000000;

    public final static Move NULL_MOVE = new Move(0, 0, Move.NULL);
    private final short bits;

//    public Move(){
//        bits = 0;
//    }

    public Move(short m){
        bits = m;
    }

    public Move(int from, int to){
        bits = (short) ((from << 6) | to);
    }

    public Move(int from, int to, int flags){
        bits = (short) (flags | (from << 6) | to);
    }

    public int to(){
        return bits & 0x3f;
    }

    public int from(){
        return (bits >>> 6) & 0x3f;
    }

//    public int toFrom(){
//        return bits & 0xffff;
//    }

    public int flags() {
        return bits & FLAGS_MASK;
    }

    public short bits(){
        return bits;
    }


//    public boolean isCapture() {
//        return ((bits >>> 12 ) & CAPTURE) != 0;
//    }

    public boolean isPromotion() {
        return (bits & PROMOTION) != 0;
    }

    @Override
    public boolean equals(Object other) {
        if (other != null && getClass() == other.getClass())
            return this.bits == ((Move)other).bits();
        return false;
    }

    public String uci(){
        String promo = switch (this.flags()) {
            case Move.PC_BISHOP, Move.PR_BISHOP -> "b";
            case Move.PC_KNIGHT, Move.PR_KNIGHT -> "n";
            case Move.PC_ROOK, Move.PR_ROOK -> "r";
            case Move.PC_QUEEN, Move.PR_QUEEN -> "q";
            default -> "";
        };
        return Square.getName(this.from()) + Square.getName(this.to()) + promo;
    }

//    public void addToScore(int score){
//        sortScore += score;
//    }

    @Override
    public String toString() {
        return isNullMove() ? "NULL_MOVE" : this.uci();
    }

    public boolean isNullMove() {
        return this.flags() == NULL;
    }

    public static List<Move> parseUciMoves(List<String> moves, BoardState state) {
        return moves.stream()
                .map(Move::fromUciString)
                .collect(Collectors.toList());
    }

//    public static Move fromUciString(String str, BoardState state) {
//        int fromSq = Square.getSquareFromName(str.substring(0, 2));
//        int toSq = Square.getSquareFromName(str.substring(2, 4));
//        String typeStr = "";
//        boolean capturingPromotion = false;
//        if (str.length() > 4) {
//            typeStr = str.substring(4);
//            if (state.pieceAt(toSq) != Piece.NONE) {
//                capturingPromotion = true;
//            }
//        }
//
//        int flags = switch (typeStr) {
//            case "q" -> Move.PR_QUEEN;
//            case "n" -> Move.PR_KNIGHT;
//            case "b" -> Move.PR_BISHOP;
//            case "r" -> Move.PR_ROOK;
//            default -> Move.QUIET;
//        };
//
//        if (capturingPromotion) {
//            flags |= Move.CAPTURE;
//        }
//
//        return new Move(fromSq, toSq, flags);
//    }

    public static Move fromUciString(String str) {
        int fromSq = Square.getSquareFromName(str.substring(0, 2));
        int toSq = Square.getSquareFromName(str.substring(2, 4));
        String typeStr = "";
        if (str.length() > 4)
            typeStr = str.substring(4);

        return switch (typeStr) {
            case "q" -> new Move(fromSq, toSq, Move.PR_QUEEN);
            case "n" -> new Move(fromSq, toSq, Move.PR_KNIGHT);
            case "b" -> new Move(fromSq, toSq, Move.PR_BISHOP);
            case "r" -> new Move(fromSq, toSq, Move.PR_ROOK);
            default -> new Move(fromSq, toSq, Move.QUIET);
        };
    }
//    public static Move fromFirstUciSubstring(String movesDelimitedWithSpace) {
//        String[] moves = movesDelimitedWithSpace.split(" ");
//        return fromUciString(moves[0]);
//    }

    public int getPieceType() {
        return ((flags() >>> 12) & 0b11) + 1;
    }

    public int getPieceTypeForSide(int sideToPlay) {
        return this.getPieceType() + sideToPlay * 8;
    }

    public boolean isCastling() {
        return this.flags() == Move.OO || this.flags() == Move.OOO;
    }

//    public int getPromotionPieceType() {
//    }
}
