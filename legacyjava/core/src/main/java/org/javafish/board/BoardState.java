package org.javafish.board;

import org.javafish.bitboard.Bitboard;
import org.javafish.move.Move;
import org.javafish.move.MoveList;
import org.javafish.move.Zobrist;

import java.util.List;

import static org.javafish.Constants.CHESSBOARD_LINE;
import static org.javafish.bitboard.Bitboard.DARK_SQUARES;
import static org.javafish.bitboard.Bitboard.LIGHT_SQUARES;
import static org.javafish.bitboard.Bitboard.PAWN_DOUBLE_PUSH_LINES;
import static org.javafish.bitboard.Bitboard.PAWN_FINAL_RANKS;
import static org.javafish.bitboard.Bitboard.PAWN_RANKS;
import static org.javafish.bitboard.Bitboard.attacks;
import static org.javafish.bitboard.Bitboard.between;
import static org.javafish.bitboard.Bitboard.blackLeftPawnAttacks;
import static org.javafish.bitboard.Bitboard.blackRightPawnAttacks;
import static org.javafish.bitboard.Bitboard.getBishopAttacks;
import static org.javafish.bitboard.Bitboard.getKingAttacks;
import static org.javafish.bitboard.Bitboard.getKnightAttacks;
import static org.javafish.bitboard.Bitboard.getRookAttacks;
import static org.javafish.bitboard.Bitboard.ignoreOOODanger;
import static org.javafish.bitboard.Bitboard.line;
import static org.javafish.bitboard.Bitboard.pawnAttacks;
import static org.javafish.bitboard.Bitboard.whiteLeftPawnAttacks;
import static org.javafish.bitboard.Bitboard.whiteRightPawnAttacks;
import static org.javafish.board.Square.A1;
import static org.javafish.board.Square.A8;
import static org.javafish.board.Square.BACK;
import static org.javafish.board.Square.C1;
import static org.javafish.board.Square.C8;
import static org.javafish.board.Square.D1;
import static org.javafish.board.Square.D8;
import static org.javafish.board.Square.DOUBLE_FORWARD;
import static org.javafish.board.Square.E1;
import static org.javafish.board.Square.E8;
import static org.javafish.board.Square.F1;
import static org.javafish.board.Square.F8;
import static org.javafish.board.Square.FORWARD;
import static org.javafish.board.Square.FORWARD_LEFT;
import static org.javafish.board.Square.FORWARD_RIGHT;
import static org.javafish.board.Square.G1;
import static org.javafish.board.Square.G8;
import static org.javafish.board.Square.H1;
import static org.javafish.board.Square.H8;
import static org.javafish.board.Square.NO_SQUARE;
import static org.javafish.eval.PieceSquareTable.BASIC_MATERIAL_VALUE;
import static org.javafish.eval.PieceSquareTable.EGS;
import static org.javafish.eval.PieceSquareTable.MGS;

public class BoardState implements Cloneable {
    public static int TOTAL_PHASE = 24;
    public static int[] PIECE_PHASES = {0, 1, 1, 2, 4, 0};

    public int ply;
    private long[] history;
    private long[] piece_bb = new long[Piece.NPIECES];
    public int[] items = new int[64];
    private int sideToPlay;
    private long hash;
    // private long materialHash;
    public int fullMoveNormalized = 0;
    public int halfMoveClock = 0;
    public int phase = TOTAL_PHASE;

    // middle game score
    private int mg = 0;
    // end game score
    private int eg = 0;

    private long checkers;
    public long movements;
    public long enPassant;

    public BoardState(int[] items, int sideToPlay, long movements, long enPassantMask, int halfMoveClock, int fullMoveCount, int maxSearchDepth) {
//        if (items.length != 64) {
//            throw new IllegalStateException(String.format("Expected a vector with 64 elements, but got %s", items.length));
//        }
//
        for (int i = 0; i < 64; i++) {
            int item = items[i];
            if (item != Piece.NONE) {
                setPieceAt(item, i);
            } else {
                this.items[i] = Piece.NONE;
            }
        }

        this.sideToPlay = sideToPlay;

        if (sideToPlay == Side.BLACK)
            this.hash ^= Zobrist.SIDE;

        this.enPassant = enPassantMask;
        if (this.enPassant != 0) {
            this.hash ^= Zobrist.EN_PASSANT[(int) (this.enPassant & 0b111)];
        }

        this.movements = movements;

        this.halfMoveClock = halfMoveClock;
        this.fullMoveNormalized = (fullMoveCount - 1) * 2 + (sideToPlay == Side.WHITE ? 0 : 1);
        this.history = new long[maxSearchDepth];
        this.ply = 0;
    }

    public static BoardState fromFen(String fen) {
        return Fen.fromFen(fen, null);
    }

    public static BoardState fromFen(String fen, int maxSearchDepth) {
        return Fen.fromFen(fen, maxSearchDepth);
    }

    @Override
    protected BoardState clone() {
        try {
            BoardState result = (BoardState) super.clone();
            // result.score = (Score) this.score.clone();
            result.piece_bb = this.piece_bb.clone();
            result.items = this.items.clone();
            result.history = this.history.clone();
            return result;
        } catch (CloneNotSupportedException e) {
            throw new IllegalStateException(e);
        }
    }

//    public void clear(){
//        phase = EConstants.TOTAL_PHASE;
//        pSqScore = 0;
//        materialScore = 0;
//        // history = new UndoInfo[1000];
//        side_to_play = Side.WHITE;
//        // gamePly = 0;
//        for (int piece = 0; piece < Piece.NPIECES; piece++)
//            piece_bb[piece] = 0L;
//
//        for (int sq = Square.A1; sq <= Square.H8; sq++)
//            board[sq] = Piece.NONE;
//    }
//
    public int pieceAt(int square){
        return items[square];
    }

    public int pieceTypeAt(int square){
        return Piece.typeOf(items[square]);
    }

    public void setPieceAt(int piece, int square){

        //update incremental evaluation terms
        phase -= PIECE_PHASES[Piece.typeOf(piece)];
        mg += MGS[piece][square];
        eg += EGS[piece][square];
        // materialScore += materialValue(piece);

        //set piece on board
        items[square] = piece;
        piece_bb[piece] |= 1L << square;

        //update hashes
        hash ^= Zobrist.ZOBRIST_TABLE[piece][square];
        //materialHash ^= Zobrist.ZOBRIST_TABLE[piece][square];
    }

//    private int materialValue(int piece) {
//        boolean black = piece >= 6;
//        int mynew = EConstants.PIECE_TYPE_VALUES[piece & 0b111] * (black ? -1 : 1);
//        return mg(mynew);
//        // EConstants.PIECE_VALUES[piece]
//    }

//    private int pieceValue(int piece, int square) {
////        boolean black = piece >= 6;
////        // int mynew = EConstants.PIECE_TYPE_TABLES[piece & 0b111][black ? Square.squareMirror(square) : square] * (black ? -1 : 1);
////        int mynew = EConstants.PIECES_VALUES_COMPLETE[piece & 0b111][black ? Square.squareMirror(square) : square] * (black ? -1 : 1);
////        return mynew;
////
//        return MGS[piece][square];
//    }

    public void removePiece(int square){
        int piece = items[square];
        phase += PIECE_PHASES[Piece.typeOf(piece)];
        mg -= MGS[piece][square]; // EConstants.PIECE_TABLES[piece][square];
        eg -= EGS[piece][square];
        // materialScore -= materialValue(piece);

        //update hash tables
        hash ^= Zobrist.ZOBRIST_TABLE[items[square]][square];
        //materialHash ^= Zobrist.ZOBRIST_TABLE[items[square]][square];

        //update board
        piece_bb[items[square]] &= ~(1L << square);
        items[square] = Piece.NONE;
    }

