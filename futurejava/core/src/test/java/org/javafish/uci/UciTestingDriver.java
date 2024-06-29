package org.javafish.uci;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader;
import java.io.OutputStreamWriter;
import java.util.List;
import java.util.Optional;
import java.util.Scanner;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import java.util.stream.Collectors;

public class UciTestingDriver implements AutoCloseable {
    private String executable;
    private Process process;
    private BufferedReader reader;
    private OutputStreamWriter writer;
    // private BufferedReader readerError;

    private UciTestingDriver() {

    }

    public static UciTestingDriver forExecutable(String executable) {
        UciTestingDriver result = new UciTestingDriver();
        result.executable = executable;
        result.start();
        return result;
    }

    private void start() {
        var pb = new ProcessBuilder("/bin/sh", "-c", executable);
        try {
            pb.redirectErrorStream(true);
            this.process = pb.start();
            pb.redirectErrorStream(true);
            this.reader = new BufferedReader(new InputStreamReader(process.getInputStream()));
            // this.readerError = new BufferedReader(new InputStreamReader(process.getErrorStream()));
            this.writer = new OutputStreamWriter(process.getOutputStream());
            pb.redirectErrorStream(true);
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    @Override
    public void close() throws Exception {
        this.writer.close();
        this.reader.close();
        this.process.destroy();
    }

    public static void main(String[] args) throws Exception {
        try (UciTestingDriver driver = UciTestingDriver.forExecutable("stockfish")) {
            driver.cmd("isready", "readyok");
        }
    }

    private void cmd(String cmd) {
        try {
            writer.write(cmd);
            writer.write("\n");
            writer.flush();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    public String cmd(String cmd, String expected) {
        try {
            writer.write(cmd);
            writer.write("\n");
            writer.flush();
            StringBuilder sb = new StringBuilder();
            while (true) {
                String line = reader.readLine();
//                if (line == null) {
//                    line = readerError.readLine();
//                }
                // System.err.println(line);
                sb.append(line).append("\n");
                if (Pattern.compile(expected).matcher(sb.toString()).find()) {
                //if (sb.toString().indexOf(expected) != -1) {
                    break;
                }
            }
            return sb.toString().trim();
        } catch (IOException e) {
            throw new RuntimeException(e);
        }
    }

    public String isReady() {
        return this.cmd("isready", "readyok");
    }

//    public String isReady(String expected) {
//        return this.cmd("isready", expected);
//    }

    public void uciNewGame() {
        this.cmd("ucinewgame");
        // this.isReady();
    }

    public void positionMoves(String moves) {
        this.cmd(String.format("position startpos moves %s", moves));
    }

    public void positionFen(String fen) {
        this.cmd(String.format("position fen %s", fen));
    }

    public Optional<String> go(int depth) {
        String result = this.cmd(String.format("go depth %d", depth), "bestmove");
        //String result = this.isReady("bestmove");
        Pattern regex = Pattern.compile("bestmove (\\(none\\)|[a-h][1-8][a-h][1-8]\\w?)");
        Matcher match = regex.matcher(result);
        if (!match.find()) {
            throw new IllegalStateException(String.format("Cannot find bestmove in: %s", result));
        } else {
            String bestmove = match.group(1);
            if (bestmove.equals("(none)")) {
                return Optional.empty();
            } else {
                return Optional.of(bestmove);
            }
        }
    }

    public List<ScoredMove> goMulti(int depth) {
        String result = this.cmd(String.format("go depth %d", depth), "bestmove|Segmentation fault");
        //String result = this.isReady("bestmove");
        Pattern regex = Pattern.compile(String.format("info depth %d .+?multipv (\\d+) score (cp|mate) ([\\d-]+).+?time \\d+ \\w+ ([\\w\\d]{4,5})", depth));
        return new Scanner(result).findAll(regex)
                .map(l -> {
                    String score = String.format("%s%s", l.group(2).equals("mate") ? "M" : "", l.group(3));
                    return new ScoredMove(l.group(4), score);
                })
                .collect(Collectors.toList());
    }

    public String display() {
        return this.cmd("d", "Checkers");
    }

    public String getPlayingSide() {
        String display = this.display();
        Pattern regex = Pattern.compile("Fen: \\S+ ([bw])");
        Matcher match = regex.matcher(display);
        if (!match.find()) {
            throw new IllegalStateException(display);
        }
        return match.group(1);
    }

    public void setOption(String name, String value) {
        this.cmd(String.format("setoption name %s value %s", name, value));
    }
    public void setOption(String name, int value) {
        this.setOption(name, Integer.toString(value));
    }

    public Optional<String> getCheckers() {
        Pattern regex = Pattern.compile("Checkers: ?(.+)?");
        Matcher matcher = regex.matcher(this.display());
        if (!matcher.find()) {
            throw new IllegalStateException(this.display());
        }
        if (matcher.group(1) == null) {
            return Optional.empty();
        }
        String checkers = matcher.group(1).trim();
        return checkers.length() > 0 ? Optional.of(checkers) : Optional.empty();
    }
}
