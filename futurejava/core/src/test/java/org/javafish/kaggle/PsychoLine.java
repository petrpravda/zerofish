package org.javafish.kaggle;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

public class PsychoLine {
    public record ParsedLine(int gameId, String whiteElo, String blackElo, String pgns) {}

    public static void main(String[] args) {
        //String line = "1 2000.03.14 1-0 2851 None 67 date_false result_false welo_false belo_true edate_true setup_false fen_false result2_false oyrange_false blen_false ### W1.d4 B1.d5 W2.c4 B2.e6 W3.Nc3 B3.Nf6 W4.cxd5 B4.exd5 W5.Bg5 B5.Be7 W6.e3 B6.Ne4 W7.Bxe7 B7.Nxc3 W8.Bxd8 B8.Nxd1 W9.Bxc7 B9.Nxb2 W10.Rb1 B10.Nc4 W11.Bxc4 B11.dxc4 W12.Ne2 B12.O-O W13.Nc3 B13.b6 W14.d5 B14.Na6 W15.Bd6 B15.Rd8 W16.Ba3 B16.Bb7 W17.e4 B17.f6 W18.Ke2 B18.Nc7 W19.Rhd1 B19.Ba6 W20.Ke3 B20.Kf7 W21.g4 B21.g5 W22.h4 B22.h6 W23.Rh1 B23.Re8 W24.f3 B24.Bb7 W25.hxg5 B25.fxg5 W26.d6 B26.Nd5+ W27.Nxd5 B27.Bxd5 W28.Rxh6 B28.c3 W29.d7 B29.Re6 W30.Rh7+ B30.Kg8 W31.Rbh1 B31.Bc6 W32.Rh8+ B32.Kf7 W33.Rxa8 B33.Bxd7 W34.Rh7+ ";
        String line = "8956 1993.??.?? 1/2-1/2 2705 2660 37 date_true result_false welo_false belo_false edate_true setup_false fen_false result2_false oyrange_true blen_false ### W1.e4 B1.c5 W2.Nf3 B2.d6 W3.d4 B3.cxd4 W4.Nxd4 B4.Nf6 W5.Nc3 B5.g6 W6.Be3 B6.Bg7 W7.f3 B7.Nc6 W8.Qd2 B8.O-O W9.O-O-O B9.d5 W10.Qe1 B10.e6 W11.Kb1 B11.Qe7 W12.Nb3 B12.Rd8 W13.Bc5 B13.Qc7 W14.Bb5 B14.dxe4 W15.Rxd8+ B15.Nxd8 W16.Nxe4 B16.Nxe4 W17.Qxe4 B17.Bd7 W18.Bxd7 B18.Qxd7 W19.Bd4 ";
        PsychoLine psychoLine = new PsychoLine();
        Optional<ParsedLine> result = psychoLine.parse(line);
        System.out.println(result);
    }
    // (([\s\S]+? ){3})(\w+) (\w+) (([\s\S]+? ){13})
    private static final Pattern PATTERN = Pattern.compile("(?<id>\\d+)(([\\s\\S]+? ){2})(?<whiteElo>\\w+) (?<blackElo>\\w+) (.+W1\\.)(?<pgns>[\\s\\S]*)");
    //private static final Pattern PATTERN = Pattern.compile("(([\\s\\S]+? ){17})(?<pgns>[\\s\\S]*)");
    public Optional<ParsedLine> parse(String line) {
        if (line.length() > 0) {
            Matcher matcher = PATTERN.matcher(line);
            if (!matcher.find()) {
                //throw new IllegalStateException(String.format("No match found for: %s", line));
                return Optional.empty();
            }

            int gameId = Integer.parseInt(matcher.group("id"));
            String whiteElo = matcher.group("whiteElo");
            String blackElo = matcher.group("blackElo");
            String pgns = matcher.group("pgns");
            //ParsedLine result = new ParsedLine(null, null, trimMoveCounting(pgns));
            ParsedLine result = new ParsedLine(gameId, whiteElo, blackElo, trimMoveCounting(pgns));
            return Optional.of(result);
        } else {
            return Optional.empty();
        }
    }

    //private static final Pattern REGEX_TRIM = Pattern.compile("[WB]\\d+\\.");
    public static String trimMoveCounting(String pgns) {
        return pgns.replaceAll("[WB]\\d+\\.", "");
    }
}
