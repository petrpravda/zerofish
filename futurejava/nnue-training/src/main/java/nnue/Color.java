package nnue;

public enum Color {
    White(0),
    Black(1);

    private final int value;

    Color(int value) {
        this.value = value;
    }

    public int value() {
        return value;
    }

    public static Color from(byte n) {
        for (Color color : Color.values()) {
            if (color.value == n) return color;
        }
        throw new IllegalArgumentException("Invalid color value");
    }
}