    public void movePieceQuiet(int fromSq, int toSq){
        //update incremental evaluation terms
        int piece = items[fromSq];
        mg += MGS[piece][toSq] - MGS[piece][fromSq];
        eg += EGS[piece][toSq] - EGS[piece][fromSq];

        //update hashes
        hash ^= Zobrist.ZOBRIST_TABLE[piece][fromSq] ^ Zobrist.ZOBRIST_TABLE[piece][toSq];
        //materialHash ^= Zobrist.ZOBRIST_TABLE[piece][fromSq] ^ Zobrist.ZOBRIST_TABLE[piece][toSq];

        //update board
        piece_bb[piece] ^= (1L << fromSq | 1L << toSq);
        items[toSq] = piece;
        items[fromSq] = Piece.NONE;
    }

    public void movePiece(int fromSq, int toSq){
        removePiece(toSq);
        movePieceQuiet(fromSq, toSq);
    }

    public long hash(){
        return hash;
    }

//    public long materialHash(){
//        return materialHash;
//    }

    public long bitboardOf(int piece){
        return piece_bb[piece];
    }

    public long bitboardOf(int side, int pieceType){
        return piece_bb[Piece.makePiece(side, pieceType)];
    }

    public long checkers(){
        return checkers;
    }

    public long diagonalSliders(int side){
        return side == Side.WHITE ? piece_bb[Piece.WHITE_BISHOP] | piece_bb[Piece.WHITE_QUEEN] :
                                 piece_bb[Piece.BLACK_BISHOP] | piece_bb[Piece.BLACK_QUEEN];
    }

    public long orthogonalSliders(int side){
        return side == Side.WHITE ? piece_bb[Piece.WHITE_ROOK] | piece_bb[Piece.WHITE_QUEEN] :
                piece_bb[Piece.BLACK_ROOK] | piece_bb[Piece.BLACK_QUEEN];
    }

    public long allPieces(int side){
        return side == Side.WHITE ? piece_bb[Piece.WHITE_PAWN] | piece_bb[Piece.WHITE_KNIGHT] |
                                 piece_bb[Piece.WHITE_BISHOP] | piece_bb[Piece.WHITE_ROOK] |
                                 piece_bb[Piece.WHITE_QUEEN] | piece_bb[Piece.WHITE_KING] :

                                 piece_bb[Piece.BLACK_PAWN] | piece_bb[Piece.BLACK_KNIGHT] |
                                 piece_bb[Piece.BLACK_BISHOP] | piece_bb[Piece.BLACK_ROOK] |
                                 piece_bb[Piece.BLACK_QUEEN] | piece_bb[Piece.BLACK_KING];
    }

    public long allPieces() {
        return allPieces(Side.WHITE) | allPieces(Side.BLACK);
    }

    public long attackersFrom(int square, long occ, int side){
        return side == Side.WHITE ? (pawnAttacks(square, Side.BLACK) & piece_bb[Piece.WHITE_PAWN]) |
                (getKnightAttacks(square) & piece_bb[Piece.WHITE_KNIGHT]) |
                (getBishopAttacks(square, occ) & (piece_bb[Piece.WHITE_BISHOP] | piece_bb[Piece.WHITE_QUEEN])) |
                (getRookAttacks(square, occ) & (piece_bb[Piece.WHITE_ROOK] | piece_bb[Piece.WHITE_QUEEN])) :

                (pawnAttacks(square, Side.WHITE) & piece_bb[Piece.BLACK_PAWN]) |
                (getKnightAttacks(square) & piece_bb[Piece.BLACK_KNIGHT]) |
                (getBishopAttacks(square, occ) & (piece_bb[Piece.BLACK_BISHOP] | piece_bb[Piece.BLACK_QUEEN])) |
                (getRookAttacks(square, occ) & (piece_bb[Piece.BLACK_ROOK] | piece_bb[Piece.BLACK_QUEEN]));
    }

    public long attackersFromIncludingKings(int square, long occ, int side){
        return side == Side.WHITE ? (pawnAttacks(square, Side.BLACK) & piece_bb[Piece.WHITE_PAWN]) |
                (getKingAttacks(square) & piece_bb[Piece.WHITE_KING]) |
                (getKnightAttacks(square) & piece_bb[Piece.WHITE_KNIGHT]) |
                (getBishopAttacks(square, occ) & (piece_bb[Piece.WHITE_BISHOP] | piece_bb[Piece.WHITE_QUEEN])) |
                (getRookAttacks(square, occ) & (piece_bb[Piece.WHITE_ROOK] | piece_bb[Piece.WHITE_QUEEN])) :

                (pawnAttacks(square, Side.WHITE) & piece_bb[Piece.BLACK_PAWN]) |
                (getKingAttacks(square) & piece_bb[Piece.BLACK_KING]) |
                (getKnightAttacks(square) & piece_bb[Piece.BLACK_KNIGHT]) |
                (getBishopAttacks(square, occ) & (piece_bb[Piece.BLACK_BISHOP] | piece_bb[Piece.BLACK_QUEEN])) |
                (getRookAttacks(square, occ) & (piece_bb[Piece.BLACK_ROOK] | piece_bb[Piece.BLACK_QUEEN]));
    }

//    public void pushNull(){
//        gamePly++;
//        history[gamePly] = new UndoInfo();
//        history[gamePly].entry = history[gamePly - 1].entry;
//        history[gamePly].halfmoveCounter = history[gamePly - 1].halfmoveCounter + 1;
//        history[gamePly].pliesFromNull = 0;
//        history[gamePly].hash = history[gamePly - 1].hash;
//        history[gamePly].materialHash = history[gamePly - 1].materialHash;
//        if (history[gamePly - 1].epsq != Square.NO_SQUARE)
//            hash ^= Zobrist.EN_PASSANT[Square.getFile(history[gamePly - 1].epsq)];
//        hash ^= Zobrist.SIDE;
//        side_to_play = Side.flip(side_to_play);
//    }

//    public void popNull(){
//        gamePly--;
//        hash ^= Zobrist.SIDE;
//        if (history[gamePly].epsq != Square.NO_SQUARE)
//            hash ^= Zobrist.EN_PASSANT[Square.getFile(history[gamePly].epsq)];
//        side_to_play = Side.flip(side_to_play);
//    }


    public BoardState doMove(Move move) {
        return performMove(move, this);
    }

    public BoardState doMove(String uciMove) {
        return performMove(this.generateLegalMoves().stream().filter(m->m.toString().equals(uciMove)).findFirst().orElseThrow(), this);
    }

    public BoardState doNullMove() {
        return performNullMove(this);
    }

    private BoardState performNullMove(BoardState oldBoardState) {
        BoardState state = oldBoardState.clone();

        state.halfMoveClock += 1;
        state.clearEnPassant();
        state.sideToPlay = Side.flip(state.sideToPlay);
        state.hash ^= Zobrist.SIDE;

        //    public void pushNull(){
//        gamePly++;
//        history[gamePly] = new UndoInfo();
//        history[gamePly].entry = history[gamePly - 1].entry;
//        history[gamePly].halfmoveCounter = history[gamePly - 1].halfmoveCounter + 1;
//        history[gamePly].pliesFromNull = 0;
//        history[gamePly].hash = history[gamePly - 1].hash;
//        history[gamePly].materialHash = history[gamePly - 1].materialHash;
//        if (history[gamePly - 1].epsq != Square.NO_SQUARE)
//            hash ^= Zobrist.EN_PASSANT[Square.getFile(history[gamePly - 1].epsq)];
//        hash ^= Zobrist.SIDE;
//        side_to_play = Side.flip(side_to_play);
//    }
        return state;
    }


