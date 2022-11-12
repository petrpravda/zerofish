package org.miniyaga.move;


import org.miniyaga.board.Bitboard;
import org.miniyaga.board.Board;
import org.miniyaga.board.BoardPosition;
import org.miniyaga.board.Fen;
import org.miniyaga.model.enu.BlackBoardPos;
import org.miniyaga.model.enu.EnumCastling;
import org.miniyaga.model.enu.EnumColor;
import org.miniyaga.model.enu.EnumMoveType;
import org.miniyaga.model.enu.EnumPieceType;
import org.miniyaga.model.enu.WhiteBoardPos;
import org.miniyaga.score.Score;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.atomic.AtomicLong;
import java.util.stream.Collectors;

import static org.miniyaga.board.Bitboard.BLACK_KING_SIDE_CASTLING_BIT_PATTERN;
import static org.miniyaga.board.Bitboard.BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN;
import static org.miniyaga.board.Bitboard.LINE_MASKS;
import static org.miniyaga.board.Bitboard.PAWN_DOUBLE_MOVE_LINES;
import static org.miniyaga.board.Bitboard.WHITE_KING_SIDE_CASTLING_BIT_PATTERN;
import static org.miniyaga.board.Bitboard.WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN;
import static org.miniyaga.board.Bitboard.blackLeftPawnAttacks;
import static org.miniyaga.board.Bitboard.blackRightPawnAttacks;
import static org.miniyaga.board.Bitboard.getLineAttacks;
import static org.miniyaga.board.Bitboard.whiteLeftPawnAttacks;
import static org.miniyaga.board.Bitboard.whiteRightPawnAttacks;
import static org.miniyaga.board.Fen.fromFen;
import static org.miniyaga.score.PieceSquareTables.pst;

@FunctionalInterface
interface BitboardGetter {
    long run(EnumPieceType pieceType);
}

@FunctionalInterface
interface GenerateCheckingAndPinning {
    void run1(long occupiedC, long origins);
}

@FunctionalInterface
interface GeneratePawnCheckingAndPinning {
    void run1(long occupiedC, long origins, EnumColor player);
}

@FunctionalInterface
interface FindPins {
    void run2();
}

@FunctionalInterface
interface GenerateMovesForFigure {
    void run3();
}

@FunctionalInterface
interface CheckPinned {
    boolean run4(int start, int end);
}

@FunctionalInterface
interface AddMoveWhenLegal {
    void run5(EnumMoveType typ, EnumPieceType piece, int start, int end);
}

@FunctionalInterface
interface StreamMoves {
    void run6(EnumMoveType typ, EnumPieceType piece, int pos, long target_bb);
}

@FunctionalInterface
interface StreamPieceMoves {
    void run7(EnumPieceType piece, int pos, long target_bb, long opponent_bb, long empty_bb);
}

@FunctionalInterface
interface StreamPawnMoves {
    void run8(EnumMoveType nonpromotionalMoveType, long target_bb, int direction);
}

public class BoardState implements Cloneable {
    private static final String CHESSBOARD_LINE = "+---+---+---+---+---+---+---+---+\n";

    public long en_passant;
    public int castling;
    public int halfmove_count;
    public int[] items;
    public long[] bitboards; // 2 x 6 = 12
    public BoardState parent;
    public Move move;
    public Score score;


    public BoardState() {
        this.items = new int[64];
        this.bitboards = new long[12];
    }

    public static BoardState fromPosition(BoardPosition boardPosition) {
        if (boardPosition.items().length != 64) {
            throw new IllegalStateException(String.format("Expected a vector with 64 elements, but got %s", boardPosition.items().length));
        }

        BoardState state = new BoardState();

        // TODO (boardPosition.active_player() == EnumColor.WHITE ? 0 : 1) -> boardPosition.actiP.ordinal()
        state.halfmove_count = (boardPosition.fullmove_num() - 1) * 2 + (boardPosition.active_player() == EnumColor.WHITE ? 0 : 1);
        //this.state.halfmove_clock = boardPosition.halfmove_clock;
        //this.state.hash = 0; // TODO nezapomenout
        state.castling = boardPosition.castling_state();
        state.en_passant = 0;
        state.score = new Score(); //calculatePieces

        if (boardPosition.enpassant_target() != null) {
            // System.out.println(Bitboard.bitboardToString(1L << boardPosition.enpassant_target));
            state.set_enpassant(1L << boardPosition.enpassant_target());
            // throw new UnsupportedOperationException();
        }

        Arrays.fill(state.bitboards, 0);
        System.arraycopy(boardPosition.items(), 0, state.items, 0, 64);

        for (int i = 0; i < 64; i++) {
            int item = state.items[i];

            if (item != EnumPieceType.EMPTY.ordinal()) {
                state.addPiece(EnumColor.fromValue((int) Math.signum(item)), Math.abs(item), i);
            }
        }

        return state;
    }

    @Override
    protected BoardState clone() {
        try {
            BoardState result = (BoardState) super.clone();
            // result.score = (Score) this.score.clone();
            result.items = this.items.clone();
            result.bitboards = this.bitboards.clone();
            result.parent = this;
            result.score = (Score) this.score.clone();
            return result;
        } catch (CloneNotSupportedException e) {
            throw new IllegalStateException(e);
        }
    }

    public EnumColor getActivePlayer() {
        if ((this.halfmove_count & 1) == 0) {
            return EnumColor.WHITE;
        } else {
            return EnumColor.BLACK;
        }
    }

    private void increase_half_move_count() {
        this.halfmove_count += 1;
        // this.halfmove_clock += 1;

        // this.state.hash ^= zobrist.player();
    }

    private void reset_half_move_clock() {
        // this.halfmove_clock = 0;
    }

    private long getAllPieceBitboard(EnumColor color) {
//        long[] bbs = bitboards[color.ordinal()];
//        return Arrays.stream(bbs).reduce(0, (a, b) -> a | b);
        long result = 0;
        int delta = color.ordinal() * 6;
        for (int i = 0; i < 6; i++) {
            result |= bitboards[i + delta];
        }
        //for (int )
        return result;
    }

