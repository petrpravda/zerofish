package org.javafish.uci;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.Optional;
import java.util.logging.Level;
import java.util.logging.Logger;

public record HalfParsedCommand(String[] tokens, UciProcessor.UciCommandMetaInfo metaInfo) implements Comparable<HalfParsedCommand> {
    private static final Logger LOGGER = Logger.getLogger(String.valueOf(HalfParsedCommand.class));

    @Override
    public int compareTo(HalfParsedCommand o) {
        return 0;
    }

    public boolean isQuitting() {
        return this.metaInfo.quit();
    }

    public void execute(UciCommands uciCommandsInstance) {
        Optional<Method> method = Arrays.stream(UciCommands.class.getMethods())
                .filter(m -> m.getName().equals(this.metaInfo.method()))
                .findFirst();

        if (method.isEmpty()) {
            throw new IllegalStateException();
        }

        try {
            method.get().invoke(uciCommandsInstance, this.metaInfo.passArgs() ? new Object[]{this.tokens} : new Object[]{});
        } catch (IllegalAccessException | InvocationTargetException e) {
            LOGGER.log(Level.SEVERE, String.format("Cannot execute method %s", method.get().getName()), e);
        }
    }
}