    public static BoardState performMove(Move move, BoardState oldBoardState) {
        BoardState state = oldBoardState.clone();

        state.fullMoveNormalized += 1;
        state.halfMoveClock += 1;
        state.history[state.ply++] = move.bits();
        state.movements |= (1L << move.to() | 1L << move.from());
        // board.pliesFromNull = history[gamePly - 1].pliesFromNull + 1;

        if (Piece.typeOf(state.items[move.from()]) == PieceType.PAWN)
            state.halfMoveClock = 0;

        state.clearEnPassant();

        switch (move.flags()){
            case Move.QUIET:
                state.movePieceQuiet(move.from(), move.to());
                break;
            case Move.DOUBLE_PUSH:
                state.movePieceQuiet(move.from(), move.to());
                state.enPassant = 1L << (move.from() + Square.direction(FORWARD, state.sideToPlay));
                state.hash ^= Zobrist.EN_PASSANT[(int) (state.enPassant & 0b111)];
                break;
            case Move.OO:
                if (state.sideToPlay == Side.WHITE){
                    state.movePieceQuiet(E1, G1);
                    state.movePieceQuiet(H1, F1);
                }
                else {
                    state.movePieceQuiet(E8, G8);
                    state.movePieceQuiet(H8, F8);
                }
                break;
            case Move.OOO:
                if (state.sideToPlay == Side.WHITE){
                    state.movePieceQuiet(E1, C1);
                    state.movePieceQuiet(A1, D1);
                }
                else {
                    state.movePieceQuiet(E8, C8);
                    state.movePieceQuiet(A8, D8);
                }
                break;
            case Move.EN_PASSANT:
                state.movePieceQuiet(move.from(), move.to());
                state.removePiece(move.to() + Square.direction(BACK, state.sideToPlay));
                break;
            case Move.PR_KNIGHT:
                state.removePiece(move.from());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, PieceType.KNIGHT), move.to());
                break;
            case Move.PR_BISHOP:
                state.removePiece(move.from());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, PieceType.BISHOP), move.to());
                break;
            case Move.PR_ROOK:
                state.removePiece(move.from());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, PieceType.ROOK), move.to());
                break;
            case Move.PR_QUEEN:
                state.removePiece(move.from());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, PieceType.QUEEN), move.to());
                break;
            case Move.PC_KNIGHT:
                state.removePiece(move.from());
                state.removePiece(move.to());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, PieceType.KNIGHT), move.to());
                break;
            case Move.PC_BISHOP:
                state.removePiece(move.from());
                state.removePiece(move.to());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, PieceType.BISHOP), move.to());
                break;
            case Move.PC_ROOK:
                state.removePiece(move.from());
                state.removePiece(move.to());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, PieceType.ROOK), move.to());
                break;
            case Move.PC_QUEEN:
                state.removePiece(move.from());
                state.removePiece(move.to());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, PieceType.QUEEN), move.to());
                break;
            case Move.CAPTURE:
                state.halfMoveClock = 0;
                state.movePiece(move.from(), move.to());
                break;
        }
        // board.hash = board.hash();
        //board.materialHash = board.materialHash;
        state.sideToPlay = Side.flip(state.sideToPlay);
        state.hash ^= Zobrist.SIDE;

        return state;
    }

    public int getSideToPlay(){
        return sideToPlay;
    }

    public boolean kingAttacked(){
        final int us = sideToPlay;
        final int them = Side.flip(sideToPlay);
        final int ourKing = Long.numberOfTrailingZeros(bitboardOf(us, PieceType.KING));

        if ((pawnAttacks(ourKing, us) & bitboardOf(them, PieceType.PAWN)) != 0)
            return true;

        if ((getKnightAttacks(ourKing) & bitboardOf(them, PieceType.KNIGHT)) != 0)
            return true;

        final long usBb = allPieces(us);
        final long themBb = allPieces(them);
        final long all = usBb | themBb;

        final long theirDiagSliders = diagonalSliders(them);
        final long theirOrthSliders = orthogonalSliders(them);

        if ((getRookAttacks(ourKing, all) & theirOrthSliders) != 0)
            return true;

        return (getBishopAttacks(ourKing, all) & theirDiagSliders) != 0;
    }

