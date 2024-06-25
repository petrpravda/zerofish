package org.javafish.pgn;


import org.javafish.board.BoardState;
import org.javafish.board.Square;
import org.javafish.move.Move;
import org.javafish.board.PieceType;

import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

public class UciMoves {
    private BoardState state;
    private List<String> moveStrings;

    public UciMoves(BoardState state) {
        this.state = state;
    }

    public void setMoveStrings(List<String> moveStrings) {
        this.moveStrings = moveStrings;
    }

    public List<String> getMoveStrings() {
        return moveStrings;
    }

    public String asUci() {
        return String.join(" ", moveStrings);
    }

    public String asSan() {
        return moveStrings.stream()
                .map(move -> oneUciToSan(move))
                .collect(Collectors.joining(" "));
    }

    private String oneUciToSan(String uciMove) {
        List<Move> uciMoves = state.generateLegalMoves();
        Optional<Move> moveCheating = uciMoves.stream()
                .filter(move -> move.toString().equals(uciMove))
                .findFirst();

        Integer piece = moveCheating.map(Move::getPieceType).orElse(null);
        String uciDestination = uciMove.substring(2, 4);
        Character sourceFile = uciMove.charAt(0);
        Character sourceRank = uciMove.charAt(1);

        List<Move> matchingMoves = uciMoves.stream()
                .filter(move -> Square.getName(move.to()).equals(uciDestination))
                .filter(move -> piece == null || piece == move.getPieceType())
//                .filter(move -> moveCheating.isEmpty() || move.typ() == moveCheating.get().typ())
                .collect(Collectors.toList());

        if (matchingMoves.size() == 1) {
            return moveToPgn(matchingMoves.get(0), false, false);
        }

        matchingMoves = uciMoves.stream()
                .filter(move -> Square.getName(move.to()).equals(uciDestination))
                .filter(move -> piece == null || piece == move.getPieceType())
                .filter(move -> sourceFile == Square.getFile(move.from()))
                .filter(move -> moveCheating.isEmpty() /*|| move.typ() == moveCheating.get().typ()*/)
//                .filter(move -> fromRankFinal == null || Square.getRank(move.start()) == fromRankFinal)
                .collect(Collectors.toList());

        if (matchingMoves.size() == 1) {
            return moveToPgn(matchingMoves.get(0), true, false);
        }

        matchingMoves = uciMoves.stream()
                .filter(move -> Square.getName(move.to()).equals(uciDestination))
                .filter(move -> piece == null || piece == move.getPieceType())
                //.filter(move -> sourceFile == Square.getFile(move.start()))
                .filter(move -> sourceRank == Square.getRank(move.from()))
                .filter(move -> moveCheating.isEmpty() /*|| move.typ() == moveCheating.get().typ()*/)
                .collect(Collectors.toList());

        if (matchingMoves.size() == 1) {
            return moveToPgn(matchingMoves.get(0), false, true);
        }

        matchingMoves = uciMoves.stream()
                .filter(move -> Square.getName(move.to()).equals(uciDestination))
                .filter(move -> piece == null || piece == move.getPieceType())
                .filter(move -> sourceFile == Square.getFile(move.from()))
                .filter(move -> sourceRank == Square.getRank(move.from()))
                .filter(move -> moveCheating.isEmpty() /*|| move.typ() == moveCheating.get().typ()*/)
                .collect(Collectors.toList());

        if (matchingMoves.size() == 1) {
            return moveToPgn(matchingMoves.get(0), true, true);
        }


        if (matchingMoves.size() > 1) {
            throw new IllegalStateException(String.format("Ambiguous possible moves: %s for %s", matchingMoves, uciMove));
        } else {
            throw new IllegalStateException(String.format("Move %s not found", uciMove));
        }
    }

    private String moveToPgn(Move move, boolean fileNeededParam, boolean rankNeededParam) {
        String destination = Square.getName(move.to());
        String pieceIdentification = "";
        boolean fileNeeded = fileNeededParam;
        boolean rankNeeded = rankNeededParam;
        final String halfResult;
        boolean capturing; // = false;
        String moveString = move.toString();
        if (move.getPieceType() == PieceType.KING && (moveString.equals("e1g1") || moveString.equals("e8g8"))) {
            halfResult = "O-O";
        } else if (move.getPieceType() == PieceType.KING && (moveString.equals("e1c1") || moveString.equals("e8c8"))) {
            halfResult = "O-O-O";
        } else {
            throw new IllegalStateException("TBD");
//            if ((!move.isPromotion() /*&& move.piece_id() == EnumPieceType.P*/)
//                || move.isPromotion()) { // TODO lze zjednodusit
//                capturing = Square.getFile(move.from()) != Square.getFile(move.to());
//                if (capturing) {
////                        sourceFile = Square.getFile(move.start());
//                    fileNeeded = true;
//                }
//            } else {
//                capturing = move.typ() == EnumMoveType.CAPTURE
//                    || move.typ() == EnumMoveType.KING_CAPTURE;
//                pieceIdentification = move.piece_id().name().toUpperCase();
//            }
//            halfResult = new StringBuffer()
//                    .append(pieceIdentification)
//                    .append(fileNeeded ? Square.getFile(move.from()) : "")
//                    .append(rankNeeded ? Square.getRank(move.from()) : "")
//                    .append(capturing ? "x" : "")
//                    .append(destination)
//                    .toString();
        }
        boolean checking = false; // state.isMoveChecking(move);
        StringBuilder result = new StringBuilder(halfResult);
//        if (move.typ() == EnumMoveType.PROMOTION) {
//            result.append('=');
//            result.append(move.piece_id().name());
//        }

        this.state = state.doMove(move);

        List<Move> legalMovesForOpponent = state.generateLegalMoves();

        if (legalMovesForOpponent.size() == 0 && checking) {
            // we need to be sure that it is the checkmate, therefore "checking"
            result.append('#');
        } else if (checking) {
            result.append('+');
        }

        return result.toString();
    }
}
