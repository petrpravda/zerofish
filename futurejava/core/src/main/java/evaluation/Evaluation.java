package evaluation;

import org.javafish.board.BoardState;
import org.javafish.board.PieceType;
import org.javafish.board.Side;
import org.javafish.eval.annotation.Parameter;

import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;


public class Evaluation {
    record ParamEvaluation(Method method, Parameter parameter) {
    }

    private static final ParamEvaluation[] PARAMETERS;
    static {
        Method[] methods = Evaluation.class.getMethods();
        PARAMETERS = Arrays.stream(methods)
                .map(method -> new ParamEvaluation(method, method.getAnnotation(Parameter.class)))
                .filter(pe -> pe.parameter != null)
                .toArray(ParamEvaluation[]::new);
    }


    public static int whiteKingSq;
    public static int blackKingSq;
//    public static long allWhitePieces;
//    public static long allBlackPieces;
    public static long allPieces;


//    public static void initEval( BoardState state){
//        allWhitePieces = state.allPieces(Side.WHITE);
//        allBlackPieces = state.allPieces(Side.BLACK);
//        allPieces = allWhitePieces ^ allBlackPieces;
//        whiteKingSq = Bitboard.lsb(state.bitboardOf(Piece.WHITE_KING));
//        blackKingSq = Bitboard.lsb(state.bitboardOf(Piece.BLACK_KING));
//    }

//    public static boolean hasBishopPair(long bishopBb){
//        return ((bishopBb & Bitboard.LIGHT_SQUARES) != 0) && ((bishopBb & Bitboard.DARK_SQUARES) != 0);
//    }

//    public static int pawnsShieldingKing(BoardState state, int side){
//        int kingSq = side == Side.WHITE ? whiteKingSq : blackKingSq;
//        long pawns = state.bitboardOf(side, PieceType.PAWN);
//        return Bitboard.popcount(EConstants.PAWN_SHIELD_MASKS[side][kingSq] & pawns);
//    }

//    public static int pawnsOnSameColorSquare(BoardState state, int square, int side){
//        return Bitboard.popcount(state.bitboardOf(side, PieceType.PAWN) &
//                ((Bitboard.DARK_SQUARES & Square.getBb(square)) != 0 ? Bitboard.DARK_SQUARES : Bitboard.LIGHT_SQUARES));
//    }

//    public static int bishopScore(BoardState state, int side){
//        long bishopsBb = state.bitboardOf(side, PieceType.BISHOP);
//        int score = 0;
//        if (hasBishopPair(bishopsBb))
//            score += EConstants.BISHOP_SCORES[EConstants.IX_BISHOP_PAIR_VALUE];
//
//        while (bishopsBb != 0) {
//            int sq = Bitboard.lsb(bishopsBb);
//            bishopsBb = Bitboard.extractLsb(bishopsBb);
//            long attacks = Attacks.getBishopAttacks(sq, allPieces) & ~allPieces;
//
//            if (Bitboard.popcount(attacks & Bitboard.CENTER) == 2)
//                score += EConstants.BISHOP_SCORES[EConstants.IX_BISHOP_ATTACKS_CENTER];
//            score += EConstants.BISHOP_SCORES[EConstants.IX_BISHOP_SAME_COLOR_PAWN_PENALTY]*pawnsOnSameColorSquare(state, sq, side);
//        }
//        return score;
//    }
//
//    public static int rookScore(BoardState state, int side){
//        long rooksBb = state.bitboardOf(side, PieceType.ROOK);
//        int ourKingSq = side == Side.WHITE ? whiteKingSq : blackKingSq;
//        int score = 0;
//
//        long ourPawns = state.bitboardOf(side, PieceType.PAWN);
//        long enemyPawns = state.bitboardOf(Side.flip(side), PieceType.PAWN);
//
//        while (rooksBb != 0){
//            int sq = Bitboard.lsb(rooksBb);
//            rooksBb = Bitboard.extractLsb(rooksBb);
//
//            long rookFileBb = Square.getFileBb(sq);
//            if ((ourPawns & rookFileBb) == 0){
//                if ((enemyPawns & rookFileBb) == 0)
//                    score += EConstants.ROOK_SCORES[EConstants.IX_ROOK_ON_OPEN_FILE];
//                else
//                    score += EConstants.ROOK_SCORES[EConstants.IX_ROOK_ON_SEMIOPEN_FILE];
//            }
//
//            int pieceMobility = Bitboard.popcount(Attacks.getRookAttacks(sq, allPieces) & ~allPieces);
//            // check to see if the king has trapped a rook
//            if (pieceMobility <= 3) {
//                int kf = Square.getFile(ourKingSq);
//                if ((kf < File.FILE_E) == (Square.getFile(sq) < kf))
//                    score += EConstants.ROOK_SCORES[EConstants.IX_KING_TRAPPING_ROOK_PENALTY];
//            }
//        }
//        return score;
//    }

//    public static int kingScore(BoardState state, int side){
//        int score = 0;
//        int pawnShield = pawnsShieldingKing(state, side);
//        score += EConstants.KING_SCORES[EConstants.IX_KING_PAWN_SHIELD_BONUS]*pawnShield;
//        return score;
//    }

//    public static int pSqScore(BoardState state, int side){
//        int score = 0;
//        for (int pt = PieceType.PAWN; pt <= PieceType.KING; pt++) {
//            long bb = state.bitboardOf(side, pt);
//            while (bb != 0){
//                int sq = Bitboard.lsb(bb);
//                bb = Bitboard.extractLsb(bb);
//                score += EConstants.PIECE_TYPE_TABLES[pt][Square.relativeSquare(sq, side)];
//            }
//        }
//        return score;
//    }
//
//    public static int materialScore(BoardState state, int side){
//        int score = 0;
//        for (int pt = PieceType.PAWN; pt <= PieceType.QUEEN; pt++) {
//            long bb = state.bitboardOf(side, pt);
//            score += EConstants.PIECE_TYPE_VALUES_TUNING[pt]*Bitboard.popcount(bb);
//        }
//        return score;
//    }

