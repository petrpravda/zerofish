package org.javafish.board;

public class Square {
    // https://www.chessprogramming.org/Square_Mapping_Considerations


    public final static int FORWARD = 8, BACK = -8, RIGHT = 1, LEFT = -1, FORWARD_RIGHT = FORWARD + RIGHT, FORWARD_LEFT = FORWARD + LEFT,
            DOUBLE_FORWARD = FORWARD + FORWARD;

    public static int A1 =  0, B1 =  1, C1 =  2, D1 =  3, E1 =  4, F1 =  5, G1 =  6, H1 =  7,
            A8 = 56, B8 = 57, C8 = 58, D8 = 59, E8 = 60, F8 = 61, G8 = 62, H8 = 63,
            NO_SQUARE = 64;

    public static String getName(int square){
        //int mirrored = square ^ BOARD_FLIPPING;
        char file = (char) (square & 0b111);
        char rank = (char) ((square & 0b111000) >>> 3);
        file += 'a';
        rank += '1';
        return String.valueOf(file) + rank;
    }

    public static int getSquareFromName(String square){
        char file = square.charAt(0); //char) (square & 0b111);
        char rank = square.charAt(1); //char) ((square & 0b111000) >>> 3);
        file -= 'a';
        rank -= '1';
        return (rank << 3 | file); // ^ BOARD_FLIPPING;
    }

    public static char getFile(int square){
        char file = (char) (square & 0b111);
        file += 'a';
        return file;
    }

    public static char getRank(int square) {
        char rank = (char) ((square & 0b111000) >>> 3);
        rank += '1';
        return rank;
    }

    public static int getRankIndex(int square){
        return square >>> 3;
    }

    public static int getFileIndex(int square){
        return square & 7;
    }

    public static int getDiagonalIndex(int square){
        return 7 + getRankIndex(square) - getFileIndex(square);
    }

    public static int getAntiDiagonalIndex(int square){
        return getRankIndex(square) + getFileIndex(square);
    }

    public static int direction(int direction, int side){
        return side == Side.WHITE ? direction : -direction;
    }
}