    private long getPieceBitboard(EnumPieceType piece, EnumColor color) {
        return bitboards[color.ordinal() * 6 + piece.ordinal() - 1];
    }

    private void addPiece(EnumColor color, int piece_id, int pos) {
        int piece = piece_id * color.getValue();
        this.items[pos] = piece;

        this.score.add(pst(piece, pos));

//        this.add_piece_score(piece, pos);
//        this.state.hash ^= zobrist.piece(piece, pos);
//
//        this.bitboards_all_pieces[(color.getValue() + 1)] |= 1L << pos;
        this.bitboards[color.ordinal() * 6 + piece_id - 1] |= 1L << pos;
    }


//    public int get_item(int pos) {
//        return this.items[pos];
//    }

    private int removePiece(int pos) {
        int piece = this.items[pos];
        this.score.sub(pst(piece, pos));
//        this.state.hash ^= zobrist.piece(piece, pos);


        if (piece == EnumPieceType.R.ordinal()) {
            // TODO .getValue - inline!!!
            if (pos == WhiteBoardPos.QUEEN_SIDE_ROOK.getValue()) {
                this.setRookMoved(EnumCastling.WHITE_QUEEN_SIDE);
            } else if (pos == WhiteBoardPos.KING_SIDE_ROOK.getValue()) {
                this.setRookMoved(EnumCastling.WHITE_KING_SIDE);
            }
        } else if (piece == -EnumPieceType.R.ordinal()) {
            if (pos == BlackBoardPos.QUEEN_SIDE_ROOK.getValue()) {
                this.setRookMoved(EnumCastling.BLACK_QUEEN_SIDE);
            } else if (pos == BlackBoardPos.KING_SIDE_ROOK.getValue()) {
                this.setRookMoved(EnumCastling.BLACK_KING_SIDE);
            }
        }

        EnumColor color = EnumColor.fromValue((int) Math.signum(piece));
        return this.remove(piece, color, pos);
    }

    private void setRookMoved(EnumCastling castling) {
        if (this.canCastle(castling)) {
            int previous_state = this.castling;
            this.castling ^= castling.getValue();
//            this.update_hash_for_castling(previous_state);
        }
    }

    private boolean canCastle(EnumCastling castling) {
        return (this.castling & castling.getValue()) != 0;
    }

    private int remove(int piece, EnumColor color, int pos) {
        int piece_id = Math.abs(piece);
        //this.bitboards[(piece_id - 1) + (color == EnumColor.BLACK ? 6 : 0)] &= ~(1L << pos);
        this.bitboards[color.ordinal() * 6 + piece_id - 1] &= ~(1L << pos);
//        this.bitboards_all_pieces[color.getValue() + 1] &= ~(1L << pos);
        this.items[pos] = 0; //EnumPieceType.EMPTY.ordinal();

        return piece;
    }

    private void clearEnPassant() {
        // TODO bud null anebo vubec nepouzivat
        long previous_state = this.en_passant;

        if (previous_state != 0) {
            this.en_passant = 0;
            // this.update_hash_for_enpassant(previous_state);
        }
    }


    private void set_enpassant(long pos) {
        long previous_state = this.en_passant;
        this.en_passant = pos;

        // this.update_hash_for_enpassant(previous_state);
    }

    private void set_white_has_castled() {
        int previous_state = this.castling;
        this.castling = EnumCastling.clear_castling_bits(EnumColor.WHITE, this.castling);
        this.castling |= EnumCastling.WHITE_HAS_CASTLED.getValue();
//        this.update_hash_for_castling(previous_state);
    }

    private void set_black_has_castled() {
        int previous_state = this.castling;
        this.castling = EnumCastling.clear_castling_bits(EnumColor.BLACK, this.castling);
        this.castling |= EnumCastling.BLACK_HAS_CASTLED.getValue();
//        this.update_hash_for_castling(previous_state);
    }

    private void set_king_moved(EnumColor color) {
        // int previous_state = this.state.castling;
        this.castling = EnumCastling.clear_castling_bits(color, this.castling);
        // this.update_hash_for_castling(previous_state);
    }

    private boolean is_in_check(EnumColor color) {
        // TODO zjednodusit
        // TODO nemelo by byt static?

        if (color == EnumColor.WHITE) {
            return is_attacked(EnumColor.BLACK, Long.numberOfTrailingZeros(this.bitboards[color.ordinal() * 6 +EnumPieceType.K.ordinal() - 1]));
        } else {
            return is_attacked(EnumColor.WHITE, Long.numberOfTrailingZeros(this.bitboards[color.ordinal() * 6 + EnumPieceType.K.ordinal() - 1]));
        }
    }

    private boolean is_attacked(EnumColor opponent_color, int pos) {
        long occupied_bb = this.getAllPieceBitboard(EnumColor.WHITE) | this.getAllPieceBitboard(EnumColor.BLACK);
        long target_bb = 1L << pos;

        // long[] activeBitboards = state.getBitboards(opponent_color);
        BitboardGetter getBitboard = (EnumPieceType pieceType) -> this.bitboards[opponent_color.ordinal() * 6 + pieceType.ordinal() - 1];

        // Check knights
        if ((getBitboard.run(EnumPieceType.N) & Bitboard.getKnightAttacks(pos)) != 0) {
            return true;
        }

        // Check diagonal
        long queens = getBitboard.run(EnumPieceType.Q);
        if (((getBitboard.run(EnumPieceType.B) | queens) & Bitboard.getBishopAttacks(pos, occupied_bb)) != 0) {
            return true;
        }

        // Check orthogonal
        if (((getBitboard.run(EnumPieceType.R) | queens) & Bitboard.getRookAttacks(pos, occupied_bb)) != 0) {
            return true;
        }

        // Check pawns
        long pawns = getBitboard.run(EnumPieceType.P);
        if (opponent_color == EnumColor.WHITE) {
            if (((whiteLeftPawnAttacks(pawns) | whiteRightPawnAttacks(pawns)) & target_bb) != 0) {
                return true;
            }
        } else {
            if (((blackLeftPawnAttacks(pawns) | blackRightPawnAttacks(pawns)) & target_bb) != 0) {
                return true;
            }
        }

        // Check king
        return (Bitboard.getKingAttacks(Long.numberOfTrailingZeros(getBitboard.run(EnumPieceType.K))) & target_bb) != 0;
    }

