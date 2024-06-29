package org.javafish.uci;

import org.javafish.uci.annotation.UciArgs;
import org.javafish.uci.annotation.UciMapping;
import org.javafish.uci.annotation.UciQuit;

import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Optional;

public class UciProcessor {
    UciCommands uciCommands = new UciCommands();
//    public record UciCommandMetaInfo(String method, boolean passArgs, boolean quit) {
//
//    }

//    private static final Map<String, UciCommandMetaInfo> COMMAND_HANDLERS = new LinkedHashMap<>();
    // public static final Class<?>[] STRING_ARRAY_TYPE = {String[].class};

//    static {
//        Method[] methods = UciCommands.class.getMethods();
//        for (Method method : methods) {
//            //Annotation[] annotations = method.getDeclaredAnnotations();
//            UciMapping uciAnnotation = method.getAnnotation(UciMapping.class);
//            if (uciAnnotation != null) {
//                boolean passArgs = Arrays.stream(method.getParameters()).anyMatch(p -> p.getAnnotation(UciArgs.class) != null);
//                UciCommandMetaInfo uciCommandMetaInfo = new UciCommandMetaInfo(method.getName(),
//                        passArgs, method.getAnnotation(UciQuit.class) != null);
//                COMMAND_HANDLERS.put(uciAnnotation.value(), uciCommandMetaInfo);
//                // System.out.format("%s - %s%n", uciAnnotation.value(), uciCommandMetaInfo);
//            }
//        }
//    }
//
//    public static HalfParsedCommand makeQuitCommand() {
//        return matchCommand("quit").orElseThrow();
//    }

    public Optional<UciLambdaCommand> matchCommand(String line) {
        String[] tokens = line.split("\\s+");
        String[] restTokens = Arrays.copyOfRange(tokens, 1, tokens.length);
        return uciCommands.createCommandInstance(tokens[0], restTokens);
//        UciCommandMetaInfo uciInfo = COMMAND_HANDLERS.get(tokens[0]);
//        return Optional.ofNullable(uciInfo)
//                .map(clazz -> {
//
//                    return new HalfParsedCommand(rightTokens, uciInfo);
//                });
    }

    public UciLambdaCommand makeQuitCommand() {
        return uciCommands.makeQuitRequest();
    }
}
