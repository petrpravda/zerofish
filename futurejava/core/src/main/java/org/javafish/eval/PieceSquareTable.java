package org.javafish.eval;

import org.javafish.board.EnumPieceType;
import org.javafish.board.Piece;

import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.Objects;
import java.util.Scanner;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

public class PieceSquareTable {
    public static int[][] MGS = new int[Piece.BLACK_KING + 1][];
    public static int[][] EGS = new int[Piece.BLACK_KING + 1][];

    public final static int[] BASIC_MATERIAL_VALUE = {1, 3, 3, 5, 9};

    static {
        InputStream piecesStream = PieceSquareTable.class.getResourceAsStream("piece.square.table.txt");
        String table = new BufferedReader(new InputStreamReader(Objects.requireNonNull(piecesStream))).lines().collect(Collectors.joining("\n"));
        Pattern regex = Pattern.compile("([A-Z]+)\\((\\d+)\\)((\\s*?\\d+){128})\\s*", Pattern.MULTILINE);
        Matcher match = regex.matcher(table);
        while (match.find()) {
            EnumPieceType piece = EnumPieceType.valueOf(match.group(1));
            String values = match.group(3).replaceAll("\\s", " ");
            Scanner line = new Scanner(values);
            MGS[piece.ordinal()] = new int[64];
            MGS[piece.ordinal() + 8] = new int[64];
            EGS[piece.ordinal()] = new int[64];
            EGS[piece.ordinal() + 8] = new int[64];
            for (int rank = 7; rank >= 0; rank--) {
                for (int file = 7; file >= 0 ; file--) {
                    int value = line.nextInt();
                    MGS[piece.ordinal()][rank * 8 + file] = value;
                    MGS[piece.ordinal() + 8][(7 - rank) * 8 + file] = -value;
                }
                for (int file = 7; file >= 0 ; file--) {
                    int value = line.nextInt();
                    EGS[piece.ordinal()][rank * 8 + file] = value;
                    EGS[piece.ordinal() + 8][(7 - rank) * 8 + file] = -value;
                }
            }
        }
    }

    public static void main(String[] args) {
        //System.out.println(Arrays.asList(MGS[0]));
        System.out.println(MGS[1][3]);
    }
}
