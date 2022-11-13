package org.javafish.board;

import org.javafish.bitboard.Bitboard;
import org.javafish.move.Move;
import org.javafish.move.MoveList;
import org.javafish.move.Zobrist;

import java.util.List;

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
import static org.javafish.eval.PieceSquareTable.EGS;
import static org.javafish.eval.PieceSquareTable.MGS;

public class BoardState implements Cloneable {
    public static int TOTAL_PHASE = 24;
    public static int[] PIECE_PHASES = {0, 1, 1, 2, 4, 0};

    public int ply;
    private long[] history;
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

    public BoardState(int[] items, int sideToPlay, long movements, long enPassantMask, int halfMoveClock, int fullMoveCount, int maxSearchDepth) {
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
            result.piece_bb = this.piece_bb.clone();
            result.items = this.items.clone();
            result.history = this.history.clone();
            return result;
        } catch (CloneNotSupportedException e) {
            throw new IllegalStateException(e);
        }
    }

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
    }

    public void removePiece(int square){
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
        return state;
    }


    public static BoardState performMove(Move move, BoardState oldBoardState) {
        BoardState state = oldBoardState.clone();

        state.fullMoveNormalized += 1;
        state.halfMoveClock += 1;
        state.history[state.ply++] = move.bits();
        state.movements |= (1L << move.to() | 1L << move.from());

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
    }

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