//    public boolean kingAttacked(int side) {
//        final int us = side;
//        final int them = Side.flip(side);
//        final int ourKing = Long.numberOfTrailingZeros(bitboardOf(us, PieceType.KING));
//
//        if ((Attacks.pawnAttacks(ourKing, us) & bitboardOf(them, PieceType.PAWN)) != 0)
//            return true;
//
//        if ((getKnightAttacks(ourKing) & bitboardOf(them, PieceType.KNIGHT)) != 0)
//            return true;
//
//        final long usBb = allPieces(us);
//        final long themBb = allPieces(them);
//        final long all = usBb | themBb;
//
//        final long theirDiagSliders = diagonalSliders(them);
//        final long theirOrthSliders = orthogonalSliders(them);
//
//        if ((getRookAttacks(ourKing, all) & theirOrthSliders) != 0)
//            return true;
//
//        return (getBishopAttacks(ourKing, all) & theirDiagSliders) != 0;
//    }

    /* not    side of the attacker */
    /**
     * @param side attacked side
     * @return attacked pieces
     */
    public long attackedPieces(int side) {
        BoardState workingState = this.getSideToPlay() == side ? this.doNullMove() : this;
        MoveList quiescence = workingState.generateLegalQuiescence();
        //BoardState finalWorkingState = workingState;
        List<Move> attackingMoves = quiescence.stream()
                .filter(m -> workingState.pieceAt(m.to()) != Piece.NONE)
                .toList();
        long result = 0L;
        for (Move move : attackingMoves) {
            result |= 1L << move.to();
        }
        return result;

//        final int us = side;
//        final int them = Side.flip(us);
//
//        final long usBb = allPieces(us);
//        final long themBb = allPieces(them);
//        final long all = usBb | themBb;
//
//        long ourKingBb = bitboardOf(us, PieceType.KING);
//        final int ourKing = Long.numberOfTrailingZeros(ourKingBb);
//        final int theirKing = Long.numberOfTrailingZeros(bitboardOf(them, PieceType.KING));
//        final long theirBishopsAndQueens = diagonalSliders(them);
//        final long theirRooksAndQueens = orthogonalSliders(them);
//
//        long squaresUnderAttack = 0;
//        squaresUnderAttack |= pawnAttacks(bitboardOf(them, PieceType.PAWN), them) | getKingAttacks(theirKing);
//
//        long work = bitboardOf(them, PieceType.KNIGHT);
//        while (work != 0){
//            squaresUnderAttack |= getKnightAttacks(Long.numberOfTrailingZeros(work));
//            work = Bitboard.extractLsb(work);
//        }
//
//
//        work = theirBishopsAndQueens;
//        while (work != 0){
//            squaresUnderAttack |= getBishopAttacks(Long.numberOfTrailingZeros(work), all ^ 1L << ourKing);
//            work = Bitboard.extractLsb(work);
//        }
//
//        work = theirRooksAndQueens;
//        while (work != 0){
//            squaresUnderAttack |= getRookAttacks(Long.numberOfTrailingZeros(work), all ^ 1L << ourKing);
//            work = Bitboard.extractLsb(work);
//        }
//
//        // work = getKingAttacks(ourKing) & ~(usBb | underAttack);
//
//        long ourPieces = allPieces(us);
//        return squaresUnderAttack & ourPieces;
    }

    /**
     * @param side attacked side
     * @return
     */
    public long attackedPiecesUndefended(int side) {
        int sideThem = Side.flip(side);
        final long usBb = allPieces(side);
        final long themBb = allPieces(sideThem);
        final long all = usBb | themBb;

        long attackedPieces = this.attackedPieces(side);
        long attackedUndefendedPieces = 0L;
        long work = attackedPieces;
        while (work != 0){
            int square = Long.numberOfTrailingZeros(work);
            long attackingPieces = attackersFromIncludingKings(square, all, sideThem);
            while (attackingPieces != 0) {
                int attackingSquare = Long.numberOfTrailingZeros(attackingPieces);
                long allWithoutAttacker = all & ~(1L << attackingSquare);
                long defendingPieces = attackersFromIncludingKings(square, allWithoutAttacker, side);
                if (defendingPieces == 0L) {
                    attackedUndefendedPieces |= 1L << square;
                }

                attackingPieces = Bitboard.extractLsb(attackingPieces);
            }
            work = Bitboard.extractLsb(work);
        }

        return attackedUndefendedPieces;
    }

    /**
     * @param side attacked side
     * @return attacked pieces
     */
    public long attackedPiecesUnderdefended(int side) {
        int sideThem = Side.flip(side);
//        final long usBb = allPieces(side);
//        final long themBb = allPieces(sideThem);
//        final long all = usBb | themBb;

//        BoardState workingState = this.getSideToPlay() == side ? this.doNullMove() : this;
//        MoveList quiescence = workingState.generateLegalQuiescence();


        long attackedPieces = this.attackedPieces(side);
        long attackedUnderdefendedPieces = 0L;
        long work = attackedPieces;
        while (work != 0){
            int square = Long.numberOfTrailingZeros(work);
            int score = this.seeScore(square, sideThem).score();
            if (score > 0) {
                attackedUnderdefendedPieces |= 1L << square;
            }
//            long attackingPieces = attackersFromIncludingKings(square, all, sideThem);
//            while (attackingPieces != 0) {
//                int attackingSquare = Long.numberOfTrailingZeros(attackingPieces);
//                long allWithoutAttacker = all & ~(1L << attackingSquare);
//                long defendingPieces = attackersFromIncludingKings(square, allWithoutAttacker, side);
//                if (defendingPieces == 0L) {
//                    attackedUnderdefendedPieces |= 1L << square;
//                }
//
//                attackingPieces = Bitboard.extractLsb(attackingPieces);
//            }
            work = Bitboard.extractLsb(work);
        }

        return attackedUnderdefendedPieces;
    }

    public int smallestAttackerWithKing(int square, int side) {
        return smallestAttacker(square, side, true);
    }

    public int smallestAttacker(int square, int side, boolean withAttackingKing){
        final int us = Side.flip(side);
        final int them = side;

        long pawns = pawnAttacks(square, us) & bitboardOf(them, PieceType.PAWN);
        if (pawns != 0)
            return Long.numberOfTrailingZeros(pawns);

        long knights = getKnightAttacks(square) & bitboardOf(them, PieceType.KNIGHT);
        if (knights != 0)
            return Long.numberOfTrailingZeros(knights);

        final long usBb = allPieces(us);
        final long themBb = allPieces(them);
        final long all = usBb | themBb;

        final long bishopAttacks = getBishopAttacks(square, all);
        long bishops = bishopAttacks & bitboardOf(them, PieceType.BISHOP);

        if (bishops != 0)
            return Long.numberOfTrailingZeros(bishops);

        final long rookAttacks = getRookAttacks(square, all);
        long rooks = rookAttacks & bitboardOf(them, PieceType.ROOK);
        if (rooks != 0)
            return Long.numberOfTrailingZeros(rooks);

        long queens = (bishopAttacks | rookAttacks) & bitboardOf(them, PieceType.QUEEN);
        if (queens != 0)
            return Long.numberOfTrailingZeros(queens);

        if (withAttackingKing) {
            long kings = getKingAttacks(square) & bitboardOf(them, PieceType.KING);
            if (kings != 0) {
                return Long.numberOfTrailingZeros(kings);
            }
        }

        return NO_SQUARE;
    }

    public boolean isInsufficientMaterial(int color){
        if ((bitboardOf(color, PieceType.PAWN) | bitboardOf(color, PieceType.ROOK) | bitboardOf(color, PieceType.QUEEN)) != 0)
            return false;

        long ourPieces = allPieces(color);
        long theirPieces = allPieces(Side.flip(color));
        if (bitboardOf(color, PieceType.KNIGHT) != 0)
            return Long.bitCount(ourPieces) <= 2 && (theirPieces & ~bitboardOf(Side.flip(color), PieceType.KING) & ~bitboardOf(Side.flip(color), PieceType.QUEEN)) == 0;

        long ourBishops = bitboardOf(color, PieceType.BISHOP);
        if (ourBishops != 0){
            boolean sameColor = (ourBishops & DARK_SQUARES) == 0 || (ourBishops & LIGHT_SQUARES) == 0;
            return sameColor && (bitboardOf(color, PieceType.PAWN) | bitboardOf(Side.flip(color), PieceType.PAWN)) == 0
                    || (bitboardOf(color, PieceType.KNIGHT) | bitboardOf(Side.flip(color), PieceType.KNIGHT)) == 0;
        }
        return true;
    }

    public boolean isRepetitionOrFifty(BoardPosition position){
        final long lastMoveBits = this.ply > 0 ? this.history[this.ply - 1] : position.history[position.historyIndex - 1];
        int count = 0;
        int index = this.ply - 1;
        while (index >= 0) {
            if (this.history[index--] == lastMoveBits) {
                count++;
            }
        }
        index = position.historyIndex - 1;
        while (index >= 0) {
            if (position.history[index--] == lastMoveBits) {
                count++;
            }
        }
        return count > 2 || this.halfMoveClock >= 100;

//        final int lookBack = Math.min(history[gamePly].pliesFromNull, history[gamePly].halfmoveCounter);
//        for (int i = 2; i <= lookBack; i += 2){
//            if (materialHash == history[gamePly - i].materialHash) {
//                return true;
//            }
//        }
    }

//    public static BoardState fromPosition(BoardPosition boardPosition) {
//    }

