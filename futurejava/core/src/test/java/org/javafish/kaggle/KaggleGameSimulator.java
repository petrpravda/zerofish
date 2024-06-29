package org.javafish.kaggle;

import org.javafish.pgn.PgnMoves;
import org.javafish.pgn.PgnParser;
import org.javafish.uci.UciTestingDriver;

import java.io.BufferedWriter;
import java.io.FileWriter;
import java.io.IOException;
import java.io.Serializable;
import java.security.SecureRandom;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

public class KaggleGameSimulator {
    //private static final UciTestingDriver uciDriver = UciTestingDriver.forExecutable("/home/petr/Downloads/stockfish_15_linux_x64_ssse/stockfish_15_src/src/stockfish");
    public static final int DEPTH = 12;

    private static void extractGameData() throws IOException {
        SecureRandom random = new SecureRandom();
        BufferedWriter writer = new BufferedWriter(new FileWriter("kaggle.2500.nnue.eval.source.txt", true));
        int readCount = 0;
        int highEloCount = 0;
        try (GameReader gameReader = new GameReader()) {
            for (Optional<PsychoLine.ParsedLine> game : gameReader) {
                if (readCount++ % 10000 == 0) {
                    System.out.format("%d, high elo: %d%n", readCount - 1, highEloCount);
                }

                if (game.isPresent()) {
                    PsychoLine.ParsedLine pGame = game.get();
                    try {
                        int whiteElo = Integer.parseInt(pGame.whiteElo());
                        int blackElo = Integer.parseInt(pGame.blackElo());
                        String[] moves = pGame.pgns().split(" ");
                        int MARGIN = 28;
                        int MARGIN_LEFT = 12;
                        if (whiteElo > 2500 && blackElo > 2500 && moves.length > MARGIN && pGame.gameId() > 2799) {
                            int positionIndex = random.nextInt(moves.length - MARGIN) + MARGIN_LEFT;
                            highEloCount++;

                            try {
                                // PgnMoves pgnMoves = PgnParser.fromSan(pGame.pgns());
                                String[] positionMoves = Arrays.copyOfRange(moves, 0, positionIndex);
                                playAgainstUci(pGame.gameId(), writer, String.join(" ", positionMoves));
//                                System.out.println(positionIndex);
//                                System.out.println(moves.length);
//                                System.out.println(pGame.pgns());
//                                System.out.println(asUci);
//                                break;

                            } catch (IllegalStateException e) {
                                System.err.println(e.getMessage());
                            }
                        }
                        //System.out.format("%s, %s%n", game.get().whiteElo(), game.get().blackElo());
                    } catch (NumberFormatException e) {
                    }
                }
            }
        }
    }

    private static void playAgainstUci(int gameId, BufferedWriter writer, String pgns) {
        int additionalMoves = 0;
        String positionMoves;
        String side;
        Optional<String> checkers;
        long started = System.currentTimeMillis();
        try (UciTestingDriver uciDriver = UciTestingDriver.forExecutable("/home/petr/Downloads/stockfish_15_linux_x64_ssse/stockfish_15_src/src/stockfish")) {
        // try (UciTestingDriver uciDriver = UciTestingDriver.forExecutable("stockfish")) {
            PgnMoves pgnMoves = PgnParser.fromSan(pgns);
            positionMoves = pgnMoves.asUci();
            String originalMoves = new String(positionMoves);
            System.out.println(positionMoves);
            uciDriver.isReady();
            // setoption name Threads value 8
            uciDriver.setOption("Threads", "7");
            uciDriver.uciNewGame();
            uciDriver.positionMoves(positionMoves);
            //String display = uciDriver.display();
            side = uciDriver.getPlayingSide();
            System.out.println(side);
            Optional<String> bestmove;
            do {
                uciDriver.positionMoves(positionMoves);
                // System.out.println(positionMoves);
                bestmove = uciDriver.go(DEPTH);
                additionalMoves++;
                if (bestmove.isPresent()) {
                    positionMoves = String.format("%s %s", positionMoves, bestmove.get());
                } else {
                    break;
                }
                if (additionalMoves >= 500) {
                    break;
                }
            } while (true);
            if (additionalMoves >= 500) {
                System.out.format("gameId: %d, too many moves%n", gameId);
            } else {
                side = uciDriver.getPlayingSide();
                checkers = uciDriver.getCheckers();
                if (checkers.isEmpty()) {
                    System.out.format("   %s%n", positionMoves);
                }
                double timeSpent = ((double)(System.currentTimeMillis() - started)) / 1000.;
                String timeSpentString = String.format("%.3f", timeSpent);
                System.out.format("gameId: %d, moves: %d, side: %s, checkers: %s, time: %.3f%n", gameId, additionalMoves, side, checkers, timeSpent);
                if (checkers.isPresent()) {
                    List<String> csvLine = List.of(Integer.toString(gameId), side, timeSpentString, originalMoves, positionMoves);
                            //Arrays.asList(side, timeSpentString, originalMoves, additionalMoves);
                    writer.write(String.join(";", csvLine));
                    writer.write("\n");
                    writer.flush();
                }
            }
        } catch (Exception e) {
            throw new UciEngineException(e);
        }
        //String result = uciDriver.cmd("isready", "readyok");
        //System.out.println(result);
    }

    public static void main(String[] args) throws IOException {
        extractGameData();
        // playAgainstUci("e4 e5 Nf3 Nc6 Bb5 a6 Ba4 Nf6 O-O Be7 Re1 b5 Bb3 d6 c3 O-O h3 Na5 Bc2");
    }
}