    public long get_enpassant_state() {
        return this.en_passant;
    }

    private boolean is_kingside_castling_valid_for_white(long empty_bb) {
        return (empty_bb & WHITE_KING_SIDE_CASTLING_BIT_PATTERN) == WHITE_KING_SIDE_CASTLING_BIT_PATTERN
                && !is_attacked(EnumColor.BLACK, WhiteBoardPos.KING_START.getValue())
                && !is_attacked(EnumColor.BLACK, WhiteBoardPos.KING_START.getValue() + 1)
                && !is_attacked(EnumColor.BLACK, WhiteBoardPos.KING_START.getValue() + 2);
    }

    private boolean is_queenside_castling_valid_for_white(long empty_bb) {
        return (empty_bb & WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN) == WHITE_QUEEN_SIDE_CASTLING_BIT_PATTERN
                && !is_attacked(EnumColor.BLACK, WhiteBoardPos.KING_START.getValue())
                && !is_attacked(EnumColor.BLACK, WhiteBoardPos.KING_START.getValue() - 1)
                && !is_attacked(EnumColor.BLACK, WhiteBoardPos.KING_START.getValue() - 2);
    }

    private boolean is_kingside_castling_valid_for_black(long empty_bb) {
        return (empty_bb & BLACK_KING_SIDE_CASTLING_BIT_PATTERN) == BLACK_KING_SIDE_CASTLING_BIT_PATTERN
                && !is_attacked(EnumColor.WHITE, BlackBoardPos.KING_START.getValue())
                && !is_attacked(EnumColor.WHITE, BlackBoardPos.KING_START.getValue() + 1)
                && !is_attacked(EnumColor.WHITE, BlackBoardPos.KING_START.getValue() + 2);
    }

    private boolean is_queenside_castling_valid_for_black(long empty_bb) {
        return (empty_bb & BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN) == BLACK_QUEEN_SIDE_CASTLING_BIT_PATTERN
                && !is_attacked(EnumColor.WHITE, BlackBoardPos.KING_START.getValue())
                && !is_attacked(EnumColor.WHITE, BlackBoardPos.KING_START.getValue() - 1)
                && !is_attacked(EnumColor.WHITE, BlackBoardPos.KING_START.getValue() - 2);
    }

    @Override
    public String toString() {
        StringBuilder result = new StringBuilder(CHESSBOARD_LINE);
        for (int i = 56; i >= 0; i -= 8){
            for (int j = 0; j < 8; j++){
                int piece = this.items[i + j];
                boolean black = piece < 0;
                EnumPieceType pieceType = EnumPieceType.values()[Math.abs(piece)];

                result
                        .append("| ")
                        .append(black ? pieceType.getBoardChar().toLowerCase() : pieceType.getBoardChar())
                        .append(' ');
            }
            result.append("|\n")
                    .append(CHESSBOARD_LINE);
        }
        result.append("Fen: ").append(Fen.toFen(this));
        return result.toString();
    }

    public int getFullMoveNumber() {
        // state.halfmove_count = (boardPosition.fullmove_num() - 1) * 2 + (boardPosition.active_player() == EnumColor.WHITE ? 0 : 1);
        return this.halfmove_count / 2 + 1;
    }

    public List<Move> getMovesPath() {
        List<Move> path = new ArrayList<>();
        BoardState pointer = this;
        while (pointer != null) {
            if (pointer.move != null) {
                path.add(pointer.move);
            }
            pointer = pointer.parent;
        }
        Collections.reverse(path);
        return path;
    }

    public String getMovesPathPosition() {
        return String.format("position startpos moves %s", getMovesPath().stream().map(Move::toString).collect(Collectors.joining(" ")));
    }

    public boolean isMoveChecking(Move move) {
//        if (Math.abs(this.items[move.end()]) == EnumPieceType.K.ordinal()) {
//            return true;
//        } else {
//            return false;
//        }

//        EnumColor opposite = this.getActivePlayer().getOpposite();
//        int oppositeKingPosition = Long.numberOfTrailingZeros(this.bitboards[opposite.ordinal() * 6 + EnumPieceType.K.ordinal() - 1]);
//        return this.is_attacked(this.getActivePlayer(), oppositeKingPosition);
        Board.MoveResult moveResult = performMove(move, this);
        boolean result = moveResult.boardState.is_in_check(moveResult.boardState.getActivePlayer());
        return result;
    }


    public BoardState doMove(Move move) {
        return BoardState.performMove(move, this).boardState;
    }