//    public boolean isDraw(MoveList moves){
//        if (history[gamePly].halfmoveCounter >= 100 || (checkers == 0 && moves.size() == 0))
//            return true;
//
//        final int lookBack = Math.min(history[gamePly].pliesFromNull, history[gamePly].halfmoveCounter);
//        int rep = 0;
//        for (int i = 2; i <= lookBack; i += 2){
//            if (materialHash == history[gamePly - i].materialHash
//            && ++rep >= 2) {
//                return true;
//            }
//        }
//        return false;
//    }

    public boolean hasNonPawnMaterial(int side) {
        int start = Piece.makePiece(side, PieceType.KNIGHT);
        int end = Piece.makePiece(side, PieceType.QUEEN);
        for (int piece = start; piece <= end; piece++){
            if (bitboardOf(piece) != 0)
                return true;
        }
        return false;
    }

    public MoveList generateLegalMoves(){
        return this.generateLegalMoves(false);
    }

    public MoveList generateLegalQuiescence(){
        return generateLegalMoves(true);
    }

    public MoveList generateLegalMoves(boolean onlyQuiescence) {
        MoveList moves = new MoveList();
        final int us = sideToPlay;
        final int them = Side.flip(sideToPlay);

        final long usBb = allPieces(us);
        final long themBb = allPieces(them);
        final long all = usBb | themBb;

        long ourKingBb = bitboardOf(us, PieceType.KING);
        final int ourKing = Long.numberOfTrailingZeros(ourKingBb);
        final int theirKing = Long.numberOfTrailingZeros(bitboardOf(them, PieceType.KING));

        final long ourBishopsAndQueens = diagonalSliders(us);
        final long theirBishopsAndQueens = diagonalSliders(them);
        final long ourRooksAndQueens = orthogonalSliders(us);
        final long theirRooksAndQueens = orthogonalSliders(them);

        // General purpose to keep down initialized primitives
        long b1, b2, b3;

        // Squares that the king can't move to
        long underAttack = 0;
        underAttack |= pawnAttacks(bitboardOf(them, PieceType.PAWN), them) | getKingAttacks(theirKing);

        b1 = bitboardOf(them, PieceType.KNIGHT);
        while (b1 != 0){
            underAttack |= getKnightAttacks(Long.numberOfTrailingZeros(b1));
            b1 = Bitboard.extractLsb(b1);
        }


        b1 = theirBishopsAndQueens;
        while (b1 != 0){
            underAttack |= getBishopAttacks(Long.numberOfTrailingZeros(b1), all ^ 1L << ourKing);
            b1 = Bitboard.extractLsb(b1);
        }

        b1 = theirRooksAndQueens;
        while (b1 != 0){
            underAttack |= getRookAttacks(Long.numberOfTrailingZeros(b1), all ^ 1L << ourKing);
            b1 = Bitboard.extractLsb(b1);
        }

        b1 = getKingAttacks(ourKing) & ~(usBb | underAttack);

        moves.makeQ(ourKing, b1 & ~themBb);
        moves.makeC(ourKing, b1 & themBb);

        //captureMask contains destinations where there is an enemy piece that is checking the king and must be captured
        //quietMask contains squares where pieces must be moved to block an incoming attack on the king
        long captureMask;
        long quietMask;
        int s;

        checkers = (getKnightAttacks(ourKing) & bitboardOf(them, PieceType.KNIGHT))
                | (pawnAttacks(ourKing, us) & bitboardOf(them, PieceType.PAWN));

        long candidates = (getRookAttacks(ourKing, themBb) & theirRooksAndQueens)
                | (getBishopAttacks(ourKing, themBb) & theirBishopsAndQueens);

        long pinned = 0;
        while (candidates != 0){
            s = Long.numberOfTrailingZeros(candidates);
            candidates = Bitboard.extractLsb(candidates);
            b1 = between(ourKing, s) & usBb;

            if (b1 == 0)
                checkers ^= 1L << s;
            else if (Bitboard.extractLsb(b1) == 0)
                pinned ^= b1;
        }

        final long notPinned = ~pinned;
        switch(Long.bitCount(checkers)){
            case 2:
                return moves;
            case 1: {
                int checkerSquare = Long.numberOfTrailingZeros(checkers);
                switch (Piece.typeOf(items[checkerSquare])){
                    case PieceType.PAWN:
                        // check to see if the checker is a pawn that can be captured ep
                        //if (checkers == Bitboard.shift(enPassant, Square.relative_dir(Square.SOUTH, us))){
                        if (checkers == (us == Side.WHITE ? enPassant >>> 8 : enPassant << 8)){
                            int enPassantSquare = Long.numberOfTrailingZeros(enPassant);
                            b1 = pawnAttacks(enPassantSquare, them) & bitboardOf(us, PieceType.PAWN) & notPinned;
                            while (b1 != 0){
                                moves.add(new Move(Long.numberOfTrailingZeros(b1), enPassantSquare, Move.EN_PASSANT));
                                b1 = Bitboard.extractLsb(b1);
                            }
                        }
                        // FALL THROUGH INTENTIONAL
                    case PieceType.KNIGHT:

                        b1 = attackersFrom(checkerSquare, all, us) & notPinned;
                        while (b1 != 0){
                            int sq = Long.numberOfTrailingZeros(b1);
                            b1 = Bitboard.extractLsb(b1);
                            if (pieceTypeAt(sq) == PieceType.PAWN && (1L << sq & PAWN_FINAL_RANKS) != 0L) {
                                moves.add(new Move(sq, checkerSquare, Move.PC_QUEEN));
                                moves.add(new Move(sq, checkerSquare, Move.PC_ROOK));
                                moves.add(new Move(sq, checkerSquare, Move.PC_KNIGHT));
                                moves.add(new Move(sq, checkerSquare, Move.PC_BISHOP));
                            }
                            else {
                                moves.add(new Move(sq, checkerSquare, Move.CAPTURE));
                            }
                        }
                        return moves;
                    default:
                        // We have to capture the checker
                        captureMask = checkers;
                        // ...or block it
                        quietMask = between(ourKing, checkerSquare);
                        break;
                }
                break;
            }
            default:
                captureMask = themBb;

                quietMask = ~all;

                if (enPassant != 0L) {
                    int enPassantSquare = Long.numberOfTrailingZeros(enPassant);
                    b2 = pawnAttacks(enPassantSquare, them) & bitboardOf(us, PieceType.PAWN);
                    // b2 holds pawns that can do an ep capture
                    b1 = b2 & notPinned;
                    while (b1 != 0) {
                        // s hold square from which pawn attack to epsq can be done
                        s = Long.numberOfTrailingZeros(b1);
                        b1 = Bitboard.extractLsb(b1);

//                        long attacks = Attacks.slidingAttacks(ourKing,
//                                all ^ 1L << s) ^ Bitboard.shift(1L << this.epsq), Square.relative_dir(Square.SOUTH, us)),
//                                Rank.getBb(Square.getRank(ourKing)));

                        // Bitboard.shift(1L << this.epsq), Square.relative_dir(Square.SOUTH, us)) holds pawn which can be en-passant taken
                        long qqq = themBb ^ (us == Side.WHITE ? enPassant >>> 8 : enPassant << 8);
                        candidates = (getRookAttacks(ourKing, qqq | usBb) & theirRooksAndQueens)
                                | (getBishopAttacks(ourKing, qqq | usBb) & theirBishopsAndQueens);

                        if (candidates == 0 /*&& (attacks & theirOrthSliders) == 0*/)
                            moves.add(new Move(s, enPassantSquare, Move.EN_PASSANT));
                    }
                }

                if (!onlyQuiescence) {
                    if (0 == ((this.movements & Bitboard.castlingPiecesKingsideMask(us)) | ((all | underAttack) & Bitboard.castlingBlockersKingsideMask(us))))
                        moves.add(us == Side.WHITE ? new Move(E1, G1, Move.OO) : new Move(E8, G8, Move.OO));

                    if (0 == ((this.movements & Bitboard.castlingPiecesQueensideMask(us)) |
                            ((all | (underAttack & ~ignoreOOODanger(us))) & Bitboard.castlingBlockersQueensideMask(us))))
                        moves.add(us == Side.WHITE ? new Move(E1, C1, Move.OOO) : new Move(E8, C8, Move.OOO));
                }

                // For each pinned rook, bishop, or queen...
                b1 = ~(notPinned | bitboardOf(us, PieceType.KNIGHT));
                while (b1 != 0){
                    s = Long.numberOfTrailingZeros(b1);
                    b1 = Bitboard.extractLsb(b1);

                    b2 = attacks(Piece.typeOf(items[s]), s, all) & line(ourKing, s);
                    if (!onlyQuiescence) {
                        moves.makeQ(s, b2 & quietMask);
                    }
                    moves.makeC(s, b2 & captureMask);
                }

                // for each pinned pawn
                b1 = ~notPinned & bitboardOf(us, PieceType.PAWN);
                while (b1 != 0){
                    s = Long.numberOfTrailingZeros(b1);
                    b1 = Bitboard.extractLsb(b1);

                    if (((1L << s) & PAWN_FINAL_RANKS) != 0L) {
                        b2 = pawnAttacks(s, us) & captureMask & line(ourKing, s);
                        moves.makePC(s, b2);
                    }
                    else{
                        b2 = pawnAttacks(s, us) & themBb & line(s, ourKing);
                        moves.makeC(s, b2);

                        if (!onlyQuiescence) {
                            //single pawn pushes
                            b2 = Bitboard.push(1L << s, us) & ~all & line(ourKing, s);
                            b3 = Bitboard.push(b2 & PAWN_DOUBLE_PUSH_LINES[us], us) & ~all & line(ourKing, s);

                            moves.makeQ(s, b2);
                            moves.makeDP(s, b3);
                        }
                    }
                }
                //Pinned knights cannot move anywhere, so we're done with pinned pieces.
                break;

        }

        //non-pinned knight moves.
        b1 = bitboardOf(us, PieceType.KNIGHT) & notPinned;
        while (b1 != 0){
            s = Long.numberOfTrailingZeros(b1);
            b1 = Bitboard.extractLsb(b1);
            b2 = getKnightAttacks(s);
            moves.makeC(s, b2 & captureMask);
            if (!onlyQuiescence) {
                moves.makeQ(s, b2 & quietMask);
            }
        }

        b1 = ourBishopsAndQueens & notPinned;
        while (b1 != 0){
            s = Long.numberOfTrailingZeros(b1);
            b1 = Bitboard.extractLsb(b1);
            b2 = getBishopAttacks(s, all);
            moves.makeC(s, b2 & captureMask);
            if (!onlyQuiescence) {
                moves.makeQ(s, b2 & quietMask);
            }
        }

        b1 = ourRooksAndQueens & notPinned;
        while(b1 != 0){
            s = Long.numberOfTrailingZeros(b1);
            b1 = Bitboard.extractLsb(b1);
            b2 = getRookAttacks(s, all);
            moves.makeC(s, b2 & captureMask);
            if (!onlyQuiescence) {
                moves.makeQ(s, b2 & quietMask);
            }
        }

        b1 = bitboardOf(us, PieceType.PAWN) & notPinned & ~PAWN_RANKS[us];

        if (!onlyQuiescence) {
            // single pawn pushes
            b2 = (us == Side.WHITE ? b1 << 8 : b1 >>> 8) & ~all;

            //double pawn pushes
            b3 = Bitboard.push(b2 & PAWN_DOUBLE_PUSH_LINES[us], us) & quietMask;

            b2 &= quietMask;

            while (b2 != 0) {
                s = Long.numberOfTrailingZeros(b2);
                b2 = Bitboard.extractLsb(b2);
                moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.QUIET));
            }

            while (b3 != 0) {
                s = Long.numberOfTrailingZeros(b3);
                b3 = Bitboard.extractLsb(b3);
                moves.add(new Move(s - Square.direction(DOUBLE_FORWARD, us), s, Move.DOUBLE_PUSH));
            }
        }

        b2 = (us == Side.WHITE ? whiteLeftPawnAttacks(b1) : blackRightPawnAttacks(b1)) & captureMask;
        b3 = (us == Side.WHITE ? whiteRightPawnAttacks(b1) : blackLeftPawnAttacks(b1)) & captureMask;


        while (b2 != 0){
            s = Long.numberOfTrailingZeros(b2);
            b2 = Bitboard.extractLsb(b2);
            moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.CAPTURE));
        }

        while (b3 != 0){
            s = Long.numberOfTrailingZeros(b3);
            b3 = Bitboard.extractLsb(b3);
            moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.CAPTURE));
        }

        b1 = bitboardOf(us, PieceType.PAWN) & notPinned & PAWN_RANKS[us];
        if (b1 != 0){
            if (!onlyQuiescence) {
                b2 = (us == Side.WHITE ? b1 << 8 : b1 >>> 8) & quietMask;
                while (b2 != 0) {
                    s = Long.numberOfTrailingZeros(b2);
                    b2 = Bitboard.extractLsb(b2);

                    moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.PR_QUEEN));
                    moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.PR_ROOK));
                    moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.PR_KNIGHT));
                    moves.add(new Move(s - Square.direction(FORWARD, us), s, Move.PR_BISHOP));
                }
            }

