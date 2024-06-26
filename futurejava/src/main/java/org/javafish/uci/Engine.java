package org.javafish.uci;

import java.util.concurrent.BlockingQueue;
import java.util.concurrent.PriorityBlockingQueue;
import java.util.logging.Level;
import java.util.logging.Logger;

public class Engine {
    private static final Logger LOGGER = Logger.getLogger(String.valueOf(UciRepl.class));
    public static volatile boolean QUIT_REQUESTED = false;
    private UciCommands uciCommandsInstance = new UciCommands();

    public static final String POSITION_FEN = "position fen";
    private static final String STARTPOS_MOVES = "startpos moves ";
    private static final int TT_SIZE_MB = 100;

    private final BlockingQueue<UciLambdaCommand> queue;
    private String[] args;

    public Engine(BlockingQueue<UciLambdaCommand> queue, String[] args) {
        this.queue = queue;
        this.args = args;
    }

    public void mainLoop() {
        try {
            System.out.println("Javafish 0.1 by Petr Pravda");

//            Search search = new Search(this.queue);
//            search.board = new Board(fromFen(START_POS));
//            configureCommandLineOptions(search.board, args);
//            //search.board = fromFen("r5k1/p1p1n2p/1p5b/4p3/NP1p4/4R2P/P1PK4/5q2 b - - 2 29");
//            search.movegen = new MoveGenerator(search.board.bitboard);
//            search.hh = new HistoryHeuristics();
//            search.tt = new TranspositionTable(TT_SIZE_MB);

            LOGGER.fine("board inicializovany");


            //noinspection InfiniteLoopStatement
            while (true) {
                UciLambdaCommand command = queue.take();
                //System.out.printf("Executing %s\n", command.getClass().getSimpleName());
                command.execute();

                if (QUIT_REQUESTED) {
                    break;
                }
                //System.out.printf("Execution of %s finished\n", command.getClass().getSimpleName());
            }
//        } catch (QuitProcessingException e) {
//            LOGGER.info("Quit requested");
        } catch (Exception e) {
            LOGGER.log(Level.SEVERE, e.getMessage(), e);
        }
        Thread.currentThread().interrupt();
    }

//    private void configureCommandLineOptions(Board board, String[] args) {
//        for (String arg : args) {
//            String[] strings = arg.split("=");
//            if (strings.length == 2) {
//                String name = strings[0];
//                String value = strings[1];
//                Optional<Field> field = board.options.findField(name);
//                if (field.isEmpty()) {
//                    LOGGER.warning(String.format("Option \"%s\" not supported", name));
//                } else {
//                    try {
//                        board.options.setOptionValue(field.get(), value);
//                    } catch (IllegalAccessException e) {
//                        LOGGER.log(Level.WARNING, e.getMessage(), e);
//                    }
//                }
//            }
//        }
//
//    }
//
//
//    public static List<UciMove> parseUciMoves(List<String> moves) {
//
//        return moves.stream()
//                .map(s -> UciMove.fromUciString(s))
//                .collect(Collectors.toList());
//    }

//    public static class QuitProcessingException extends RuntimeException {}

}
