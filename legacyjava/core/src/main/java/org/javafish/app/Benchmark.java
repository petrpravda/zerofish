package org.javafish.app;


import org.javafish.board.BoardState;

import static org.javafish.board.Fen.START_POS;
import static org.javafish.app.Perft.perft;

// 119060324 nodes in 3464, nps: 34370763.279446
//119060324 nodes in 1936, nps: 61498101.239669
//119060324 nodes in 1951, nps: 61025281.394157
//119060324 nodes in 1931, nps: 61657340.238219
//119060324 nodes in 1933, nps: 61593545.783756
//119060324 nodes in 1931, nps: 61657340.238219
//119060324 nodes in 1932, nps: 61625426.501035
//119060324 nodes in 1940, nps: 61371301.030928
//119060324 nodes in 1931, nps: 61657340.238219
//119060324 nodes in 1931, nps: 61657340.238219

public class Benchmark {

    public static final int GIGA = 1000000000;
    public static final double MEGA = 1000000.;

    public static void main(String[] args)     {
        long totalTime = 0;
        long totalNodes = 0;
        for (int i = 0; i < 10; i++) {
            BoardState state = BoardState.fromFen(START_POS);

            long start = System.nanoTime();
            // long nodesCount = perft2(state, 5);
            long nodesCount = perft(state, 6);
            // System.out.println(nodesCount);
            long end = System.nanoTime();
            long time = end - start;
            double nps = ((double) nodesCount) / time * GIGA;
            System.out.format("%d nodes in %f ms, nps: %f%n", nodesCount, ((double)time) / MEGA, nps);

            if (i > 0) {
                totalTime += time;
                totalNodes += nodesCount;
            }
        }
        double nps = ((double) totalNodes) / totalTime * GIGA;
        System.out.format("AVG:%n%d nodes in %f, nps: %f%n", totalNodes, ((double)totalTime) / MEGA, nps);
    }
}
