package org.javafish.epd;

import org.javafish.board.BoardPosition;
import org.javafish.pgn.PgnMoves;
import org.javafish.pgn.PgnParser;
import org.javafish.uci.UciTestingDriver;
import org.junit.jupiter.api.Test;

import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.List;
import java.util.Optional;

import static org.javafish.epd.StdEpd.parseLine;
import static org.javafish.pgn.PgnMoves.oneSanToUci;
import static org.junit.jupiter.api.Assertions.assertEquals;

class StdEpdTest {
    @Test
    public void parseEpds() throws Exception {
        InputStream epdStream = StdEpd.class.getResourceAsStream("STS1-STS15_LAN_v3.epd");
        assert epdStream != null;
        int matchingCount = 0;
        List<String> epds = new BufferedReader(new InputStreamReader(epdStream)).lines().toList();
        try (UciTestingDriver driver = UciTestingDriver.forExecutable("stockfish")) {
            driver.cmd("isready", "readyok");
            for (String epdLine : epds) {
                StdEpd.EpdLine epd = parseLine(epdLine);
                driver.uciNewGame();
                String fen = String.format("%s 1 1", epd.fen());
                driver.positionFen(fen);
                Optional<String> result = driver.go(15);

                String uciMove = oneSanToUci(epd.bestMove(), BoardPosition.fromFen(fen));
                if (result.get().equals(uciMove)) {
                    matchingCount++;
                }

                System.out.printf("%s %s %s%n", result, uciMove, epd.fen());
            }
        }

        int totalSize = epds.size();
        System.out.printf("%d / %s%n", matchingCount, totalSize);
    }
}