    /**
     *
     * @param m move
     * @return Pair of own and captured piece
     */
    public static Board.MoveResult performMove(Move m, BoardState oldState) {
        // Leaving this method intentionally static, so that nobody confuses this and state which
        // could lead to problems that are hard to detect.
        BoardState state = oldState.clone();
        state.move = m;
        state.increase_half_move_count();

        int move_start = m.start();
        int move_end = m.end();
        EnumPieceType target_piece_id = m.piece_id();

        int own_piece = state.removePiece(move_start);
        EnumColor color = EnumColor.fromValue((int) Math.signum(own_piece));

        state.clearEnPassant();

        if (state.items[move_end] != EnumPieceType.EMPTY.ordinal()) {
            // Capture move (except en passant)
            int removed_piece = state.removePiece(move_end);
            state.addPiece(color, target_piece_id.ordinal(), move_end);

            state.reset_half_move_clock();

            return Board.MoveResult.ofCaptured(own_piece, Math.abs(removed_piece), state);
        }

        state.addPiece(color, target_piece_id.ordinal(), move_end);

        switch (m.typ()) {
            case PAWN_QUIET:
                state.reset_half_move_clock();

                if (Math.abs(move_start - move_end) == 16) {  // VEL001
                    // this.set_enpassant(1L << move_start);
                    state.set_enpassant(1L << ((move_start + move_end) / 2));
                }
                break;

            case PROMOTION:
                state.reset_half_move_clock();
                break;

            case EN_PASSANT:
                state.reset_half_move_clock();
                if (Math.abs(move_start - move_end) == 7) {
                    state.removePiece(move_start - color.getValue());
                    // this.pos_history.push(this.state.hash);
                } else {
                    state.removePiece(move_start + color.getValue());
                    // this.pos_history.push(this.state.hash);
                }
                return Board.MoveResult.ofCaptured(own_piece, EnumPieceType.P.ordinal(), state);

            case KING_QUIET:
            case KING_CAPTURE:
                // this.set_king_pos(color, move_end);
                state.set_king_moved(color);
                break;

            case CASTLING:
                if (own_piece == EnumPieceType.K.ordinal()) {
//                    this.set_king_pos(EnumColor.WHITE, move_end);

                    // Special castling handling
                    if (move_start - move_end == -2) {
                        state.removePiece(WhiteBoardPos.KING_SIDE_ROOK.getValue());
                        state.addPiece(EnumColor.WHITE, EnumPieceType.R.ordinal(), WhiteBoardPos.KING_START.getValue() + 1);
                        state.set_white_has_castled();
                    } else if (move_start - move_end == 2) {
                        state.removePiece(WhiteBoardPos.QUEEN_SIDE_ROOK.getValue());
                        state.addPiece(EnumColor.WHITE, EnumPieceType.R.ordinal(), WhiteBoardPos.KING_START.getValue() - 1);
                        state.set_white_has_castled();
                    }
                } else if (own_piece == -EnumPieceType.K.ordinal()) {
//                    this.set_king_pos(EnumColor.BLACK, move_end);

                    // Special castling handling
                    if (move_start - move_end == -2) {
                        state.removePiece(BlackBoardPos.KING_SIDE_ROOK.getValue());
                        state.addPiece(EnumColor.BLACK, EnumPieceType.R.ordinal(), BlackBoardPos.KING_START.getValue() + 1);
                        state.set_black_has_castled();
                    } else if (move_start - move_end == 2) {
                        state.removePiece(BlackBoardPos.QUEEN_SIDE_ROOK.getValue());
                        state.addPiece(EnumColor.BLACK, EnumPieceType.R.ordinal(), BlackBoardPos.KING_START.getValue() - 1);
                        state.set_black_has_castled();
                    }
                }
                break;
        }

//        this.pos_history.push(this.state.hash);
        return Board.MoveResult.ofQuiet(own_piece, state);
    }

