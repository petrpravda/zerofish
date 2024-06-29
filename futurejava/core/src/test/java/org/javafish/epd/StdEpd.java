package org.javafish.epd;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class StdEpd {
    public record StsDesc(String section, Integer number) {}

    public record EpdLine(String fen, String bestMove, StsDesc desc) {}

    private static final Pattern REGEX_STS_DESC = Pattern.compile("id \".+? (?<section>.+)\\.(?<number>\\d+)\"");
    private static final Pattern REGEX = Pattern.compile("(?<fenStart>\\w+(\\/\\w+){7} [wb] [KQkq-]+ [a-h1-8-]+) bm (?<bestMove>[\\w-+]+)(?<rest>.+)");
    public static EpdLine parseLine(String line) {
        Matcher matcher = REGEX.matcher(line);
        if (!matcher.find()) {
            throw new IllegalArgumentException(String.format("Illegal EPD line %s", line));
        } else {
            String fenStart = matcher.group("fenStart");
            String bestMove = matcher.group("bestMove");
            String rest = matcher.group("rest");
            StsDesc stsDesc = parseDescriptor(rest);
            return new EpdLine(fenStart, bestMove, stsDesc);
        }
    }

    private static StsDesc parseDescriptor(String rest) {
        Matcher matcher = REGEX_STS_DESC.matcher(rest);
        if (!matcher.find()) {
            throw new IllegalArgumentException();
        }
        String section = matcher.group("section");
        Integer number = Integer.valueOf(matcher.group("number"));
        return new StsDesc(section, number);
    }

    public static void main(String[] args) {
        System.out.println();
    }
}