//            b2 = Bitboard.shift(b1, Square.relative_dir(Square.NORTH_WEST, us)) & captureMask;
//            b3 = Bitboard.shift(b1, Square.relative_dir(Square.NORTH_EAST, us)) & captureMask;
            b2 = (us == Side.WHITE ? whiteLeftPawnAttacks(b1) : blackRightPawnAttacks(b1)) & captureMask;
            b3 = (us == Side.WHITE ? whiteRightPawnAttacks(b1) : blackLeftPawnAttacks(b1)) & captureMask;

            while (b2 != 0){
                s = Long.numberOfTrailingZeros(b2);
                b2 = Bitboard.extractLsb(b2);

                moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_QUEEN));
                moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_ROOK));
                moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_KNIGHT));
                moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_BISHOP));
            }

            while (b3 != 0){
                s = Long.numberOfTrailingZeros(b3);
                b3 = Bitboard.extractLsb(b3);

                moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_QUEEN));
                moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_ROOK));
                moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_KNIGHT));
                moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_BISHOP));
            }
        }

        return moves;
    }

//    public MoveList generateLegalQuiescence(){
//        MoveList moves = new MoveList();
//        final int us = side_to_play;
//        final int them = Side.flip(side_to_play);
//
//        final long usBb = allPieces(us);
//        final long themBb = allPieces(them);
//        final long all = usBb | themBb;
//
//        final int ourKing = Long.numberOfTrailingZeros(bitboardOf(us, PieceType.KING));
//        final int theirKing = Long.numberOfTrailingZeros(bitboardOf(them, PieceType.KING));
//
//        final long ourDiagSliders = diagonalSliders(us);
//        final long theirDiagSliders = diagonalSliders(them);
//        final long ourOrthSliders = orthogonalSliders(us);
//        final long theirOrthSliders = orthogonalSliders(them);
//
//        long b1, b2, b3;
//
//        long danger = 0;
//        danger |= Attacks.pawnAttacks(bitboardOf(them, PieceType.PAWN), them) | Attacks.getKingAttacks(theirKing);
//
//        b1 = bitboardOf(them, PieceType.KNIGHT);
//        while (b1 != 0){
//            danger |= getKnightAttacks(Long.numberOfTrailingZeros(b1));
//            b1 = MyBitboard.extractLsb(b1);
//        }
//
//        b1 = theirDiagSliders;
//        while(b1 != 0){
//            danger |= getBishopAttacks(Long.numberOfTrailingZeros(b1), all ^ 1L << ourKing));
//            b1 = MyBitboard.extractLsb(b1);
//        }
//
//        b1 = theirOrthSliders;
//        while (b1 != 0){
//            danger |= getRookAttacks(Long.numberOfTrailingZeros(b1), all ^ 1L << ourKing));
//            b1 = MyBitboard.extractLsb(b1);
//        }
//
//
//        b1 = Attacks.getKingAttacks(ourKing) & ~(usBb | danger);
//
//        moves.makeC(ourKing, b1 & themBb);
//
//        long captureMask;
//        int s;
//
//        checkers = (getKnightAttacks(ourKing) & bitboardOf(them, PieceType.KNIGHT))
//                | (Attacks.pawnAttacks(ourKing, us) & bitboardOf(them, PieceType.PAWN));
//
//        long candidates = (getRookAttacks(ourKing, themBb) & theirOrthSliders)
//                | (getBishopAttacks(ourKing, themBb) & theirDiagSliders);
//
//        long pinned = 0;
//        while (candidates != 0){
//            s = Long.numberOfTrailingZeros(candidates);
//            candidates = MyBitboard.extractLsb(candidates);
//            b1 = Bitboard.between(ourKing, s) & usBb;
//
//            if (b1 == 0)
//                checkers ^= 1L << s);
//            else if (MyBitboard.extractLsb(b1) == 0)
//                pinned ^= b1;
//        }
//
//        final long notPinned = ~pinned;
//        switch(Long.bitCount(checkers)){
//            case 2:
//                return moves;
//            case 1:{
//                int checkerSquare = Long.numberOfTrailingZeros(checkers);
//                switch (Piece.typeOf(items[checkerSquare])){
//                    case PieceType.PAWN:
//                        // check to see if the checker is a pawn that can be captured ep
//                        if (checkers == Bitboard.shift(enPassant, Square.relative_dir(Square.SOUTH, us))){
//                            int enPassantSquare = Long.numberOfTrailingZeros(enPassant);
//                            b1 = Attacks.pawnAttacks(enPassantSquare, them) & bitboardOf(us, PieceType.PAWN) & notPinned;
//                            while (b1 != 0) {
//                                moves.add(new Move(Long.numberOfTrailingZeros(b1), enPassantSquare, Move.EN_PASSANT));
//                                b1 = MyBitboard.extractLsb(b1);
//                            }
//                        }
//                        // FALL THROUGH INTENTIONAL
//                    case PieceType.KNIGHT:
//                        b1 = attackersFrom(checkerSquare, all, us) & notPinned;
//                        while (b1 != 0){
//                            int sq = Long.numberOfTrailingZeros(b1);
//                            b1 = MyBitboard.extractLsb(b1);
//                            if (pieceTypeAt(sq) == PieceType.PAWN && Rank.relativeRank(Square.getRank(sq), side_to_play) == Rank.RANK_7){
//                                moves.add(new Move(sq, checkerSquare, Move.PC_QUEEN));
//                                moves.add(new Move(sq, checkerSquare, Move.PC_ROOK));
//                                moves.add(new Move(sq, checkerSquare, Move.PC_KNIGHT));
//                                moves.add(new Move(sq, checkerSquare, Move.PC_BISHOP));
//                            }
//                            else {
//                                moves.add(new Move(sq, checkerSquare, Move.CAPTURE));
//                            }
//                        }
//                        return moves;
//                    default:
//                        // We have to capture the checker
//                        captureMask = checkers;
//                        // ...or block it
//                        break;
//                }
//                break;
//            }
//            default:
//                // No checkers, we can capture any enemy piece.
//                captureMask = themBb;
//
//
//                if (enPassant != 0L){
//                    int enPassantSquare = Long.numberOfTrailingZeros(enPassant);
//                    //b1 contains pawns that can do an ep capture
//                    b1 = Attacks.pawnAttacks(enPassantSquare, them) & bitboardOf(us, PieceType.PAWN) & notPinned;
//                    while (b1 != 0){
//                        s = Long.numberOfTrailingZeros(b1);
//                        b1 = MyBitboard.extractLsb(b1);
//
//                        long attacks = Attacks.slidingAttacks(ourKing,
//                                all ^ 1L << s) ^ Bitboard.shift(1L << enPassantSquare), Square.relative_dir(Square.SOUTH, us)),
//                                Rank.getBb(Square.getRank(ourKing)));
//
//                        if ((attacks & theirOrthSliders) == 0)
//                            moves.add(new Move(s, enPassantSquare, Move.EN_PASSANT));
//                    }
//                }
//
//
//                // For each pinned rook, bishop, or queen...
//                b1 = ~(notPinned | bitboardOf(us, PieceType.KNIGHT));
//                while (b1 != 0){
//                    s = Long.numberOfTrailingZeros(b1);
//                    b1 = MyBitboard.extractLsb(b1);
//
//                    b2 = Attacks.attacks(Piece.typeOf(items[s]), s, all) & Bitboard.line(ourKing, s);
//                    moves.makeC(s, b2 & captureMask);
//                }
//
//                // for each pinned pawn
//                b1 = ~notPinned & bitboardOf(us, PieceType.PAWN);
//                while (b1 != 0){
//                    s = Long.numberOfTrailingZeros(b1);
//                    b1 = MyBitboard.extractLsb(b1);
//
//                    if (Square.getRank(s) == Rank.relativeRank(Rank.RANK_7, us)){
//                        b2 = Attacks.pawnAttacks(s, us) & captureMask & Bitboard.line(ourKing, s);
//                        moves.makePC(s, b2);
//                    }
//                    else{
//                        b2 = Attacks.pawnAttacks(s, us) & themBb & Bitboard.line(s, ourKing);
//                        moves.makeC(s, b2);
//
//                    }
//
//                }
//                //Pinned knights cannot move anywhere, so we're done with pinned pieces.
//                break;
//
//        }
//        //non-pinned knight moves.
//        b1 = bitboardOf(us, PieceType.KNIGHT) & notPinned;
//        while (b1 != 0){
//            s = Long.numberOfTrailingZeros(b1);
//            b1 = MyBitboard.extractLsb(b1);
//            b2 = getKnightAttacks(s);
//            moves.makeC(s, b2 & captureMask);
//        }
//
//        b1 = ourDiagSliders & notPinned;
//        while (b1 != 0){
//            s = Long.numberOfTrailingZeros(b1);
//            b1 = MyBitboard.extractLsb(b1);
//            b2 = getBishopAttacks(s, all);
//            moves.makeC(s, b2 & captureMask);
//        }
//
//        b1 = ourOrthSliders & notPinned;
//        while(b1 != 0){
//            s = Long.numberOfTrailingZeros(b1);
//            b1 = MyBitboard.extractLsb(b1);
//            b2 = getRookAttacks(s, all);
//            moves.makeC(s, b2 & captureMask);
//        }
//
//        b1 = bitboardOf(us, PieceType.PAWN) & notPinned & ~Rank.getBb(Rank.relativeRank(Rank.RANK_7, us));
//        b2 = Bitboard.shift(b1, Square.relative_dir(Square.NORTH_WEST, us)) & captureMask;
//        b3 = Bitboard.shift(b1, Square.relative_dir(Square.NORTH_EAST, us)) & captureMask;
//
//        while (b2 != 0){
//            s = Long.numberOfTrailingZeros(b2);
//            b2 = MyBitboard.extractLsb(b2);
//            moves.add(new Move(s - Square.relative_dir(Square.NORTH_WEST, us), s, Move.CAPTURE));
//        }
//
//        while (b3 != 0){
//            s = Long.numberOfTrailingZeros(b3);
//            b3 = MyBitboard.extractLsb(b3);
//            moves.add(new Move(s - Square.relative_dir(Square.NORTH_EAST, us), s, Move.CAPTURE));
//        }
//
//        b1 = bitboardOf(us, PieceType.PAWN) & notPinned & Rank.getBb(Rank.relativeRank(Rank.RANK_7, us));
//        if (b1 != 0){
//            b2 = Bitboard.shift(b1, Square.relative_dir(Square.NORTH_WEST, us)) & captureMask;
//            b3 = Bitboard.shift(b1, Square.relative_dir(Square.NORTH_EAST, us)) & captureMask;
//
//            while (b2 != 0){
//                s = Long.numberOfTrailingZeros(b2);
//                b2 = MyBitboard.extractLsb(b2);
//
//                moves.add(new Move(s - Square.relative_dir(Square.NORTH_WEST, us), s, Move.PC_QUEEN));
//                moves.add(new Move(s - Square.relative_dir(Square.NORTH_WEST, us), s, Move.PC_ROOK));
//                moves.add(new Move(s - Square.relative_dir(Square.NORTH_WEST, us), s, Move.PC_KNIGHT));
//                moves.add(new Move(s - Square.relative_dir(Square.NORTH_WEST, us), s, Move.PC_BISHOP));
//
//            }
//
//            while (b3 != 0){
//                s = Long.numberOfTrailingZeros(b3);
//                b3 = MyBitboard.extractLsb(b3);
//
//                moves.add(new Move(s - Square.relative_dir(Square.NORTH_EAST, us), s, Move.PC_QUEEN));
//                moves.add(new Move(s - Square.relative_dir(Square.NORTH_EAST, us), s, Move.PC_ROOK));
//                moves.add(new Move(s - Square.relative_dir(Square.NORTH_EAST, us), s, Move.PC_KNIGHT));
//                moves.add(new Move(s - Square.relative_dir(Square.NORTH_EAST, us), s, Move.PC_BISHOP));
//
//            }
//        }
//
//        return moves;
//    }