    public static int evaluateState(BoardState state){
        int score = state.interpolatedScore();



//        score += state.getSideToPlay() == Side.WHITE ? EConstants.TEMPO[0] : -EConstants.TEMPO[0];
//
//        score += Pawns.evaluate(state, Side.WHITE);
//        score -= Pawns.evaluate(state, Side.BLACK);
//
//        score += bishopScore(state, Side.WHITE);
//        score -= bishopScore(state, Side.BLACK);
//
//        score += rookScore(state, Side.WHITE);
//        score -= rookScore(state, Side.BLACK);
//
//        score += kingScore(state, Side.WHITE);
//        score -= kingScore(state, Side.BLACK);

        return state.getSideToPlay() == Side.WHITE ? score : -score;
    }

    public static List<String> paramHeaders(boolean deltas) {
        List<String> headers = new ArrayList<>();
        for (ParamEvaluation parameter : PARAMETERS) {
            if (parameter.parameter().absolute()) {
                if (!deltas) {
                    headers.add(parameter.parameter.name());
                }
            } else if (parameter.parameter().signed()) {
                headers.add(parameter.parameter.name());
            } else if (parameter.parameter.perSide()) {
                headers.add(parameter.parameter.name() + "1");
                headers.add(parameter.parameter.name() + "2");
            }
        }
        return !deltas ? headers :
                headers.stream().map(n -> n + "-D").collect(Collectors.toList());
    }

