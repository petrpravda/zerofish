package org.javafish.uci;


import java.util.List;
import java.util.Optional;
import java.util.Scanner;
import java.util.concurrent.PriorityBlockingQueue;
import java.util.logging.Logger;


public class UciRepl {
    private static final Logger LOGGER = Logger.getLogger(String.valueOf(UciRepl.class));

    private final PriorityBlockingQueue<HalfParsedCommand> queue;

    public UciRepl(PriorityBlockingQueue<HalfParsedCommand> queue) {
        this.queue = queue;
    }

    public void mainLoop() {
        Scanner sc = new Scanner(System.in);
        while (sc.hasNextLine()) {
            String line = sc.nextLine();
            Optional<HalfParsedCommand> command = UciProcessor.matchCommand(line);

            if (command.isPresent()) {
                HalfParsedCommand cmd = command.get();
                LOGGER.info(String.format("processing: %s %s", cmd.metaInfo().method(), String.join(" ", List.of(command.get().tokens()))));
                queue.add(command.get());

                if (command.get().isQuitting()) {
                    System.out.println("bye");
                    break;
                }
            }
        }

        // CTLR + D handling
        queue.add(UciProcessor.makeQuitCommand());
    }
}
