package org.javafish.board;

import org.javafish.bitboard.Bitboard;
import org.javafish.move.Move;
import org.javafish.move.MoveList;
import org.javafish.move.Zobrist;

import java.util.List;
import java.util.function.BiConsumer;
import java.util.function.Function;

import static org.javafish.Constants.CHESSBOARD_LINE;
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

    private long[] piece_bb = new long[Piece.PIECES_COUNT];
    public int[] items = new int[64];
    private int sideToPlay;
    private long hash;
    public int fullMoveNormalized = 0;
    public int halfMoveClock = 0;
    public int phase = TOTAL_PHASE;

    private int mg = 0;
    private int eg = 0;

    private long checkers;
    public long movements;
    public long enPassant;

    public BoardState(int[] items, int sideToPlay, long movements, long enPassantMask, int halfMoveClock, int fullMoveCount) {
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
            this.hash ^= Zobrist.EN_PASSANT[(Bitboard.lsb(this.enPassant) & 0b111)];
        }

        this.movements = movements;

        this.halfMoveClock = halfMoveClock;
        this.fullMoveNormalized = (fullMoveCount - 1) * 2 + (sideToPlay == Side.WHITE ? 0 : 1);
    }

    public static BoardState fromFen(String fen) {
        return Fen.fromFen(fen, null);
    }

    public static BoardState fromFen(String fen, int maxSearchDepth) {
        return Fen.fromFen(fen, maxSearchDepth);
    }

    @Override
    protected final BoardState clone() {
        try {
            BoardState result = (BoardState) super.clone();
            result.piece_bb = this.piece_bb.clone();
            result.items = this.items.clone();
            return result;
        } catch (CloneNotSupportedException e) {
            throw new IllegalStateException(e);
        }
    }

    public final int pieceAt(int square){
        return items[square];
    }

    public final int pieceTypeAt(int square){
        return Piece.typeOf(items[square]);
    }

    public final void setPieceAt(int piece, int square){

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
    }

    public final void removePiece(int square){
        int piece = items[square];
        phase += PIECE_PHASES[Piece.typeOf(piece)];
        mg -= MGS[piece][square]; // EConstants.PIECE_TABLES[piece][square];
        eg -= EGS[piece][square];

        //update hash tables
        hash ^= Zobrist.ZOBRIST_TABLE[items[square]][square];

        //update board
        piece_bb[items[square]] &= ~(1L << square);
        items[square] = Piece.NONE;
    }

    public final void movePieceQuiet(int fromSq, int toSq){
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

    public final void movePiece(int fromSq, int toSq){
        removePiece(toSq);
        movePieceQuiet(fromSq, toSq);
    }

    public final long hash(){
        return hash;
    }

    public final long bitboardOf(int piece){
        return piece_bb[piece];
    }

    public final long bitboardOf(int side, int pieceType){
        return piece_bb[Piece.makePiece(side, pieceType)];
    }

    public final long diagonalSliders(int side){
        return side == Side.WHITE ? piece_bb[Piece.WHITE_BISHOP] | piece_bb[Piece.WHITE_QUEEN] :
                                 piece_bb[Piece.BLACK_BISHOP] | piece_bb[Piece.BLACK_QUEEN];
    }

    public final long orthogonalSliders(int side){
        return side == Side.WHITE ? piece_bb[Piece.WHITE_ROOK] | piece_bb[Piece.WHITE_QUEEN] :
                piece_bb[Piece.BLACK_ROOK] | piece_bb[Piece.BLACK_QUEEN];
    }

    public final long allPieces(int side){
        return side == Side.WHITE ? piece_bb[Piece.WHITE_PAWN] | piece_bb[Piece.WHITE_KNIGHT] |
                                 piece_bb[Piece.WHITE_BISHOP] | piece_bb[Piece.WHITE_ROOK] |
                                 piece_bb[Piece.WHITE_QUEEN] | piece_bb[Piece.WHITE_KING] :

                                 piece_bb[Piece.BLACK_PAWN] | piece_bb[Piece.BLACK_KNIGHT] |
                                 piece_bb[Piece.BLACK_BISHOP] | piece_bb[Piece.BLACK_ROOK] |
                                 piece_bb[Piece.BLACK_QUEEN] | piece_bb[Piece.BLACK_KING];
    }

    public final long allPieces() {
        return allPieces(Side.WHITE) | allPieces(Side.BLACK);
    }

    public final long attackersFrom(int square, long occ, int side){
        return side == Side.WHITE ? (pawnAttacks(square, Side.BLACK) & piece_bb[Piece.WHITE_PAWN]) |
                (getKnightAttacks(square) & piece_bb[Piece.WHITE_KNIGHT]) |
                (getBishopAttacks(square, occ) & (piece_bb[Piece.WHITE_BISHOP] | piece_bb[Piece.WHITE_QUEEN])) |
                (getRookAttacks(square, occ) & (piece_bb[Piece.WHITE_ROOK] | piece_bb[Piece.WHITE_QUEEN])) :

                (pawnAttacks(square, Side.WHITE) & piece_bb[Piece.BLACK_PAWN]) |
                (getKnightAttacks(square) & piece_bb[Piece.BLACK_KNIGHT]) |
                (getBishopAttacks(square, occ) & (piece_bb[Piece.BLACK_BISHOP] | piece_bb[Piece.BLACK_QUEEN])) |
                (getRookAttacks(square, occ) & (piece_bb[Piece.BLACK_ROOK] | piece_bb[Piece.BLACK_QUEEN]));
    }

    public final long attackersFromIncludingKings(int square, long occ, int side){
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

    public final BoardState doMove(Move move) {
        return performMove(move, this);
    }

    public final BoardState doMove(String uciMove) {
        return performMove(this.generateLegalMoves().stream().filter(m->m.toString().equals(uciMove)).findFirst().orElseThrow(), this);
    }

    public final BoardState doNullMove() {
        return performNullMove(this);
    }

    private BoardState performNullMove(BoardState oldBoardState) {
        BoardState state = oldBoardState.clone();

        state.halfMoveClock += 1;
        state.clearEnPassant();
        state.sideToPlay = Side.flip(state.sideToPlay);
        state.hash ^= Zobrist.SIDE;
        return state;
    }


    public static BoardState performMove(Move move, BoardState oldBoardState) {
        var state = oldBoardState.clone();

        state.fullMoveNormalized++;
        state.halfMoveClock++;
        state.movements |= (1L << move.to() | 1L << move.from());

        if (Piece.typeOf(state.items[move.from()]) == PieceType.PAWN) {
            state.halfMoveClock = 0;
        }

        state.clearEnPassant();

        switch (move.flags()) {
            case Move.QUIET -> state.movePieceQuiet(move.from(), move.to());
            case Move.DOUBLE_PUSH -> {
                state.movePieceQuiet(move.from(), move.to());
                state.enPassant = 1L << (move.from() + Square.direction(FORWARD, state.sideToPlay));
                state.hash ^= Zobrist.EN_PASSANT[Bitboard.lsb(state.enPassant) & 0b111];
            }
            case Move.OO -> {
                if (state.sideToPlay == Side.WHITE) {
                    state.movePieceQuiet(E1, G1);
                    state.movePieceQuiet(H1, F1);
                } else {
                    state.movePieceQuiet(E8, G8);
                    state.movePieceQuiet(H8, F8);
                }
            }
            case Move.OOO -> {
                if (state.sideToPlay == Side.WHITE) {
                    state.movePieceQuiet(E1, C1);
                    state.movePieceQuiet(A1, D1);
                } else {
                    state.movePieceQuiet(E8, C8);
                    state.movePieceQuiet(A8, D8);
                }
            }
            case Move.EN_PASSANT -> {
                state.movePieceQuiet(move.from(), move.to());
                state.removePiece(move.to() + Square.direction(BACK, state.sideToPlay));
            }
            case Move.PR_KNIGHT, Move.PR_BISHOP, Move.PR_ROOK, Move.PR_QUEEN -> {
                state.removePiece(move.from());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, move.getPieceType()), move.to());
            }
            case Move.PC_KNIGHT, Move.PC_BISHOP, Move.PC_ROOK, Move.PC_QUEEN -> {
                state.removePiece(move.from());
                state.removePiece(move.to());
                state.setPieceAt(Piece.makePiece(state.sideToPlay, move.getPieceType()), move.to());
            }
            case Move.CAPTURE -> {
                state.halfMoveClock = 0;
                state.movePiece(move.from(), move.to());
            }
        }

        state.sideToPlay = Side.flip(state.sideToPlay);
        state.hash ^= Zobrist.SIDE;

        return state;
    }

    public final int getSideToPlay(){
        return sideToPlay;
    }

    public final boolean isKingAttacked() {
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

        final long theirDiagonalSliders = diagonalSliders(them);
        final long theirOrthogonalSliders = orthogonalSliders(them);

        if ((getRookAttacks(ourKing, all) & theirOrthogonalSliders) != 0)
            return true;

        return (getBishopAttacks(ourKing, all) & theirDiagonalSliders) != 0;
    }


    /* not    side of the attacker */
    /**
     * @param side attacked side
     * @return attacked pieces
     */
    public final long attackedPieces(int side) {
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

    }

    /**
     * @param side attacked side
     * @return
     */
    public final long attackedPiecesUndefended(int side) {
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

    public final int smallestAttackerWithKing(int square, int side) {
        return smallestAttacker(square, side, true);
    }

    public final int smallestAttacker(int square, int side, boolean withAttackingKing){
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

//    public boolean isInsufficientMaterial(int color){
//        if ((bitboardOf(color, PieceType.PAWN) | bitboardOf(color, PieceType.ROOK) | bitboardOf(color, PieceType.QUEEN)) != 0)
//            return false;
//
//        long ourPieces = allPieces(color);
//        long theirPieces = allPieces(Side.flip(color));
//        if (bitboardOf(color, PieceType.KNIGHT) != 0)
//            return Long.bitCount(ourPieces) <= 2 && (theirPieces & ~bitboardOf(Side.flip(color), PieceType.KING) & ~bitboardOf(Side.flip(color), PieceType.QUEEN)) == 0;
//
//        long ourBishops = bitboardOf(color, PieceType.BISHOP);
//        if (ourBishops != 0){
//            boolean sameColor = (ourBishops & DARK_SQUARES) == 0 || (ourBishops & LIGHT_SQUARES) == 0;
//            return sameColor && (bitboardOf(color, PieceType.PAWN) | bitboardOf(Side.flip(color), PieceType.PAWN)) == 0
//                    || (bitboardOf(color, PieceType.KNIGHT) | bitboardOf(Side.flip(color), PieceType.KNIGHT)) == 0;
//        }
//        return true;
//    }
//
//    public final boolean isRepetitionOrFifty(/*BoardPosition position*/){
//        return false;
//        if (this.ply < 1) {
//            return false;
//        }
//        final long lastMoveBits = /*this.ply > 0 ?*/ this.history[this.ply - 1]; // : position.history[position.historyIndex - 1];
//        int count = 0;
//        int index = this.ply - 1;
//        while (index >= 0) {
//            if (this.history[index--] == lastMoveBits) {
//                count++;
//            }
//        }
//        return count > 2 || this.halfMoveClock >= 100;
//    }

    public final boolean hasNonPawnMaterial(int side) {
        int start = Piece.makePiece(side, PieceType.KNIGHT);
        int end = Piece.makePiece(side, PieceType.QUEEN);
        for (int piece = start; piece <= end; piece++){
            if (bitboardOf(piece) != 0)
                return true;
        }
        return false;
    }

//    private long calculateAttacks(long bitboard, Function<Integer, Long> attackFunction) {
//        return LongStream.iterate(bitboard, b -> b != 0, Bitboard::extractLsb)
//                .map(Long::numberOfTrailingZeros)
//                .mapToObj(x -> attackFunction.apply((int) x))
//                .reduce(0L, (a, b) -> a | b);
//    }

    private long calculateAttacks(long bitboard, Function<Integer, Long> attackFunction) {
        long underAttack = 0;
        while (bitboard != 0) {
            underAttack |= attackFunction.apply(Long.numberOfTrailingZeros(bitboard));
            bitboard = Bitboard.extractLsb(bitboard);
        }
        return underAttack;
    }

    public final MoveList generateLegalMoves(){
        return this.generateLegalMoves(false);
    }

    public final MoveList generateLegalQuiescence(){
        return generateLegalMoves(true);
    }

    public final MoveList generateLegalMoves(boolean onlyQuiescence) {
        final MoveList moves = new MoveList();
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

//        final long underAttack = pawnAttacks(bitboardOf(them, PieceType.PAWN), them) | getKingAttacks(theirKing)
//            | calculateAttacks(bitboardOf(them, PieceType.KNIGHT), Bitboard::getKnightAttacks)
//            | calculateAttacks(theirBishopsAndQueens, index -> getBishopAttacks(index, all ^ ourKingBb))
//            | calculateAttacks(theirRooksAndQueens, index -> getRookAttacks(index, all ^ ourKingBb));
//        final long kingAttacks = getKingAttacks(ourKing) & ~(usBb | underAttack);
//
//        moves.makeQ(ourKing, kingAttacks & ~themBb);
//        moves.makeC(ourKing, kingAttacks & themBb);

        // Squares that the king can't move to
        long underAttack = 0;
        underAttack |= pawnAttacks(bitboardOf(them, PieceType.PAWN), them) | getKingAttacks(theirKing);

        long b1 = bitboardOf(them, PieceType.KNIGHT);
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

        // knight and pawn checkers
        checkers = (getKnightAttacks(ourKing) & bitboardOf(them, PieceType.KNIGHT))
                | (pawnAttacks(ourKing, us) & bitboardOf(them, PieceType.PAWN));

        // sliding pieces threatening the king (our pieces are invisible and not obstructing)
        long candidates = (getRookAttacks(ourKing, themBb) & theirRooksAndQueens)
                | (getBishopAttacks(ourKing, themBb) & theirBishopsAndQueens);

        // detect pinned pieces
        long pinned = 0;
        while (candidates != 0) {
            int attackingSquare = Long.numberOfTrailingZeros(candidates);
            candidates = Bitboard.extractLsb(candidates);
            long piecesBetweenKingAndAttacker = between(ourKing, attackingSquare) & usBb;

            if (piecesBetweenKingAndAttacker == 0) {
                // no piece between our king and the attacking sliding piece; it's a checker
                checkers ^= 1L << attackingSquare;
            } else if (Bitboard.extractLsb(piecesBetweenKingAndAttacker) == 0) {
                // Only one piece between king and attacking sliding piece; that piece is pinned
                pinned ^= piecesBetweenKingAndAttacker;
            }
        }

        final long notPinned = ~pinned;
        switch (Long.bitCount(checkers)) {
            case 2:
                // there are at least two checkers, the only remaining options are moves with a king
                return moves;
            case 1: {
                int checkerSquare = Long.numberOfTrailingZeros(checkers);
                switch (Piece.typeOf(items[checkerSquare])){
                    case PieceType.PAWN:
                        // check to see if the checker is a pawn that can be captured ep
                        if (checkers == (us == Side.WHITE ? enPassant >>> 8 : enPassant << 8)) {
                            int enPassantSquare = Long.numberOfTrailingZeros(enPassant);
                            long nonPinnedPawnAttacks = pawnAttacks(enPassantSquare, them) & bitboardOf(us, PieceType.PAWN) & notPinned;
                            while (nonPinnedPawnAttacks != 0) {
                                // ep move which can save the king
                                moves.add(new Move(Long.numberOfTrailingZeros(nonPinnedPawnAttacks), enPassantSquare, Move.EN_PASSANT));
                                nonPinnedPawnAttacks = Bitboard.extractLsb(nonPinnedPawnAttacks);
                            }
                        }
                    // intentional fall through
                    case PieceType.KNIGHT:
                        long piecesAttackingChecker = attackersFrom(checkerSquare, all, us) & notPinned;
                        while (piecesAttackingChecker != 0){
                            int sq = Long.numberOfTrailingZeros(piecesAttackingChecker);
                            piecesAttackingChecker = Bitboard.extractLsb(piecesAttackingChecker);
                            if (pieceTypeAt(sq) == PieceType.PAWN && (1L << sq & PAWN_FINAL_RANKS) != 0L) {
                                // promoting a pawn with capturing move
                                moves.add(new Move(sq, checkerSquare, Move.PC_QUEEN));
                                moves.add(new Move(sq, checkerSquare, Move.PC_ROOK));
                                moves.add(new Move(sq, checkerSquare, Move.PC_KNIGHT));
                                moves.add(new Move(sq, checkerSquare, Move.PC_BISHOP));
                            } else {
                                // "normal" capture of checking knight
                                moves.add(new Move(sq, checkerSquare, Move.CAPTURE));
                            }
                        }
                        return moves;
                    default:
                        // so the checker must be bishop, rook or queen then
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
                    long pawnAttacks = pawnAttacks(enPassantSquare, them) & bitboardOf(us, PieceType.PAWN);
                    // pawnAttacks holds pawns that can perform an en passant capture
                    long nonPinnedPawnAttacks = pawnAttacks & notPinned;
                    while (nonPinnedPawnAttacks != 0) {
                        // 'pawnAttackSquare' holds the square from which a pawn attack to the en passant square can be done
                        int pawnAttackSquare = Long.numberOfTrailingZeros(nonPinnedPawnAttacks);
                        nonPinnedPawnAttacks = Bitboard.extractLsb(nonPinnedPawnAttacks);

                        long themWoEp = themBb ^ (us == Side.WHITE ? enPassant >>> 8 : enPassant << 8);
                        long usBbEpMove = usBb ^ 1L << pawnAttackSquare ^ 1L << enPassantSquare;
                        long kingAttackersAfterEp = (getRookAttacks(ourKing, themWoEp | usBbEpMove) & theirRooksAndQueens)
                                | (getBishopAttacks(ourKing, themWoEp | usBbEpMove) & theirBishopsAndQueens);

                        if (kingAttackersAfterEp == 0)
                            // allow EP take, only when not uncovering our king
                            moves.add(new Move(pawnAttackSquare, enPassantSquare, Move.EN_PASSANT));
                    }
                }

                // castling moves
                if (!onlyQuiescence) {
                    if (0 == ((this.movements & Bitboard.castlingPiecesKingsideMask(us)) | ((all | underAttack) & Bitboard.castlingBlockersKingsideMask(us))))
                        moves.add(us == Side.WHITE ? new Move(E1, G1, Move.OO) : new Move(E8, G8, Move.OO));

                    if (0 == ((this.movements & Bitboard.castlingPiecesQueensideMask(us)) |
                            ((all | (underAttack & ~ignoreOOODanger(us))) & Bitboard.castlingBlockersQueensideMask(us))))
                        moves.add(us == Side.WHITE ? new Move(E1, C1, Move.OOO) : new Move(E8, C8, Move.OOO));
                }

                // For each pinned rook, bishop, or queen...
                long pinnedRookBishopQueen = ~(notPinned | bitboardOf(us, PieceType.KNIGHT));
                while (pinnedRookBishopQueen != 0) {
                    int square = Long.numberOfTrailingZeros(pinnedRookBishopQueen);
                    pinnedRookBishopQueen = Bitboard.extractLsb(pinnedRookBishopQueen);

                    long attacksToKing = attacks(Piece.typeOf(items[square]), square, all) & line(ourKing, square);
                    if (!onlyQuiescence) {
                        moves.makeQ(square, attacksToKing & quietMask);
                    }
                    moves.makeC(square, attacksToKing & captureMask);
                }

                // for each pinned pawn
                long pinnedPawn = ~notPinned & bitboardOf(us, PieceType.PAWN);
                while (pinnedPawn != 0) {
                    int square = Long.numberOfTrailingZeros(pinnedPawn);
                    pinnedPawn = Bitboard.extractLsb(pinnedPawn);

                    if (((1L << square) & PAWN_FINAL_RANKS) != 0L) {
                        long pawnCaptures = pawnAttacks(square, us) & captureMask & line(ourKing, square);
                        moves.makePC(square, pawnCaptures);
                    } else {
                        long pawnCaptures = pawnAttacks(square, us) & themBb & line(square, ourKing);
                        moves.makeC(square, pawnCaptures);

                        if (!onlyQuiescence) {
                            // Single pawn pushes
                            long singlePush = Bitboard.push(1L << square, us) & ~all & line(ourKing, square);
                            long doublePush = Bitboard.push(singlePush & PAWN_DOUBLE_PUSH_LINES[us], us) & ~all & line(ourKing, square);

                            moves.makeQ(square, singlePush);
                            moves.makeDP(square, doublePush);
                        }
                    }
                }
                //Pinned knights cannot move anywhere, so we're done with pinned pieces.
                break;

        }


        // Create a lambda function for generateMovesForPiece
        BiConsumer<Long, Function<Integer, Long>> generateMovesForPieceLambda =
                (pieceBitboard, getAttacksFunction) -> {
                    while (pieceBitboard != 0){
                        int piecePosition = Long.numberOfTrailingZeros(pieceBitboard);
                        pieceBitboard = Bitboard.extractLsb(pieceBitboard);
                        long attacks = getAttacksFunction.apply(piecePosition);
                        moves.makeC(piecePosition, attacks & captureMask);
                        if (!onlyQuiescence) {
                            moves.makeQ(piecePosition, attacks & quietMask);
                        }
                    }
                };

        // Generate attacks for our knights, bishops, rooks and queens
        generateMovesForPieceLambda.accept(bitboardOf(us, PieceType.KNIGHT) & notPinned, Bitboard::getKnightAttacks);
        generateMovesForPieceLambda.accept(ourBishopsAndQueens & notPinned, sq -> getBishopAttacks(sq, all));
        generateMovesForPieceLambda.accept(ourRooksAndQueens & notPinned, sq -> getRookAttacks(sq, all));

        long pawnBitboard = bitboardOf(us, PieceType.PAWN) & notPinned & ~PAWN_RANKS[us];

        if (!onlyQuiescence) {
            // Single pawn pushes
            long singlePushTargets = (us == Side.WHITE) ? pawnBitboard << 8 : pawnBitboard >>> 8;
            singlePushTargets &= ~all;

            // Double pawn pushes
            long doublePushTargets = Bitboard.push(singlePushTargets & PAWN_DOUBLE_PUSH_LINES[us], us) & quietMask;
            singlePushTargets &= quietMask;

            while (singlePushTargets != 0) {
                int square = Long.numberOfTrailingZeros(singlePushTargets);
                singlePushTargets = Bitboard.extractLsb(singlePushTargets);
                moves.add(new Move(square - Square.direction(FORWARD, us), square, Move.QUIET));
            }

            while (doublePushTargets != 0) {
                int square = Long.numberOfTrailingZeros(doublePushTargets);
                doublePushTargets = Bitboard.extractLsb(doublePushTargets);
                moves.add(new Move(square - Square.direction(DOUBLE_FORWARD, us), square, Move.DOUBLE_PUSH));
            }
        }

        long leftPawnAttacks = ((us == Side.WHITE) ? whiteLeftPawnAttacks(pawnBitboard) : blackRightPawnAttacks(pawnBitboard)) & captureMask;
        long rightPawnAttacks = ((us == Side.WHITE) ? whiteRightPawnAttacks(pawnBitboard) : blackLeftPawnAttacks(pawnBitboard)) & captureMask;


        while (leftPawnAttacks != 0){
            int s = Long.numberOfTrailingZeros(leftPawnAttacks);
            leftPawnAttacks = Bitboard.extractLsb(leftPawnAttacks);
            moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.CAPTURE));
        }

        while (rightPawnAttacks != 0){
            int s = Long.numberOfTrailingZeros(rightPawnAttacks);
            rightPawnAttacks = Bitboard.extractLsb(rightPawnAttacks);
            moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.CAPTURE));
        }

        pawnBitboard = bitboardOf(us, PieceType.PAWN) & notPinned & PAWN_RANKS[us];
        if (pawnBitboard != 0) {
            if (!onlyQuiescence) {
                long singlePushTargets = (us == Side.WHITE) ? pawnBitboard << 8 : pawnBitboard >>> 8;
                singlePushTargets &= quietMask;
                while (singlePushTargets != 0) {
                    int square = Long.numberOfTrailingZeros(singlePushTargets);
                    singlePushTargets = Bitboard.extractLsb(singlePushTargets);
                    moves.add(new Move(square - Square.direction(FORWARD, us), square, Move.PR_QUEEN));
                    moves.add(new Move(square - Square.direction(FORWARD, us), square, Move.PR_ROOK));
                    moves.add(new Move(square - Square.direction(FORWARD, us), square, Move.PR_KNIGHT));
                    moves.add(new Move(square - Square.direction(FORWARD, us), square, Move.PR_BISHOP));
                }
            }

            leftPawnAttacks = ((us == Side.WHITE) ? whiteLeftPawnAttacks(pawnBitboard) : blackRightPawnAttacks(pawnBitboard)) & captureMask;
            rightPawnAttacks = ((us == Side.WHITE) ? whiteRightPawnAttacks(pawnBitboard) : blackLeftPawnAttacks(pawnBitboard)) & captureMask;

            while (leftPawnAttacks != 0){
                int s = Long.numberOfTrailingZeros(leftPawnAttacks);
                leftPawnAttacks = Bitboard.extractLsb(leftPawnAttacks);

                moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_QUEEN));
                moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_ROOK));
                moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_KNIGHT));
                moves.add(new Move(s - Square.direction(FORWARD_LEFT, us), s, Move.PC_BISHOP));
            }

            while (rightPawnAttacks != 0){
                int s = Long.numberOfTrailingZeros(rightPawnAttacks);
                rightPawnAttacks = Bitboard.extractLsb(rightPawnAttacks);

                moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_QUEEN));
                moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_ROOK));
                moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_KNIGHT));
                moves.add(new Move(s - Square.direction(FORWARD_RIGHT, us), s, Move.PC_BISHOP));
            }
        }

        return moves;
    }

    @Override
    public final String toString() {
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
        if (this.enPassant != 0L) {
            this.hash ^= Zobrist.EN_PASSANT[(Long.numberOfTrailingZeros(this.enPassant) & 0b111)];
            this.enPassant = 0L;
        }
    }

