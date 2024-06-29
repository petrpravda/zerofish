package org.javafish.pgn;

import org.javafish.board.*;
import org.javafish.move.Move;
import org.javafish.board.PieceType;
import org.javafish.move.MoveList;

import java.util.List;
import java.util.Optional;
import java.util.StringJoiner;
import java.util.stream.Collectors;

import static org.javafish.board.Fen.START_POS;


public class PgnMoves {
    private List<String> moveStrings;
    // private BoardState state;

    public void setMoveStrings(List<String> moveStrings) {
        this.moveStrings = moveStrings;
    }

    public List<String> getMoveStrings() {
        return moveStrings;
    }

    public String asSan() {
        return String.join(" ", moveStrings);
    }

    public String asUci() {
        return asUciFromPosition(START_POS);
    }

    public String asUciFromPosition(String fen) {
        BoardPosition position = BoardPosition.fromFen(fen);
        String result = moveStrings.stream()
                .map(move -> oneSanToUci(move, position))
                .collect(Collectors.joining(" "));
        return result;
    }

    public static Optional<Move> parseOneSan(String san, BoardState state) {
        boolean checkingMove = san.endsWith("+");
        boolean checkmatingMove = san.endsWith("#");
        if (checkingMove || checkmatingMove) {
            san = san.substring(0, san.length() - 1);
        }
        final String destination;
        final int piece;
        int promotionPiece = -1;
        Character fromFile = null;
        Character fromRank = null;
        if (san.equals("O-O")) {
            destination = state.getSideToPlay() == Side.WHITE ? "g1" : "g8";
            piece = PieceType.KING;
        } else if (san.equals("O-O-O")) {
            destination = state.getSideToPlay() == Side.WHITE ? "c1" : "c8";
            piece = PieceType.KING;
        } else {
            if (san.charAt(san.length() - 2) == '=') {
                promotionPiece = EnumPieceType.fromSan(san.charAt(san.length() - 1)).orElseThrow(IllegalStateException::new).ordinal();
                san = san.substring(0, san.length() - 2);
            }

            Optional<EnumPieceType> pieceTypeOptional = EnumPieceType.fromSan(san.charAt(0));
            piece = pieceTypeOptional.orElse(EnumPieceType.PAWN).ordinal();
            String sanWithoutPieceType;
            if (pieceTypeOptional.isPresent()) {
                sanWithoutPieceType = san.substring(1);
            } else {
                sanWithoutPieceType = san;
            }
            int takingMoveFlagIndex = sanWithoutPieceType.indexOf('x');
            String source = "";
            if (takingMoveFlagIndex != -1) {
                source = sanWithoutPieceType.substring(0, takingMoveFlagIndex);
                destination = sanWithoutPieceType.substring(takingMoveFlagIndex + 1);
            } else {
                int sourceLength = sanWithoutPieceType.length() - 2;
                if (sourceLength < 0) {
                    throw new IllegalStateException(String.format("Weird move: %s", san));
                }
                source = sanWithoutPieceType.substring(0, sourceLength);
                destination = sanWithoutPieceType.substring(sourceLength);
            }

            switch (source.length()) {
                case 0:
                    break;
                case 1:
                    char fromChar = source.charAt(0);
                    if (Character.isDigit(fromChar)) {
                        fromRank = fromChar;
                    } else {
                        fromFile = fromChar;
                    }
                    break;
                case 2:
                    fromFile = source.charAt(0);
                    fromRank = source.charAt(1);
                    break;
                default:
                    throw new UnsupportedOperationException(String.format("%s is not implemented, length???, [%s]", sanWithoutPieceType, san));
            }
        }
        int destinationNumber = Square.getSquareFromName(destination);
        final Character fromFileFinal = fromFile;
        final Character fromRankFinal = fromRank;
        final Integer promotionPieceFinal = promotionPiece;
        MoveList uciMoves = state.generateLegalMoves();
        List<Move> matchingMoves = uciMoves.stream()
                .filter(move -> move.to() == destinationNumber)
                .filter(move -> move.isPromotion() ? move.getPieceType() == promotionPieceFinal :
                        /*move.flags() == EN_PASSANT ||*/ state.pieceTypeAt(move.from()) == piece)
                .filter(move -> fromFileFinal == null || Square.getFile(move.from()) == fromFileFinal)
                .filter(move -> fromRankFinal == null || Square.getRank(move.from()) == fromRankFinal)
                .toList();
        if (matchingMoves.size() > 1) {
            throw new IllegalStateException(String.format("Ambiguous possible moves: %s", matchingMoves));
        }
        Optional<Move> matchingMove = matchingMoves.stream().findAny();
        return matchingMove;
    }

    public static String oneSanToUci(String san, BoardPosition position) {
        BoardState state = position.getState();
        Optional<Move> matchingMove = parseOneSan(san, state);

        if (matchingMove.isEmpty()) {
            throw new IllegalStateException(String.format("Move %s not found", san));
        } else {
            Move theMove = matchingMove.get();
            position.doMove(theMove);
            return theMove.toString();
        }
    }

    @Override
    public String toString() {
        return new StringJoiner(", ", PgnMoves.class.getSimpleName() + "[", "]")
                .add("moveStrings=" + moveStrings)
                .toString();
    }
}
