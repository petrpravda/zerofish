package nnue;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.Arrays;
import java.util.List;

public class Constants {
    public static final short[] INPUT_LAYER_WEIGHT = loadWeights("nnue/input.layer.weights.txt");
//            { // 196608
//    };

    public static final short[] INPUT_LAYER_BIAS = {
        -8, -44, -8, -7, -35, -16, -10, -14, 9, 1, -8, -29, -33, -8, -1, 20, 19, -19, -13, -32, -20,
        -26, -12, -8, 12, -10, -8, -14, -9, -65, 26, 5, -7, -24, -72, -18, -45, 16, -9, -21, -20, 25,
        -9, -33, -19, -24, -22, -19, -36, -11, -9, -37, -11, -32, -3, 7, -5, -21, -8, 8, -9, -15, -11,
        -17, -30, -22, 5, -64, -15, -48, -14, -6, 62, -16, -16, -17, 20, -53, -13, -23, -10, 19, 29,
        -1, -1, -15, 5, -8, -54, -9, 0, -11, -10, -5, -6, 19, -2, -17, -21, -19, -16, 49, -62, -2, 9,
        25, -4, 16, -33, -5, -9, -70, 3, -61, -21, -4, -2, 4, 6, -9, -4, -10, -17, 27, 3, 17, -10, 53,
        -6, 4, -3, -7, -5, -11, 2, -56, -5, -9, -30, -38, 8, -8, -2, -4, 22, -10, -21, -22, -14, -9,
        -1, -4, -7, -14, -19, -3, -8, -34, 18, -10, 29, -43, -11, 26, -13, -25, -17, -20, -13, -3, 8,
        -36, -3, -16, -3, 25, -2, -9, 12, -4, -9, -23, -9, -7, -9, 8, -10, -5, -21, -15, 0, -28, -14,
        0, -29, -13, -21, -2, -75, 0, -34, -25, -24, -19, -8, -20, -21, 10, -43, -21, 5, -11, -3, -35,
        -28, 53, 13, -11, -26, -55, -11, -3, 1, -26, -21, -36, 0, -26, 20, -9, -16, -8, -14, -29, -13,
        4, 0, -16, -48, -25, -50, -34, -47, -18, -30, -11, -33, -14, -16, 39, -30, -10, -11, -8, -9,
        -12,
    };

    public static final short[] L1_WEIGHT = {
        -4, -17, -5, 7, -35, -5, 6, 5, 5, 4, 5, 7, 12, 5, -8, -9, -51, -7, 4, 6, -62, 11, 5, -5, 6, 7,
        -5, -8, -4, -13, -6, -6, -6, 8, -14, -29, -15, -5, -6, 26, -16, 5, 5, -10, 64, 7, -6, -14, 7,
        11, -5, -7, -9, 53, 9, 5, -4, 6, 9, -18, 5, 18, -5, -5, -34, -8, -87, -27, -7, -11, -7, -6, -6,
        -5, 5, -11, 9, 20, 5, -8, -7, 65, -5, 4, 5, -5, 9, -6, 13, -6, 5, 4, -7, 7, -10, 7, -5, -5, 8,
        6, 6, -8, -11, -13, 5, 5, 6, -5, 8, 4, -5, 38, 6, 14, 9, 9, -4, 79, -6, 4, -8, 9, -7, 6, -7, 9,
        -15, -7, -5, -4, -6, -5, 8, -5, -5, 10, 6, 5, -8, 5, -8, 11, 6, 8, 10, 5, -8, 5, -4, -10, 4,
        -4, -5, 26, 6, 4, 8, -7, -8, 6, 6, 7, 5, 6, 6, 39, 7, 6, 5, -5, 4, 8, 4, 8, -5, -5, 5, -16, 4,
        -11, -4, -6, 5, 5, -10, 12, 5, -6, 5, -6, -6, 9, -5, -4, -8, 6, -6, -8, -36, -5, -64, -5, 11,
        5, 7, -5, -8, 5, 15, -20, 15, 5, -7, 9, 6, 9, -8, -6, 10, -9, 5, 5, 5, -10, -11, 10, -5, -7, 9,
        10, -8, 7, 11, 27, 6, 8, -4, 4, -13, -9, 32, -8, -15, -9, 41, -16, 8, 4, -5, -4, 6, -4, 5, -9,
        -4, -4,
    };

    public static final short[] L1_BIAS = {
        -311
    };

    private static short[] convertListToShortArray(List<Short> data) {
        short[] array = new short[data.size()];
        for (int i = 0; i < data.size(); i++) {
            array[i] = data.get(i);
        }
        return array;
    }

    private static short[] loadWeights(String resourcePath) {
        try (InputStream inputStream = Constants.class.getClassLoader().getResourceAsStream(resourcePath);
             BufferedReader reader = new BufferedReader(new InputStreamReader(inputStream))) {
            List<Short> data = reader.lines()
                    .flatMap(line -> Arrays.stream(line.split(",\\s*|\\s+")))
                    .mapToInt(Integer::parseInt)
                    .mapToObj(i -> (short) i)
                    .toList();
            return convertListToShortArray(data);
        } catch (IOException e) {
            throw new RuntimeException("Failed to load weights", e);
        }
    }
}