//    public int gamePly(){
//        return gamePly;
//    }

//    public int materialScore(){
//        return materialScore;
//    }


//    public String toFen(){
//        StringBuilder fen = new StringBuilder();
//        int count = 0;
//        int rankCounter = 1;
//        int sqCount = 0;
//        for(int rank = 7; rank >= 0; rank--){
//            for(int file = 0; file <= 7; file++){
//                int square = Square.encode(rank, file);
//                int piece = getPieceAt(square);
//                if (piece != Piece.NONE){
//                    if (count > 0) {
//                        fen.append(count);
//                    }
//                    fen.append(Piece.getNotation(piece));
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
//        if (side_to_play == Side.WHITE){
//            fen.append(" w");
//        }
//        else{
//            fen.append(" b");
//        }
//
//        String rights = "";
//        if ((Bitboard.ooMask(Side.WHITE) & this.entry) != 0)
//            rights += "K";
//        if((Bitboard.oooMask(Side.WHITE) & this.entry) != 0)
//            rights += "Q";
//        if((Bitboard.ooMask(Side.BLACK) & this.entry) != 0)
//            rights += "k";
//        if((Bitboard.oooMask(Side.BLACK) & this.entry) != 0)
//            rights += "q";
//
//        if (rights.equals("")){
//            fen.append(" -");
//        }
//        else{
//            fen.append(" ").append(rights);
//        }
//
//        if (this.epsq != Square.NO_SQUARE)
//            fen.append(" ").append(Square.getName(this.epsq));
//        else
//            fen.append(" -");
//
//        // TODO doplnit halfmove_clock
//        fen.append(' ').append(0);
//        // TODO
//        fen.append(' ').append(0);
//
//        return fen.toString();
//    }

    @Override
    public String toString() {
        StringBuilder result = new StringBuilder(CHESSBOARD_LINE);
        for (int i = 56; i >= 0; i -= 8){
            for (int j = 0; j < 8; j++){
                int piece = items[i + j];
                String notation = Piece.getNotation(piece);
                result.append("| ").append(notation).append(' ');
            }
            result.append("|\n").append(CHESSBOARD_LINE);
        }
        result.append("Fen: ").append(Fen.toFen(this));
        return result.toString();
    }

    private void clearEnPassant() {
        // TODO zjednodusit
        long previous_state = this.enPassant;

        if (previous_state != 0L) {
            this.enPassant = 0L;
            this.hash ^= Zobrist.EN_PASSANT[(int) (previous_state & 0b111)];
        }
    }

    public BoardState forSearchDepth(int searchDepth) {
        BoardState result = this.clone();
        result.history = new long[searchDepth];
        result.ply = 0;
        return result;
    }

    public String toFen() {
        return Fen.toFen(this);
    }

    public int mg() {
        return mg;
    }

    public int eg() {
        return eg;
    }

    public int interpolatedScore() {
        int phase = (this.phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE;
        return (this.mg() * (256 - phase) + this.eg() * phase) / 256;
    }

    public boolean isCheck() {
        this.generateLegalMoves();
        return this.checkers != 0L;
    }

    public boolean isCheckMate() {
        return this.generateLegalMoves().size() == 0;
    }

    /**
     * @param attackerSquare attacker square
     * @param attackedSide attacked side
     * @return pinned pieces
     */
    public long pinnedPieces(int attackerSquare, int attackedSide) {
        final int pieceType = this.pieceTypeAt(attackerSquare);
        final int us = Side.flip(attackedSide);
        final int them = attackedSide;

        final long usBb = allPieces(us);
        final long themBb = allPieces(them);
        final long all = usBb | themBb;

        long attacked = 0;
        if (pieceType == PieceType.ROOK || pieceType == PieceType.QUEEN) {
            attacked |= getRookAttacks(attackerSquare, all);
        }
        if (pieceType == PieceType.BISHOP || pieceType == PieceType.QUEEN) {
            attacked |= getBishopAttacks(attackerSquare, all);
        }
        attacked &= themBb;

        long pinned = 0;
        long temp = attacked;
        while (temp != 0){
            int square = Long.numberOfTrailingZeros(temp);
            long examinedMask = 1L << square;
            long allExceptOne = all & (~examinedMask);

            long pinnedCandidates = 0;
            if (pieceType == PieceType.ROOK || pieceType == PieceType.QUEEN) {
                pinnedCandidates |= getRookAttacks(attackerSquare, allExceptOne);
            }
            if (pieceType == PieceType.BISHOP || pieceType == PieceType.QUEEN) {
                pinnedCandidates |= getBishopAttacks(attackerSquare, allExceptOne);
            }
            pinnedCandidates &= (themBb & allExceptOne);
            long pinnedPiece = pinnedCandidates & (~attacked);
            pinned |= pinnedPiece;

            temp = Bitboard.extractLsb(temp);
        }

        pinned = pinned & (~this.bitboardOf(them, PieceType.PAWN));
        return pinned;
    }

    public record ScoreOutcome(int score, int piecesTaken) {}

    /**
     * @param square battle square
     * @param side perspective of score, starting move
     * @return score in basic material values, the higher, the better, no matter if white or black
     */
    public ScoreOutcome seeScore(int square, int side) {
        int processedSide = side;
        int score = 0;
        int piecesTaken = 0;
        BoardState evaluatedState = this.getSideToPlay() != processedSide ? this.doNullMove() : this;

        while (true) {
//            if (evaluatedState.pieceAt(square) == Piece.NONE) { // TODO mozna nedovolit aby vubec nastavalo
//                break;
//            }

            int attacker = evaluatedState.smallestAttackerWithKing(square, processedSide);
            if (attacker == NO_SQUARE) {
                break;
            }
            List<Move> possibleMoves = evaluatedState.generateLegalMoves(true)
                    .stream()
                    .filter(m -> m.from() == attacker && m.to() == square)
                    .toList();
            if (possibleMoves.size() == 0) {
                break;
            }
            // for promotion, Q move is always first, only this move is considered
//            if (possibleMoves.size() > 1) {
//                throw new IllegalStateException(String.format("There are %d possible moves. Not implemented yet.", possibleMoves.size()));
//            }
            int pieceType = evaluatedState.pieceTypeAt(square);
            if (pieceType == PieceType.KING) {
                score = 0;
                break;
            }
            score += evaluatedState.getBasicMaterialValue(square);
            piecesTaken++;
            processedSide = Side.flip(processedSide);
            evaluatedState = evaluatedState.doMove(possibleMoves.get(0));
        }
        return new ScoreOutcome(-score * Side.multiplicator(side), piecesTaken);
    }

    private int getBasicMaterialValue(int square) {
        int piece = pieceAt(square);
        return BASIC_MATERIAL_VALUE[Piece.typeOf(piece)] * (Piece.sideOf(piece) == Side.WHITE ? 1 : -1);
    }

    public boolean isCapture(String move) {
        Move parsedMove = Move.fromUciString(move);
        return this.pieceAt(parsedMove.to()) != Piece.NONE;
    }

//    public byte[] toParams() {
//        byte[] result = new byte[96]; // 8 * 6 * 2
//        int index = 0;
//        for (int side = Side.WHITE; side <= Side.BLACK; side++) {
//            for (int piece = PieceType.PAWN; piece <= PieceType.KING; piece++) {
//                if (this.side_to_play == Side.BLACK) {
//                    long bitboard = this.bitboardOf(1 - side, piece);
//                    for (int i = 0; i < 8; i++) {
//                        result[index++] = (byte)(bitboard & 0xFFL);
//                        bitboard >>= 8;
//                    }
//                } else {
//                    long bitboard = this.bitboardOf(side, piece);
//                    for (int i = 0; i < 8; i++) {
//                        result[index++] = (byte)((bitboard & 0xFF00000000000000L) >> 56);
//                        bitboard <<= 8;
//                    }
//                }
//            }
//
//        }
//        return result;
//    }

    public record Params(byte[] pieces, int wKingPos, int bKingPos) {}

    public Params toParams() {
        byte[] result = new byte[80]; // 8 * 5 * 2
        int index = 0;
        for (int side = Side.WHITE; side <= Side.BLACK; side++) {
            for (int piece = PieceType.PAWN; piece <= PieceType.QUEEN; piece++) {
                long bitboard = this.bitboardOf(side, piece);
                for (int i = 0; i < 8; i++) {
                    result[index++] = (byte)((bitboard & 0xFF00000000000000L) >> 56);
                    bitboard <<= 8;
                }
            }

        }
        return new Params(result,
            Long.numberOfTrailingZeros(bitboardOf(Side.WHITE, PieceType.KING)),
            Long.numberOfTrailingZeros(bitboardOf(Side.BLACK, PieceType.KING)));
    }
}


