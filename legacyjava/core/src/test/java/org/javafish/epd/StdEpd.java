package org.javafish.epd;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class StdEpd {
    public record EpdLine(String fen, String bestMove) {};

    private static final Pattern REGEX = Pattern.compile("(?<fenStart>\\w+(\\/\\w+){7} [wb] [KQkq-]+ [a-h1-8-]+) bm (?<bestMove>[\\w-+]+)");
    public static EpdLine parseLine(String line) {
        Matcher matcher = REGEX.matcher(line);
        if (!matcher.find()) {
            throw new IllegalArgumentException(String.format("Illegal EPD line %s", line));
        } else {
            String fenStart = matcher.group("fenStart");
            String bestMove = matcher.group("bestMove");
            EpdLine epd = new EpdLine(fenStart, bestMove);
            return epd;
        }
    }

    public static void main(String[] args) {
        System.out.println();
    }
}