    public static List<Integer> getParams(BoardState state) {
        List<Integer> params = new ArrayList<>();
        try {
            for (ParamEvaluation parameter : PARAMETERS) {
                if (parameter.parameter().absolute()) {
                    Integer value = (Integer) parameter.method().invoke(null, state);
                    params.add(value);
                } else if (parameter.parameter().signed()) {
                    Integer value = (Integer) parameter.method().invoke(null, state);
                    params.add(state.getSideToPlay() == Side.WHITE ? value : -value);
                } else if (parameter.parameter.perSide()) {
                    params.add((Integer) parameter.method().invoke(null, state, state.getSideToPlay()));
                    params.add((Integer) parameter.method().invoke(null, state, 1 - state.getSideToPlay()));
                }
            }
        } catch (IllegalAccessException | InvocationTargetException e) {
            throw new RuntimeException(e);
        }

        //signedParams =Arrays.asList(state.mg())
        return params;
    }

    public static List<Integer> makeDeltas(List<Integer> params, List<Integer> rootParams) {
        List<Integer> deltas = new ArrayList<>();
        int index = 0;
        for (ParamEvaluation parameter : PARAMETERS) {
            if (parameter.parameter().absolute()) {
                index++;
            } else if (parameter.parameter().signed()) {
                Integer value = -(rootParams.get(index) + params.get(index));
                deltas.add(value);
                index++;
            } else if (parameter.parameter.perSide()) {
                deltas.add(params.get(index + 1) - rootParams.get(index));
                deltas.add(params.get(index) - rootParams.get(index + 1));
                index += 2;
            }
        }
        return deltas;
    }

    @Parameter(name="move count", absolute=true)
    public static int moveCount(BoardState state) {
        return state.fullMoveNormalized;
    }

//    @Parameter(name="mg", signed=true)
//    public static int mg(BoardState state) {
//        return state.mg();
//    }
//
//    @Parameter(name="eg", signed=true)
//    public static int eg(BoardState state) {
//        return state.eg();
//    }

    // @Tuning(rangeFrom = 0, rangeTo = 20)
    @Parameter(name="pieces", perSide=true)
    public static int countOfPieces(BoardState state, int side) {
        return Long.bitCount(state.allPieces(side));
    }

    @Parameter(name="pawn count", perSide=true)
    public static int pawnCount(BoardState state, int side) {
        return Long.bitCount(state.bitboardOf(side, PieceType.PAWN));
    }

    @Parameter(name="bishop count", perSide=true)
    public static int bishopCount(BoardState state, int side) {
        return Long.bitCount(state.bitboardOf(side, PieceType.BISHOP));
    }

    @Parameter(name="queen count", perSide=true)
    public static int queenCount(BoardState state, int side) {
        return Long.bitCount(state.bitboardOf(side, PieceType.QUEEN));
    }
//    public static int evaluateForTune( BoardState state){
//        initEval(state);
//        int score = 0;
//
//        score += materialScore(state, Side.WHITE);
//        score -= materialScore(state, Side.BLACK);
//
//        score += pSqScore(state, Side.WHITE);
//        score -= pSqScore(state, Side.BLACK);
//
//        score += state.getSideToPlay() == Side.WHITE ? EConstants.TEMPO[0] : -EConstants.TEMPO[0];
//
//        score += Pawns.evaluate(state, Side.WHITE);
//        score -= Pawns.evaluate(state, Side.BLACK);
//
//        score += bishopScore(state, Side.WHITE);
//        score -= bishopScore(state, Side.BLACK);
//
//        score += rookScore(state, Side.WHITE);
//        score -= rookScore(state, Side.BLACK);
//
//        score += kingScore(state, Side.WHITE);
//        score -= kingScore(state, Side.BLACK);
//
//        return Score.eval(score, state.phase());
//    }


//    public static void main(String[] args) {
//        Method[] methods = Evaluation.class.getMethods();
//        for (Method method : methods) {
//            Parameter annotation = method.getAnnotation(Parameter.class);
//            if (annotation != null) {
//                System.out.println(method.getName());
//            }
//        }
//    }
}