    public List<Move> generateLegalMoves() {
        EnumColor activePlayer = this.getActivePlayer();
        EnumColor oppositePlayer = activePlayer.getOpposite();
        long opponent_bb = this.getAllPieceBitboard(oppositePlayer);
        long own_bb = this.getAllPieceBitboard(activePlayer);
        long occupied = opponent_bb | own_bb;
        long empty_bb = ~occupied;

        // long[] activeBitboards = state.getBitboards(active_player);

        BitboardGetter getOppoBitboard = (EnumPieceType pieceType) -> this.bitboards[oppositePlayer.ordinal() * 6 + pieceType.ordinal() - 1];
        BitboardGetter getBitboard = (EnumPieceType pieceType) -> this.bitboards[activePlayer.ordinal() * 6 + pieceType.ordinal() - 1];

        // Stream.Builder<Move> result = Stream.builder();
        long kingBitboard = getBitboard.run(EnumPieceType.K);
        int kingPos = Long.numberOfTrailingZeros(kingBitboard);
        long oppoKingBitboard = getOppoBitboard.run(EnumPieceType.K);
        int oppoKingPos = Long.numberOfTrailingZeros(oppoKingBitboard);
        long occupiedWoKing = occupied ^ kingBitboard;
        List<Move> result = new ArrayList<>(40); // TODO finetune

        if (kingPos >= 64) {
            throw new IllegalStateException(String.format("Where is my king?%n%s", this.getMovesPathPosition()));
        }

//        if (oppoKingPos >= 64) {
//            throw new IllegalStateException("Where is king of my opponent?");
//        }

        AtomicLong checkingMask = new AtomicLong();

        GenerateCheckingAndPinning generateKingChecking = (long occupiedC, long origins) -> {
            checkingMask.updateAndGet(v -> v | Bitboard.getKingAttacks(oppoKingPos));
        };

        GenerateCheckingAndPinning generateKnightChecking = (long occupiedC, long origins) -> {
            // long knights = getOppoBitboard.run(EnumPieceType.N);
            for (int pos1 : Bitboard.iter(origins)) {
                checkingMask.updateAndGet(v -> v | Bitboard.getKnightAttacks(pos1));
            }
        };

        GenerateCheckingAndPinning generateRookChecking = (long occupiedC, long origins) -> {
            //long rooks = getOppoBitboard.run(EnumPieceType.R);
            for (int pos : Bitboard.iter(origins)) {
                checkingMask.updateAndGet(v -> v | Bitboard.getRookAttacks(pos, occupiedC));
            }
        };

        GenerateCheckingAndPinning generateBishopChecking = (long occupiedC, long origins) -> {
            //long bishops = getOppoBitboard.run(EnumPieceType.B);
            for (int pos : Bitboard.iter(origins)) {
                checkingMask.updateAndGet(v -> v | Bitboard.getBishopAttacks(pos, occupiedC));
            }
        };

//        GenerateCheckingAndPinning generateQueenChecking = (long occupiedC) -> {
//            long bishops = getOppoBitboard.run(EnumPieceType.Q);
//            for (int pos : Bitboard.iter(bishops)) {
//                checkingMask.updateAndGet(v -> v | Bitboard.getQueenAttacks(pos, occupiedC));
//            }
//        };

        GeneratePawnCheckingAndPinning generatePawnChecking = (long occupiedC, long origins, EnumColor player) -> {
            // long pawns = getOppoBitboard.run(EnumPieceType.P);
            long left_attacks = origins & 0xfefefefefefefefeL; // mask right column
            long right_attacks = origins & 0x7f7f7f7f7f7f7f7fL; // mask left column
            //if (oppositePlayer == EnumColor.WHITE) {
            if (player == EnumColor.WHITE) {
                left_attacks <<= 7;
                right_attacks <<= 9;
            } else {
                left_attacks >>>= 9;
                right_attacks >>>= 7;
            }
            long allAttacks = left_attacks | right_attacks;
            checkingMask.updateAndGet(v -> v | allAttacks);
        };

        List<Long> pinMasks = new  ArrayList<>();

        FindPins findPins = () -> {
            // oppoKingPos;
            long bishops = getOppoBitboard.run(EnumPieceType.B);
            long rooks = getOppoBitboard.run(EnumPieceType.R);
            long queens = getOppoBitboard.run(EnumPieceType.Q);

            // long diagonalAttacks = Bitboard.getBishopAttacks(kingPos, bishops | queens);

//            for (Bitboard.MoveDirection dir : BISHOP_MOVE_DIRECTIONS) {
//                long pinRay = generateRay(kingPos, dir.x(), dir.y());
//                if ((pinRay & (bishops | queens)) != 0L) {
//                    long pinnedPieces = pinRay & own_bb;
//                }
//
//                System.out.println(Bitboard.bitboardToString(pinRay));
//            }


            Bitboard.LineAttackMask masks1 = LINE_MASKS[Bitboard.Directions.Diagonal.maskIndex(kingPos)];
            Bitboard.LineAttackMask masks2 = LINE_MASKS[Bitboard.Directions.AntiDiagonal.maskIndex(kingPos)];

            long[] pinRays = new long[] {getLineAttacks(opponent_bb, masks1.lower(), 0L),
                getLineAttacks(opponent_bb, 0L, masks1.upper()),
                getLineAttacks(opponent_bb, masks2.lower(), 0L),
                getLineAttacks(opponent_bb, 0L, masks2.upper())};

            for (long pinRay : pinRays) {
                long occup = bishops | queens;
                if ((pinRay & occup) != 0L) {
                    // ray does come from real bishop or queen
                    long pinCandidates = pinRay & own_bb;
                    if (pinCandidates != 0L) {
                        // there is at least one pin, but there might be more which we have to check
                        if (Long.bitCount(pinCandidates) == 1) {
                            // there is only one piece in a way which is really a pin
                            pinMasks.add(pinRay);
                            // System.out.println(Bitboard.bitboardToString(pinRay));
                        }
                    }
                }
            }




            masks1 = LINE_MASKS[Bitboard.Directions.Horizontal.maskIndex(kingPos)];
            masks2 = LINE_MASKS[Bitboard.Directions.Vertical.maskIndex(kingPos)];

            pinRays = new long[] {getLineAttacks(opponent_bb, masks1.lower(), 0L),
                    getLineAttacks(opponent_bb, 0L, masks1.upper()),
                    getLineAttacks(opponent_bb, masks2.lower(), 0L),
                    getLineAttacks(opponent_bb, 0L, masks2.upper())};

            for (long pinRay : pinRays) {
                long occup = rooks | queens;
                if ((pinRay & occup) != 0L) {
                    // ray does come from real rook or queen
                    long pinCandidates = pinRay & own_bb;
                    if (pinCandidates != 0L) {
                        // there is at least one pin, but there might be more which we have to check
                        if (Long.bitCount(pinCandidates) == 1) {
                            // there is only one piece in a way which is really a pin
                            pinMasks.add(pinRay);
                            // System.out.println(Bitboard.bitboardToString(pinRay));
                        }
                    }
                }
            }


//
//            getLineAttacks(occ, LINE_MASKS[Bitboard.Directions.Diagonal.maskIndex(sq)])
//                    | getLineAttacks(occ, LINE_MASKS[Bitboard.Directions.AntiDiagonal.maskIndex(sq)]);
//



        };

        // https://www.codeproject.com/Articles/5313417/Worlds-Fastest-Bitboard-Chess-Movegenerator
        long oppoBitboardPawns = getOppoBitboard.run(EnumPieceType.P);
        generatePawnChecking.run1(occupiedWoKing, oppoBitboardPawns, oppositePlayer);
        long oppoBitboardKnights = getOppoBitboard.run(EnumPieceType.N);
        generateKnightChecking.run1(occupiedWoKing, oppoBitboardKnights);
        generateKingChecking.run1(occupiedWoKing, -1L);
        long oppoBitboardQueens = getOppoBitboard.run(EnumPieceType.Q);
        long oppoBitboardRooks = getOppoBitboard.run(EnumPieceType.R) | oppoBitboardQueens;
        generateRookChecking.run1(occupiedWoKing, oppoBitboardRooks);
        long oppoBitboardBishops = getOppoBitboard.run(EnumPieceType.B) | oppoBitboardQueens;
        generateBishopChecking.run1(occupiedWoKing, oppoBitboardBishops);

        findPins.run2();
        long uberPinMask = pinMasks.stream().reduce(0L, (a, b) -> a | b);

        boolean playerIsInCheck = (checkingMask.get() & kingBitboard) != 0L;
        long nonChecking = ~checkingMask.get();

        CheckPinned checkPinned = (int start, int end) -> {
            for (Long pinMask : pinMasks) {
                if ((pinMask & 1L << start) != 0L && (pinMask & 1L << end) == 0L) {
                    // cannot move pinned piece
                    return true;
                }
            }
            return false;
        };

        AddMoveWhenLegal addMoveWhenLegal = (EnumMoveType typ, EnumPieceType piece, int start, int end) -> {
            long endBb = 1L << end;
            long startBb = 1L << start;
            long occupiedAfterMove = (occupied & ~startBb) | endBb;
            boolean isEnPassant = typ == EnumMoveType.EN_PASSANT;
            if (isEnPassant) {
                long takenClearingMask = Long.numberOfTrailingZeros(en_passant) < 32 ? ~(en_passant << 8) : ~(en_passant >> 8);
                occupiedAfterMove &= takenClearingMask;
            }
            long endComplement = ~endBb;

            if (playerIsInCheck || typ == EnumMoveType.EN_PASSANT) {
                checkingMask.updateAndGet(v -> 0L);

                generatePawnChecking.run1(occupiedAfterMove, oppoBitboardPawns & (isEnPassant ?
                        // not using takenClearingMask because of optimalization reasons - this case is realy, realy rare
                        // d4 Nf6 c4 g6 Nc3 d5 cxd5 Nxd5 e4 Nxc3 bxc3 Bg7 Bc4 O-O Ne2 c5 Be3 Nc6 O-O Bd7 Rb1 a6 dxc5 Qc7 Qc2
                        // Na5 Bd3 Rfd8 Rfd1 Be6 Nf4 Bc4 Bxc4 Nxc4 Nd5 Rxd5 exd5 Nxe3 fxe3 Qxc5 Rxb7 Qxe3+ Qf2 Qxc3 d6 exd6 Qxf7+
                        // Kh8 h3 Rf8 Qe7 h5 Kh1 Rf2 Qxd6 Kh7 Qc7 Qxc7 Rxc7 Kh6 a3 Ra2 Rc6 Rxa3 Re1 Bd4 Kh2 h4 Ree6 Kh5 Rxa6 Rb3
                        // Ra5+ Kh6 Raa6 Kh5 Rxg6 Rb1 g4+ hxg3+
                        Long.numberOfTrailingZeros(en_passant) < 32 ? ~(en_passant << 8) : ~(en_passant >> 8) : endComplement), oppositePlayer);
                generateKnightChecking.run1(occupiedAfterMove, oppoBitboardKnights & endComplement);
                generateKingChecking.run1(occupiedAfterMove, -1L);
                generateRookChecking.run1(occupiedAfterMove, oppoBitboardRooks & endComplement);
                generateBishopChecking.run1(occupiedAfterMove, oppoBitboardBishops & endComplement);

                // has the king changed its position?
                long newKingBitboard = (typ == EnumMoveType.KING_QUIET || typ == EnumMoveType.KING_CAPTURE) ?
                        1L << end : kingBitboard;
                boolean playerIsStillInCheck = (checkingMask.get() & newKingBitboard) != 0L;
                if (playerIsStillInCheck) {
                    return;
                }
            }

//            boolean checking = false;
//
//            if (piece == EnumPieceType.Q) {
//                long attacks = Bitboard.getQueenAttacks(end, occupiedAfterMove);
//                checking = (oppoKingBitboard & attacks) != 0;
//            }

            // (occupied & ~startBb) | endBb;

            // TODO promotiny pridat
            long pawnBitboard = piece == EnumPieceType.P ? (getBitboard.run(EnumPieceType.P) & ~startBb) | endBb : getBitboard.run(EnumPieceType.P);
            long knightBitboard = piece == EnumPieceType.N ? (getBitboard.run(EnumPieceType.N) & ~startBb) | endBb : getBitboard.run(EnumPieceType.N);
            long rookBitboard = piece == EnumPieceType.R ? (getBitboard.run(EnumPieceType.R) & ~startBb) | endBb : getBitboard.run(EnumPieceType.R);
            long bishopBitboard = piece == EnumPieceType.B ? (getBitboard.run(EnumPieceType.B) & ~startBb) | endBb : getBitboard.run(EnumPieceType.B);
            long queenBitboard = piece == EnumPieceType.Q ? (getBitboard.run(EnumPieceType.Q) & ~startBb) | endBb : getBitboard.run(EnumPieceType.Q);
            rookBitboard |= queenBitboard;
            bishopBitboard |= queenBitboard;

            checkingMask.updateAndGet(v -> 0L);
            generateKnightChecking.run1(occupiedAfterMove, knightBitboard);
            generateRookChecking.run1(occupiedAfterMove, rookBitboard);
            generateBishopChecking.run1(occupiedAfterMove, bishopBitboard);
            generatePawnChecking.run1(occupiedAfterMove, pawnBitboard, activePlayer);

//            if (piece == EnumPieceType.R || piece == EnumPieceType.Q) {
//
//            }

            boolean checking = (checkingMask.get() & oppoKingBitboard) != 0L;
            Move move1 = new Move(typ, piece.ordinal(), start, end, checking);
            result.add(move1.with_score(this.doMove(move1).score.mg));
        };

        StreamMoves streamMoves = (EnumMoveType typ, EnumPieceType piece, int pos, long target_bb) -> {
            if (target_bb != 0) {
                int toSq;
                while (target_bb != 0){
                    toSq = Bitboard.lsb(target_bb);
                    target_bb = Bitboard.extractLsb(target_bb);
                    if ((uberPinMask & 1L << pos) == 0L || !checkPinned.run4(pos, toSq)) {
                        addMoveWhenLegal.run5(typ, piece, pos, toSq);
                    }
                }
            }
        };

        StreamPieceMoves streamPieceMoves = (EnumPieceType piece, int pos, long target_bb, long opponent_bb1, long empty_bb1) -> {
            if (target_bb != 0) {
                int toSq;
                while (target_bb != 0){
                    toSq = Bitboard.lsb(target_bb);
                    target_bb = Bitboard.extractLsb(target_bb);
                    if ((uberPinMask & 1L << pos) == 0L || !checkPinned.run4(pos, toSq)) {
                        long bb_toSq = 1L << toSq;
                        if ((bb_toSq & opponent_bb1) != 0L) {
                            addMoveWhenLegal.run5(EnumMoveType.CAPTURE, piece, pos, toSq);
                        } else if ((bb_toSq & empty_bb1) != 0) {
                            addMoveWhenLegal.run5(EnumMoveType.QUIET, piece, pos, toSq);
                        }
                    }
                }
            }
        };

        StreamPawnMoves streamPawnMoves = (EnumMoveType nonpromotionalMoveType, long target_bb, int direction) -> {
            // return streamPawnMoves(EnumMoveType.PAWN_QUIET, target_bb, direction);
            if (target_bb != 0) {
                for (int end : Bitboard.iter(target_bb)) {
                    //for end in BitBoard(target_bb) {
                    int start = end - direction;

                    if ((uberPinMask & 1L << start) == 0L || !checkPinned.run4(start, end)) {
                        if (end <= 7 || end >= 56) {
                            // Promotion
                            addMoveWhenLegal.run5(EnumMoveType.PROMOTION, EnumPieceType.Q, start, end);
                            addMoveWhenLegal.run5(EnumMoveType.PROMOTION, EnumPieceType.N, start, end);
                            addMoveWhenLegal.run5(EnumMoveType.PROMOTION, EnumPieceType.R, start, end);
                            addMoveWhenLegal.run5(EnumMoveType.PROMOTION, EnumPieceType.B, start, end);
                        } else {
                            // Normal move
                            addMoveWhenLegal.run5(nonpromotionalMoveType, EnumPieceType.P, start, end);
                        }
                    }
                }
            }
        };


        GenerateMovesForFigure generateKingMoves = () -> {
            long kingTargets = Bitboard.getKingAttacks(kingPos) & nonChecking;

            streamMoves.run6(EnumMoveType.KING_QUIET, EnumPieceType.K, kingPos, kingTargets & empty_bb);
            streamMoves.run6(EnumMoveType.KING_CAPTURE, EnumPieceType.K, kingPos, kingTargets & opponent_bb);

            if (activePlayer == EnumColor.WHITE) {
                if (kingPos == WhiteBoardPos.KING_START.getValue()) {
                    if (canCastle(EnumCastling.WHITE_KING_SIDE) && is_kingside_castling_valid_for_white(empty_bb)) {
                        streamMoves.run6(EnumMoveType.CASTLING, EnumPieceType.K, kingPos, 1L << (kingPos + 2));
                    }

                    if (canCastle(EnumCastling.WHITE_QUEEN_SIDE) && is_queenside_castling_valid_for_white(empty_bb)) {
                        streamMoves.run6(EnumMoveType.CASTLING, EnumPieceType.K, kingPos, 1L << (kingPos - 2));
                    }
                }
            } else {
                if (kingPos == BlackBoardPos.KING_START.getValue()) {
                    if (canCastle(EnumCastling.BLACK_KING_SIDE) && is_kingside_castling_valid_for_black(empty_bb)) {
                        streamMoves.run6(EnumMoveType.CASTLING, EnumPieceType.K, kingPos, 1L << (kingPos + 2));
                    }

                    if (canCastle(EnumCastling.BLACK_QUEEN_SIDE) && is_queenside_castling_valid_for_black(empty_bb)) {
                        streamMoves.run6(EnumMoveType.CASTLING, EnumPieceType.K, kingPos, 1L << (kingPos - 2));
                    }
                }
            }
        };

        GenerateMovesForFigure generateKnightMoves = () -> {
            long knights = getBitboard.run(EnumPieceType.N);
            for (int pos1 : Bitboard.iter(knights)) {
                long attacks = Bitboard.getKnightAttacks(pos1);
                streamPieceMoves.run7(EnumPieceType.N, pos1, attacks, opponent_bb, empty_bb);
            }
        };

        GenerateMovesForFigure generatePawnMoves = () -> {
            long pawns = getBitboard.run(EnumPieceType.P);
            if (activePlayer == EnumColor.WHITE) {
                long target_bb = (pawns << 8) & empty_bb;
                streamPawnMoves.run8(EnumMoveType.PAWN_QUIET, target_bb, 8);

                // Double move
                target_bb &= PAWN_DOUBLE_MOVE_LINES[EnumColor.WHITE.ordinal()];
                target_bb <<= 8;

                target_bb &= empty_bb;
                streamPawnMoves.run8(EnumMoveType.PAWN_QUIET, target_bb, 16);


                long left_attacks = pawns & 0xfefefefefefefefeL; // mask right column
                left_attacks <<= 7;

                left_attacks &= opponent_bb;
                streamPawnMoves.run8(EnumMoveType.CAPTURE, left_attacks, 7);

                long right_attacks = pawns & 0x7f7f7f7f7f7f7f7fL; // mask left column
                right_attacks <<= 9;

                right_attacks &= opponent_bb;
                streamPawnMoves.run8(EnumMoveType.CAPTURE, right_attacks, 9);


                long en_passant = this.get_enpassant_state();
                if (en_passant != 0) {
                    left_attacks = pawns & 0xfefefefefefefefeL; // mask right column
                    left_attacks <<= 7;
                    left_attacks &= en_passant;
                    if (left_attacks != 0) {
                        int end = Long.numberOfTrailingZeros(left_attacks);
                        streamMoves.run6(EnumMoveType.EN_PASSANT, EnumPieceType.P, end - 7, left_attacks);
                    }

                    right_attacks = pawns & 0x7f7f7f7f7f7f7f7fL; // mask left column
                    right_attacks <<= 9;
                    right_attacks &= en_passant;
                    if (right_attacks != 0) {
                        int end = Long.numberOfTrailingZeros(right_attacks);
                        streamMoves.run6(EnumMoveType.EN_PASSANT, EnumPieceType.P, end - 9, right_attacks);
                    }
                }
            } else {
                long target_bb = (pawns >>> 8) & empty_bb;
                streamPawnMoves.run8(EnumMoveType.PAWN_QUIET, target_bb, -8);

                // Double move
                target_bb &= PAWN_DOUBLE_MOVE_LINES[EnumColor.BLACK.ordinal()];
                target_bb >>>= 8;

                target_bb &= empty_bb;
                streamPawnMoves.run8(EnumMoveType.PAWN_QUIET, target_bb, -16);


                long left_attacks = pawns & 0xfefefefefefefefeL; // mask right column
                left_attacks >>>= 9;

                left_attacks &= opponent_bb;
                streamPawnMoves.run8(EnumMoveType.CAPTURE, left_attacks, -9);

                long right_attacks = pawns & 0x7f7f7f7f7f7f7f7fL; // mask left column
                right_attacks >>>= 7;

                right_attacks &= opponent_bb;
                streamPawnMoves.run8(EnumMoveType.CAPTURE, right_attacks, -7);



                long en_passant = this.get_enpassant_state();
                if (en_passant != 0) {
                    left_attacks = pawns & 0xfefefefefefefefeL; // mask right column
                    left_attacks >>>= 9;
                    left_attacks &= en_passant;
                    if (left_attacks != 0) {
                        int end = Long.numberOfTrailingZeros(left_attacks);
                        streamMoves.run6(EnumMoveType.EN_PASSANT, EnumPieceType.P, end + 9, left_attacks);
                    }

                    right_attacks = pawns & 0x7f7f7f7f7f7f7f7fL; // mask left column
                    right_attacks >>>= 7;
                    right_attacks &= en_passant;
                    if (right_attacks != 0) {
                        int end = Long.numberOfTrailingZeros(right_attacks);
                        streamMoves.run6(EnumMoveType.EN_PASSANT, EnumPieceType.P, end + 7, right_attacks);
                    }
                }

            }
        };

        GenerateMovesForFigure generateRookMoves = () -> {
            long rooks = getBitboard.run(EnumPieceType.R);
            for (int pos : Bitboard.iter(rooks)) {
                long attacks = Bitboard.getRookAttacks(pos, occupied);
                streamPieceMoves.run7(EnumPieceType.R, pos, attacks, opponent_bb, empty_bb);
            }
        };

        GenerateMovesForFigure generateBishopMoves = () -> {
            long bishops = getBitboard.run(EnumPieceType.B);
            for (int pos : Bitboard.iter(bishops)) {
                long attacks = Bitboard.getBishopAttacks(pos, occupied);
                streamPieceMoves.run7(EnumPieceType.B, pos, attacks, opponent_bb, empty_bb);
            }
        };

        GenerateMovesForFigure generateQueenMoves = () -> {
            long queens = getBitboard.run(EnumPieceType.Q);
            for (int pos : Bitboard.iter(queens)) {
                long attacks = Bitboard.getQueenAttacks(pos, occupied);
                streamPieceMoves.run7(EnumPieceType.Q, pos, attacks, opponent_bb, empty_bb);
            }
        };

        generatePawnMoves.run3();
        generateKnightMoves.run3();
        generateRookMoves.run3();
        generateBishopMoves.run3();
        generateQueenMoves.run3();
        generateKingMoves.run3();

        return result;
    }

