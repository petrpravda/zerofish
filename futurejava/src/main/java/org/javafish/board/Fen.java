package org.javafish.board;


import org.javafish.bitboard.Bitboard;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static org.javafish.bitboard.Bitboard.BLACK_KINGS_ROOK_MASK;
import static org.javafish.bitboard.Bitboard.BLACK_KING_INITIAL_SQUARE;
import static org.javafish.bitboard.Bitboard.BLACK_QUEENS_ROOK_MASK;
import static org.javafish.bitboard.Bitboard.WHITE_KINGS_ROOK_MASK;
import static org.javafish.bitboard.Bitboard.WHITE_KING_INITIAL_SQUARE;
import static org.javafish.bitboard.Bitboard.WHITE_QUEENS_ROOK_MASK;

public class Fen {
    public static final String START_POS = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    private static final Pattern REGEX_FEN_FREE = Pattern.compile("Fen: (.*)");
    public static final int MAX_SEARCH_DEPTH = 100;

    public static String toFen(BoardState state) {
        StringBuilder fen = new StringBuilder();
        int count = 0;
        int rankCounter = 1;
        int sqCount = 0;
        for (int rank = 7; rank >= 0; rank--) {
            for (int file = 0; file <= 7; file++) {
                int square = (rank << 3) + file;
                int piece = state.items[square];
                if (piece != Piece.NONE) {
                    if (count > 0) {
                        fen.append(count);
                    }
                    fen.append(Piece.getNotation(piece));
                    count = 0;
                } else {
                    count++;
                }
                if ((sqCount + 1) % 8 == 0) {
                    if (count > 0) {
                        fen.append(count);
                        count = 0;
                    }
                    if (rankCounter < 8) {
                        fen.append("/");
                    }
                    rankCounter++;
                }
                sqCount++;
            }
        }
        if (state.getSideToPlay() == Side.WHITE) {
            fen.append(" w");
        } else {
            fen.append(" b");
        }

        String rights = "";
        if ((Bitboard.castlingPiecesKingsideMask(Side.WHITE) & state.movements) == 0)
            rights += "K";
        if ((Bitboard.castlingPiecesQueensideMask(Side.WHITE) & state.movements) == 0)
            rights += "Q";
        if ((Bitboard.castlingPiecesKingsideMask(Side.BLACK) & state.movements) == 0)
            rights += "k";
        if ((Bitboard.castlingPiecesQueensideMask(Side.BLACK) & state.movements) == 0)
            rights += "q";

        if (rights.equals("")) {
            fen.append(" -");
        } else {
            fen.append(" ").append(rights);
        }

        if (state.enPassant != 0L)
            fen.append(" ").append(Square.getName(Long.numberOfTrailingZeros(state.enPassant)));
        else
            fen.append(" -");

        fen.append(' ').append(state.halfMoveClock);
        fen.append(' ').append((state.fullMoveNormalized / 2) + 1);

        return fen.toString();
    }

//    public static String toFen(BoardState state) {
//        StringBuilder fen = new StringBuilder();
//        int count = 0;
//        int rankCounter = 1;
//        int sqCount = 0;
//        for(int rank = 7; rank >= 0; rank--){
//            for(int file = 0; file <= 7; file++){
//                int square = rank << 3 | file;
//                int piece = state.items[square];
//                if (piece != 0){
//                    if (count > 0) {
//                        fen.append(count);
//                    }
//                    fen.append(EnumPieceType.toFen(piece));
//                    count = 0;
//                }
//                else{
//                    count++;
//                }
//                if ((sqCount + 1) % 8 == 0){
//                    if (count > 0){
//                        fen.append(count);
//                        count = 0;
//                    }
//                    if (rankCounter < 8){
//                        fen.append("/");
//                    }
//                    rankCounter++;
//                }
//                sqCount++;
//            }
//        }
//
//        fen.append(' ').append(state.getActivePlayer().getFenColor());
//
//        fen.append(' ');
//        fen.append(EnumCastling.toFenFlags(state.castling));
//        fen.append(' ');
//
//        long enpassantState = state.get_enpassant_state();
//        if (enpassantState == 0) {
//            fen.append('-');
////        } else if (board.active_player() == EnumColor.WHITE) {
////            fen.append(Square.getName(WhiteBoardPos.EN_PASSANT_LINE_END.getValue() - Integer.bitCount(enpassantState)));
//        } else {
//            fen.append(Square.getName(Long.numberOfTrailingZeros(enpassantState)));
//        }
//
//        fen.append(' ');
//
////        this.halfmove_count = (bp.fullmove_num - 1) * 2 +  (bp.active_player == EnumColor.WHITE ? 0 : 1);
////        this.state.halfmove_clock = bp.halfmove_clock;
//        //                + board.halfmove_clock().to_string().as_str()
////                + " "
////                + board.fullmove_count().to_string().as_str()
//
//        // TODO doplnit halfmove_clock
//        fen.append(0).append(' ');
//        fen.append(state.getFullMoveNumber()).append(' ');
//
////        if (history[gamePly].epsq != Square.NO_SQUARE)
////            fen.append(" ").append(Square.getName(history[gamePly].epsq));
////        else
////            fen.append(" -");
//
//        return fen.toString();
//    }
//


    private static final Pattern REGEX_EXPAND = Pattern.compile("[2-8]");
    private static final String ONES = "11111111";

    public static String expandFenPieces(String fenPieces) {
        Matcher matcher = REGEX_EXPAND.matcher(fenPieces);
        return matcher.replaceAll((match) -> {
            int countOfSpaces = Integer.parseInt(match.group());
            return ONES.substring(0, countOfSpaces);
        });
    }

