package org.javafish.move;

import java.security.SecureRandom;
import java.util.Random;

import static org.javafish.board.Square.A1;
import static org.javafish.board.Square.H8;

public class Zobrist {
    private static final Random RANDOM = new SecureRandom();
    
    public static long[][] ZOBRIST_TABLE = new long[14][64];
    public static long[] EN_PASSANT = new long[8];
    public static long SIDE = RANDOM.nextLong();

    static {
        for (int piece = 0; piece < 14; piece++) {
            for (int sq = A1; sq <= H8; sq++)
                ZOBRIST_TABLE[piece][sq] = RANDOM.nextLong();
        }

        for (int file = 0; file <= 7; file++)
            EN_PASSANT[file] = RANDOM.nextLong();
    }
}