    public static void main(String[] args) {
        Board board = new Board(fromFen("rnbqkbnr/ppp1pppp/8/3p4/Q7/2P5/PP1PPPPP/RNB1KBNR b KQkq - 1 2"));
        //Board board = new Board(fromFen("rnbqkb1r/ppppnppp/8/1Q2p3/2P3P1/8/PP1PPP1P/RNB1KBNR b KQkq - 0 3"));
        //Board board = new Board(fromFen("rnbqkb1r/ppppnppp/8/4p3/Q1P3P1/8/PP1PPP1P/RNB1KBNR b KQkq - 0 3"));
        board.state.generateLegalMoves();
    }

//    static final int MAX_GAME_HALFMOVES = 5898 * 2;

    static final int BASE_PIECE_PHASE_VALUE = 2;
    static final int PAWN_PHASE_VALUE = -1; // relative to the base piece value
    static final int QUEEN_PHASE_VALUE = 4; // relative to the base piece value

    public static final int MAX_PHASE = 16 * PAWN_PHASE_VALUE + 30 * BASE_PIECE_PHASE_VALUE + 2 * QUEEN_PHASE_VALUE;

//    public int calc_phase_value(long all_pieces, long all_pawns, long white_queens, long black_queens) {
//    }

    public int calc_phase_value() {
        long all_pieces = this.getAllPieceBitboard(EnumColor.WHITE) | this.getAllPieceBitboard(EnumColor.BLACK);
        long all_pawns = this.getPieceBitboard(EnumPieceType.P, EnumColor.BLACK) | this.getPieceBitboard(EnumPieceType.P, EnumColor.WHITE);
        long white_queens = this.getPieceBitboard(EnumPieceType.Q, EnumColor.WHITE);
        long black_queens = this.getPieceBitboard(EnumPieceType.Q, EnumColor.BLACK);

        int pieces_except_king_count = Long.bitCount(all_pieces) - 2; // -2 for two kings

        int white_queen_phase_score = white_queens != 0 ? QUEEN_PHASE_VALUE : 0;
        int black_queen_phase_score = black_queens != 0 ? QUEEN_PHASE_VALUE : 0;
        int queen_phase_score = white_queen_phase_score + black_queen_phase_score;
        int pawn_count = Long.bitCount(all_pawns);

        return pawn_count * PAWN_PHASE_VALUE + pieces_except_king_count * BASE_PIECE_PHASE_VALUE + queen_phase_score;
    }

    public String toFen() {
        return Fen.toFen(this);
    }
}