//    public final BoardState forSearchDepth(int searchDepth) { // TODO vyhodit
//        BoardState result = this.clone();
////        result.history = new int[searchDepth];
////        result.ply = 0;
//        return result;
//    }

    public final String toFen() {
        return Fen.toFen(this);
    }

    public final int mg() {
        return mg;
    }

    public final int eg() {
        return eg;
    }

    public final int interpolatedScore() {
        int phase = (this.phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE;
        return (this.mg() * (256 - phase) + this.eg() * phase) / 256;
    }

    public final boolean isInCheck() {
        this.generateLegalMoves();
        return this.checkers != 0L;
    }

    public final boolean isInCheckMate() {
        return this.generateLegalMoves().size() == 0;
    }

    public final boolean isCapture(String move) {
        Move parsedMove = Move.fromUciString(move, this);
        return this.pieceAt(parsedMove.to()) != Piece.NONE;
    }

    public record ScoreOutcome(int score, int piecesTaken) {}

    /**
     * @param square battle square
     * @param side perspective of score, starting move
     * @return score in basic material values, the higher, the better, no matter if white or black
     */
    public final ScoreOutcome seeScore(int square, int side) {
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
            if (possibleMoves.isEmpty()) {
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

    private final int getBasicMaterialValue(int square) {
        int piece = pieceAt(square);
        return BASIC_MATERIAL_VALUE[Piece.typeOf(piece)] * (Piece.sideOf(piece) == Side.WHITE ? 1 : -1);
    }

    /**
     * @param side attacked side
     * @return attacked pieces
     */
    public final long attackedPiecesUnderdefended(int side) {
        int sideThem = Side.flip(side);

        long attackedPieces = this.attackedPieces(side);
        long attackedUnderdefendedPieces = 0L;
        long work = attackedPieces;
        while (work != 0){
            int square = Long.numberOfTrailingZeros(work);
            int score = this.seeScore(square, sideThem).score();
            if (score > 0) {
                attackedUnderdefendedPieces |= 1L << square;
            }
            work = Bitboard.extractLsb(work);
        }

        return attackedUnderdefendedPieces;
    }

    /**
     * @param attackerSquare attacker square
     * @param attackedSide attacked side
     * @return pinned pieces
     */
    public final long pinnedPieces(int attackerSquare, int attackedSide) {
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

//    public record Params(byte[] pieces, int wKingPos, int bKingPos) {}
//
//    public Params toParams() {
//        byte[] result = new byte[80]; // 8 * 5 * 2
//        int index = 0;
//        for (int side = Side.WHITE; side <= Side.BLACK; side++) {
//            for (int piece = PieceType.PAWN; piece <= PieceType.QUEEN; piece++) {
//                long bitboard = this.bitboardOf(side, piece);
//                for (int i = 0; i < 8; i++) {
//                    result[index++] = (byte)((bitboard & 0xFF00000000000000L) >> 56);
//                    bitboard <<= 8;
//                }
//            }
//
//        }
//        return new Params(result,
//            Long.numberOfTrailingZeros(bitboardOf(Side.WHITE, PieceType.KING)),
//            Long.numberOfTrailingZeros(bitboardOf(Side.BLACK, PieceType.KING)));
//    }
}
