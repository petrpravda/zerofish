package org.javafish.epd;

import com.google.common.base.Strings;
import com.google.common.collect.Iterables;
import com.google.common.collect.Lists;
import org.javafish.board.BoardPosition;
import org.javafish.uci.UciTestingDriver;
import org.junit.jupiter.api.Test;

import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.util.*;
import java.util.concurrent.*;
import java.util.function.Function;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import static java.util.stream.Collectors.groupingBy;
import static java.util.stream.Collectors.toMap;
import static org.javafish.epd.StdEpd.parseLine;
import static org.javafish.pgn.PgnMoves.oneSanToUci;

class StdEpdTest {
    public record StsResult(String section, Integer number, Integer outcome) {}

    @Test
    public void parseEpds() throws Exception {
        String workingDir = System.getProperty("user.dir");

//        String executable = "stockfish";
//        String engineName = "sf11";
        String executable = String.format("java -classpath %s/target/classes org.javafish.app.Main", workingDir);
        String engineName = "javafish";
        int depth = 12;
        int coresCount = Runtime.getRuntime().availableProcessors();
        InputStream epdStream = StdEpd.class.getResourceAsStream("STS1-STS15_LAN_v3.epd");
        assert epdStream != null;
        // AtomicInteger matchingCount = new AtomicInteger(0);
        long start = System.currentTimeMillis();
        //List<String> epds = new BufferedReader(new InputStreamReader(epdStream)).lines().limit(120).toList();
        List<String> epds = new BufferedReader(new InputStreamReader(epdStream)).lines().toList();

        class EpdChunkTask implements Callable<List<StsResult>> {
            private final List<String> epds;

            public EpdChunkTask(List<String> epds) {
                this.epds = epds;
            }

            public List<StsResult> call() throws Exception {
                List<StsResult> resultList = new ArrayList<>();
                try (UciTestingDriver driver = UciTestingDriver.forExecutable(executable)) {
                    driver.cmd("isready", "readyok");
                    for (String epdLine : epds) {
                        StdEpd.EpdLine epd = parseLine(epdLine);
                        driver.uciNewGame();
                        String fen = String.format("%s 1 1", epd.fen());
                        driver.positionFen(fen);
                        Optional<String> result = driver.go(depth);

                        String uciMove = oneSanToUci(epd.bestMove(), BoardPosition.fromFen(fen));
                        String foundMove = result.orElse("n/a");
                        StsResult stsResult = new StsResult(epd.desc().section(), epd.desc().number(), foundMove.equals(uciMove) ? 1 : 0);
                        resultList.add(stsResult);

                        System.out.printf("%s %s %s%n", foundMove, uciMove, fen);
                    }
                }
                return resultList;
            }
        }

        ArrayList<List<String>> epdChunks = Lists.newArrayList(Iterables.partition(epds, coresCount));
        List<EpdChunkTask> chunkTasks = epdChunks.stream().map(EpdChunkTask::new).toList();

        ExecutorService executorService = Executors.newFixedThreadPool(coresCount);
        List<Future<List<StsResult>>> futures = executorService.invokeAll(chunkTasks);
        List<StsResult> stsResults = futures.stream().flatMap(x -> {
            try {
                return x.get().stream();
            } catch (InterruptedException | ExecutionException e) {
                throw new RuntimeException(e);
            }
        }).toList();

        Map<String, List<StsResult>> groupedBySection = stsResults.stream()
                .collect(groupingBy(StsResult::section));
        System.out.println(groupedBySection.size());

        Map<String, StsGrouped> stsGrouped = groupedBySection.entrySet().stream()
                .map(section -> section.getValue().stream()
                        .map(sr -> new StsGrouped("", 1, sr.outcome))
                        .reduce(new StsGrouped(section.getKey(), 0, 0), (l, r) -> new StsGrouped(section.getKey(), l.total + r.total, l.matched + r.matched)))
                .collect(toMap(StsGrouped::section, Function.identity()));
        // System.out.println(stsGrouped);
        int matchingCount = stsGrouped.values().stream().map(sg -> sg.matched).reduce(0, Integer::sum);
//        groupedBySection.keySet().forEach(s -> {
//            System.out.printf("\"%s\"%n", s);
//        });
        //:: STS ID and Titles ::
        //STS 01: Undermining
        //STS 02: Open Files and Diagonals
        //STS 03: Knight Outposts
        //STS 04: Square Vacancy
        //STS 05: Bishop vs Knight
        //STS 06: Re-Capturing
        //STS 07: Offer of Simplification
        //STS 08: Advancement of f/g/h Pawns
        //STS 09: Advancement of a/b/c Pawns
        //STS 10: Simplification
        //STS 11: Activity of the King
        //STS 12: Center Control
        //STS 13: Pawn Play in the Center
        //STS 14: Queens and Rooks to the 7th rank
        //STS 15: Avoid Pointless Exchange
        List<StsGrouped> orderedSections = Stream.of(
                "Undermine",
                "Open Files and Diagonals",
                "Knight Outposts", // /Repositioning/Centralization",
                "Square Vacancy",
                "Bishop vs Knight",
                "Recapturing",
                "Offer of Simplification",
                "AKPC", // f/g/h pawns
                "Advancement of a/b/c pawns",
                "Simplification",
                "King Activity",
                "Center Control",
                "Pawn Play in the Center",
                "7th Rank", // queens and rooks
                "AT" // avoid pointless exchange
        ).map(stsGrouped::get).toList();
        int longestTitle = orderedSections.stream().mapToInt(os -> os.section.length()).max().orElse(10);

        for (int i = 0; i < orderedSections.size(); i++) {
            StsGrouped sg = orderedSections.get(i);
            System.out.printf("STS %02d: %s %3d %3d%n", i+1, Strings.padEnd(sg.section, longestTitle, ' '), sg.matched, sg.total);
        }

        long end = System.currentTimeMillis();
        long totalTime = end - start;
        int totalSize = epds.size();
        String engineId = String.format("%s@d%d", engineName, depth);
        System.out.printf("%s: %d / %d, %.3f%%, %.3f %n", engineId, matchingCount, totalSize, ((float)matchingCount)/totalSize*100, ((float)totalTime) / 1000);
    }

