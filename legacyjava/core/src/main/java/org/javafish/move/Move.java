package org.javafish.move;

import org.javafish.board.BoardState;
import org.javafish.board.Piece;
import org.javafish.board.Square;

import java.util.List;
import java.util.stream.Collectors;

import static org.javafish.board.Square.NO_SQUARE;

public class Move {
    public final static int QUIET = 0b0000, DOUBLE_PUSH = 0b0001, OO = 0b0010, OOO = 0b0011,
        CAPTURE = 0b0100,  EN_PASSANT = 0b0101, PROMOTION = 0b1000,
        PR_KNIGHT = 0b1000, PR_BISHOP = 0b1001, PR_ROOK = 0b1010, PR_QUEEN = 0b1011,
        PC_KNIGHT = 0b1100, PC_BISHOP = 0b1101, PC_ROOK = 0b1110, PC_QUEEN = 0b1111, NULL = 0b0111;

    public final static Move NULL_MOVE = new Move(NO_SQUARE, NO_SQUARE, Move.NULL);
    private final int bits;
    private int sortScore;

    public Move(){
        bits = 0;
        sortScore = 0;
    }

    public Move(int m){
        bits = m;
        sortScore = 0;
    }

    public Move(int from, int to){
        bits = (from << 6) | to;
    }

    public Move(int from, int to, int flags){
        bits = (flags << 12) | (from << 6) | to;
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
        return (bits >>> 12) & 0xf;
    }

    public int bits(){
        return bits;
    }

    public int score(){
        return sortScore;
    }

//    public boolean isCapture() {
//        return ((bits >>> 12 ) & CAPTURE) != 0;
//    }

    public boolean isPromotion() {
        return ((bits >>> 12 ) & PROMOTION) != 0;
    }

//    public static Move nullMove(){
//        return new Move(Square.NO_SQUARE, Square.NO_SQUARE, Move.NULL);
//    }

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

    public void addToScore(int score){
        sortScore += score;
    }

    @Override
    public String toString() {
        return isNullMove() ? "NULL_MOVE" : this.uci();
    }

    public boolean isNullMove() {
        return (this.flags() & NULL) == NULL;
    }

    public static List<Move> parseUciMoves(List<String> moves, BoardState state) {
        return moves.stream()
                .map((String str) -> fromUciString(str, state))
                .collect(Collectors.toList());
    }

    public static Move fromUciString(String str, BoardState state) {
        int fromSq = Square.getSquareFromName(str.substring(0, 2));
        int toSq = Square.getSquareFromName(str.substring(2, 4));
        String typeStr = "";
        boolean capturingPromotion = false;
        if (str.length() > 4) {
            typeStr = str.substring(4);
            if (state.pieceAt(toSq) != Piece.NONE) {
                capturingPromotion = true;
            }
        }

        int flags = switch (typeStr) {
            case "q" -> Move.PR_QUEEN;
            case "n" -> Move.PR_KNIGHT;
            case "b" -> Move.PR_BISHOP;
            case "r" -> Move.PR_ROOK;
            default -> Move.QUIET;
        };

        if (capturingPromotion) {
            flags |= Move.CAPTURE;
        }

        return new Move(fromSq, toSq, flags);
    }

//    public static Move fromFirstUciSubstring(String movesDelimitedWithSpace) {
//        String[] moves = movesDelimitedWithSpace.split(" ");
//        return fromUciString(moves[0]);
//    }

    public int getPieceType() {
        return (flags() & 0b11) + 1;
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
