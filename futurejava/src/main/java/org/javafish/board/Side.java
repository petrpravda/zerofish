package org.javafish.board;

public class Side {
    public final static int WHITE = 0;
    public final static int BLACK = 1;

    public static int flip(int side){
        return BLACK ^ side;
    }

    public static int multiplicator(int side) {
        return side == WHITE ? 1 : -1;
    }
}