    public record StsGrouped(String section, int total, int matched) {};

    // STS Rating v14.0
    //Number of cores: 8
    //
    //Engine: Javafish 1.0
    //Hash: 128, Threads: 1, time/pos: 0.200s
    //
    //Number of positions in STS1-STS15_LAN_v3.epd: 1500
    //Max score = 1500 x 10 = 15000
    //Test duration: 00h:01m:30s
    //Expected time to finish: 00h:05m:45s
    //STS rating: 1977
    //
    //  STS ID   STS1   STS2   STS3   STS4   STS5   STS6   STS7   STS8   STS9  STS10  STS11  STS12  STS13  STS14  STS15    ALL
    //  NumPos    100    100    100    100    100    100    100    100    100    100    100    100    100    100    100   1500
    // BestCnt     37     20     39     40     49     42     32     19     31     57     33     42     48     48     17    554
    //   Score    482    345    534    519    591    678    422    311    403    667    447    536    579    618    348   7480
    //Score(%)   48.2   34.5   53.4   51.9   59.1   67.8   42.2   31.1   40.3   66.7   44.7   53.6   57.9   61.8   34.8   49.9
    //  Rating   1903   1293   2135   2068   2388   2776   1636   1142   1551   2727   1747   2144   2335   2509   1307   1977
    //
    //:: STS ID and Titles ::
    //STS 01: Undermining
    //STS 02: Open Files and Diagonals
    //STS 03: Knight Outposts
    //STS 04: Square Vacancy
    //STS 05: Bishop vs Knight
    //STS 06: Re-Capturing
    //STS 07: Offer of Simplification
    //STS 08: Advancement of f/g/h Pawns
    //STS 09: Advancement of a/b/c Pawns
    //STS 10: Simplification
    //STS 11: Activity of the King
    //STS 12: Center Control
    //STS 13: Pawn Play in the Center
    //STS 14: Queens and Rooks to the 7th rank
    //STS 15: Avoid Pointless Exchange
    //
    //:: Top 5 STS with high result ::
    //1. STS 06, 67.8%, "Re-Capturing"
    //2. STS 10, 66.7%, "Simplification"
    //3. STS 14, 61.8%, "Queens and Rooks to the 7th rank"
    //4. STS 05, 59.1%, "Bishop vs Knight"
    //5. STS 13, 57.9%, "Pawn Play in the Center"
    //
    //:: Top 5 STS with low result ::
    //1. STS 08, 31.1%, "Advancement of f/g/h Pawns"
    //2. STS 02, 34.5%, "Open Files and Diagonals"
    //3. STS 15, 34.8%, "Avoid Pointless Exchange"
    //4. STS 09, 40.3%, "Advancement of a/b/c Pawns"
    //5. STS 07, 42.2%, "Offer of Simplification"
}