    private static final Pattern REGEX_CONDENSE = Pattern.compile("1{2,8}");

//    public static String condenseFenPieces(String fenPieces) {
//        Matcher matcher = REGEX_CONDENSE.matcher(fenPieces);
//        return matcher.replaceAll((match) -> String.valueOf(match.group().length()));
//    }

    public static BoardState fromFen(String fen, Integer maxSearchDepth) {
        List<String> fenParts = Arrays.asList(fen.split("\\s+"));

        String squares = expandFenPieces(fenParts.get(0));
        List<String> squaresList = Arrays.asList(squares.split("/"));
        Collections.reverse(squaresList);
        int[] items = squaresList.stream()
                .flatMap(line -> line.chars().mapToObj(c -> (char) c))
                .mapToInt(c -> switch (c) {
                            case '1' -> Piece.NONE;
                            case 'P' -> Piece.WHITE_PAWN;
                            case 'N' -> Piece.WHITE_KNIGHT;
                            case 'B' -> Piece.WHITE_BISHOP;
                            case 'R' -> Piece.WHITE_ROOK;
                            case 'Q' -> Piece.WHITE_QUEEN;
                            case 'K' -> Piece.WHITE_KING;
                            case 'p' -> Piece.BLACK_PAWN;
                            case 'n' -> Piece.BLACK_KNIGHT;
                            case 'b' -> Piece.BLACK_BISHOP;
                            case 'r' -> Piece.BLACK_ROOK;
                            case 'q' -> Piece.BLACK_QUEEN;
                            case 'k' -> Piece.BLACK_KING;
                            default -> throw new IllegalArgumentException(String.format("Character \"%s\" not known.", c));
                        }
                )
                .toArray();

//                        .forEach(x -> System.out.println(x)))
//                .forEach(line -> System.out.println(line));
//        Collections.rev
//        int file;
//        int rank = 0;
//        for (String r : ranks) {
//            file = 0;
//            for (int i = 0; i < r.length(); i++) {
//                char c = r.charAt(i);
//                if (Character.isDigit(c)) {
//                    file += Integer.parseInt(c + "");
//                } else {
//                    int sq = Square.encode(rank, file);
//                    EnumPieceType pieceType = EnumPieceType.fromFen(c);
//                    EnumColor color = Character.isUpperCase(c) ? EnumColor.WHITE : EnumColor.BLACK;
//                    items[63 - sq] = pieceType.ordinal() * color.getValue();
//
//                    file++;
//                }
//            }
//            rank++;
//        }

        long entry = 0L;
        String castlingFlags = fenParts.get(2);
        if (!castlingFlags.contains("K") || items[WHITE_KING_INITIAL_SQUARE] != Piece.WHITE_KING
                || items[Long.numberOfTrailingZeros(WHITE_KINGS_ROOK_MASK)] != Piece.WHITE_ROOK)
            entry |= WHITE_KINGS_ROOK_MASK;
        if (!castlingFlags.contains("Q") || items[WHITE_KING_INITIAL_SQUARE] != Piece.WHITE_KING
                || items[Long.numberOfTrailingZeros(WHITE_QUEENS_ROOK_MASK)] != Piece.WHITE_ROOK)
            entry |= WHITE_QUEENS_ROOK_MASK;
        if (!castlingFlags.contains("k") || items[BLACK_KING_INITIAL_SQUARE] != Piece.BLACK_KING
                || items[Long.numberOfTrailingZeros(BLACK_KINGS_ROOK_MASK)] != Piece.BLACK_ROOK)
            entry |= BLACK_KINGS_ROOK_MASK;
        if (!castlingFlags.contains("q") || items[BLACK_KING_INITIAL_SQUARE] != Piece.BLACK_KING
                || items[Long.numberOfTrailingZeros(BLACK_QUEENS_ROOK_MASK)] != Piece.BLACK_ROOK)
            entry |= BLACK_QUEENS_ROOK_MASK;


        String enpassant = fenParts.get(3);
        long enPassantMask = enpassant.length() < 2 ? 0 : 1L << Square.getSquareFromName(enpassant);
//        Integer enpassantSquare = Optional.ofNullable(enpassant.length() < 2 ? null : enpassant).map(Square::getSquareFromName).orElse(null);
        String halfMoveClock = fenParts.get(4);
        String fullMoveCount = fenParts.get(5);

        // EnumColor side_to_play = fenParts.get(1).equalsIgnoreCase("w") ? EnumColor.WHITE : EnumColor.BLACK;
        int side_to_play = fenParts.get(1).equalsIgnoreCase("w") ? Side.WHITE : Side.BLACK;

        return new BoardState(items, side_to_play, entry, enPassantMask, Integer.parseInt(halfMoveClock), Integer.parseInt(fullMoveCount),
                Optional.ofNullable(maxSearchDepth).orElse(MAX_SEARCH_DEPTH));
    }

//    private static String validateCastlingFlags(int[] items, int rookSquare, int rookPieceValue, int kingSquare, int kingPieceValue, String code, String castlingFlags) {
//        if (items[rookSquare] != rookPieceValue || items[kingSquare] != kingPieceValue) {
//            return castlingFlags.replaceAll(code, "");
//        } else {
//            return castlingFlags;
//        }
//    }

    public static void main(String[] args) {
        fromFen(START_POS, null);
    }


    public static BoardPosition fromFenFree(String freeFen) {
        Matcher matcher = REGEX_FEN_FREE.matcher(freeFen);
        if (matcher.find()) {
            return BoardPosition.fromFen(matcher.group(1));
        } else {
            throw new IllegalArgumentException(String.format("%s doesn't contain 'Fen: '", freeFen));
        }
    }
}
