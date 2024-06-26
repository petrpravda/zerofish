package org.javafish.app;


import org.javafish.uci.Engine;
import org.javafish.uci.HalfParsedCommand;
import org.javafish.uci.UciRepl;

import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.PriorityBlockingQueue;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.logging.Logger;

public class Main {
    static {
        System.setProperty("java.util.logging.SimpleFormatter.format", "%1$tY-%1$tm-%1$td %1$tH:%1$tM:%1$tS.%1$tL %4$-6s %2$s %5$s%6$s%n");
    }

    private static final Logger LOGGER = Logger.getLogger(String.valueOf(Main.class));

    public static void main(String[] args) {
        LOGGER.fine("Starting");

        ExecutorService searchExecutor = Executors.newSingleThreadExecutor();

        PriorityBlockingQueue<HalfParsedCommand> queue = new PriorityBlockingQueue<>();
        Future<?> executorTask = searchExecutor.submit(() -> {
            try {
                new Engine(queue, args).mainLoop();
            } catch (Exception e) {
                e.printStackTrace();
            }
        });


        new UciRepl(queue).mainLoop();

        try {
            LOGGER.fine("before get");
            executorTask.get(1, TimeUnit.SECONDS);
            LOGGER.fine("after get");
            searchExecutor.shutdownNow();
        } catch (InterruptedException | ExecutionException | TimeoutException e) {
            LOGGER.severe(e.getMessage());
        }
    }
}
