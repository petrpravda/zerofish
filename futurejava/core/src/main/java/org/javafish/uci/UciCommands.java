package org.javafish.uci;

import org.javafish.board.BoardPosition;
import org.javafish.board.BoardState;
import org.javafish.move.Move;
import org.javafish.move.MoveList;
import org.javafish.pgn.PgnMoves;
import org.javafish.pgn.PgnParser;
import org.javafish.uci.annotation.UciArgs;
import org.javafish.uci.annotation.UciMapping;
import org.javafish.uci.annotation.UciQuit;
import search.Search;
import search.TranspositionTable;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

import static org.javafish.app.Perft.perftString;
import static org.javafish.board.Fen.START_POS;
import static org.javafish.move.Move.parseUciMoves;

public class UciCommands {

    private TranspositionTable transpositionTable = new TranspositionTable();
    private BoardPosition position = BoardPosition.fromFen(START_POS);

    public Optional<UciLambdaCommand> createCommandInstance(String baseToken, String[] tokens) {
        UciLambdaCommand result =  switch (baseToken) {
            case "d" -> new UciLambdaCommand() {
                @Override
                public void execute() {
                    MoveList moves = position.getState().generateLegalMoves();
                    String movesString = moves.stream().map(Move::uci).collect(Collectors.joining(" "));
                    String checkers = "TBD"; //checkerMoves.stream().map(m -> Square.getName(m.start())).collect(Collectors.joining(" "));

                    StringBuilder output = new StringBuilder(position.getState().toString())
                            .append('\n')
                            .append("Checkers: ").append(checkers).append("\n")
                            .append("Legal uci moves: ").append(movesString).append("\n");

                    System.out.println(output);
                }
            };
            case "isready" -> () -> System.out.println("readyok");
            case "position" -> () -> {
                List<String> args = Arrays.asList(tokens);

                if (args.size() >= 2 && args.get(0).equals("startpos") && args.get(1).equals("moves")) {
                    positionWithMovesFromStartposSoFar(args.subList(2, args.size()));
                } else if (args.size() >= 2 && args.get(0).equals("startpos") && args.get(1).equals("movespgn")) {
                    PgnMoves pgnMoves = PgnParser.fromSan(args.subList(2, args.size()));
                    String uciMoves = pgnMoves.asUci();
                    System.out.println(uciMoves);
                    List<String> moves = !uciMoves.isEmpty() ? List.of(uciMoves.split("\\s+")) : Collections.emptyList();
                    positionWithMovesFromStartposSoFar(moves);
                } else if (!args.isEmpty() && args.get(0).equals("fen")) {
                    position = BoardPosition.fromFen(String.join(" ", args.subList(1, args.size())));
                }
            };
            case "go" -> () -> {
                if (tokens.length > 1) {
                    if (tokens[0].equals("perft")) {
                        int depth = Integer.parseInt(tokens[1]);
                        System.out.println(perftString(position.getState(), depth));
                    } else if (tokens[0].equals("depth") && tokens.length == 2) {
                        int depth = Integer.parseInt(tokens[1]);
                        //BoardState state = fromPosition(boardPosition);
//                SearchBaba search = fromBoardState(state);
//                // search.pvMode = true;
//                Optional<Move> bestMove = search.findBestMove(depth);
//                String bestMoveString = String.format("bestmove %s", bestMove.map(Move::toString).orElse("(none)"));
//                System.out.println(bestMoveString);

//                BoardPosition boardPosition = fromFen(START_POS);
//                BoardState state = fromPosition(boardPosition);

                        Search.SearchResult result2 = new Search(transpositionTable, System.out).itDeep(position, depth);
                        String bestMoveString = String.format("bestmove %s", result2.move().map(Move::toString).orElse("(none)"));
                        System.out.println(bestMoveString);
                    }
                } else {
                    System.out.format("go - %s%n", Arrays.asList(tokens));
                }

            };
            case "uci" -> () -> System.out.println("""
            id name Javafish 1.0
            by Petr Pravda
            uciok""");
            case "ucinewgame" -> () -> transpositionTable.reset();
            case "quit" -> makeQuitRequest();
            default -> null;
        };
        return Optional.ofNullable(result);
    }

    public UciLambdaCommand makeQuitRequest() {
        return () -> Engine.QUIT_REQUESTED = true;
    }


    private void positionWithMovesFromStartposSoFar(List<String> args) {
        position = BoardPosition.fromFen(START_POS);
        BoardState state = position.getState();
        List<Move> moves = parseUciMoves(args, state);

        for (Move move1 : moves) {
            Move moveValidated = state.generateLegalMoves()
                    .stream().filter(m -> m.toString().equals(move1.toString()))
                    .findFirst()
                    .orElseThrow();
            // TODO repetition missing
            state = position.doMove(moveValidated);
        }

        // state = fromFen(Fen.toFen(state));
    }


//    @UciMapping("isready")
//    public void isReady() {
//        System.out.println("readyok");
//    }



//    @UciQuit
//    @UciMapping("quit")
//    public void quit() {
//    }

//    @UciMapping("pgntouci")
//    public void pgnToUci(@UciArgs String[] tokens) {
//        List<String> list = Arrays.stream(tokens).collect(Collectors.toList());
//        PgnMoves pgnMoves = PgnParser.fromSan(list);
//        System.out.println(pgnMoves.asUci());
//    }


//    @UciMapping("stop")
//    public class StopCommand extends AbstractCommand {
//        public StopCommand(String[] tokens) {
//            super(tokens);
//        }
//
//
//        @Override
//        public void execute(Search search) {
//            System.out.println("STOP!!!!");
//        }
//    }
}
