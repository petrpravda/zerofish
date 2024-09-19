import {BoardState} from '../BoardState';
import {Side} from '../Side';
import {Square} from '../Square';
import {Fen, START_POS} from '../Fen';
import {WHITE_KINGS_ROOK_MASK} from '../Bitboard';
import {zeroBB} from '../BB64Long';

describe('Fen', () => {

  test('toFen should match the initial position', () => {
    // Arrange
    const state = BoardState.fromFen(START_POS);

    // Act
    const fen = Fen.toFen(state);

    // Assert
    expect(fen).toBe(START_POS);
  });

  test('fromFen should parse the starting position correctly', () => {
    // Arrange
    const fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    // Act
    const state = Fen.fromFen(fen, null);

    // Assert
    expect(state).not.toBeNull();
    expect(state.sideToPlay).toBe(Side.WHITE);
    expect(state.halfMoveClock).toBe(0);
    expect((state.fullMoveNormalized / 2) + 1).toBe(1);
  });

  test('expandFenPieces should expand compressed FEN correctly', () => {
    // Arrange
    const fenPieces = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

    // Act
    const expandedFen = Fen.expandFenPieces(fenPieces);

    // Assert
    const expectedExpandedFen = "rnbqkbnr/pppppppp/11111111/11111111/11111111/11111111/PPPPPPPP/RNBQKBNR";
    expect(expandedFen).toBe(expectedExpandedFen);
  });

  // test('fromFenFree should extract a valid FEN string', () => {
  //   // Arrange
  //   const fenFree = "Fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
  //
  //   // Act
  //   const boardPosition = Fen.fromFenFree(fenFree);
  //
  //   // Assert
  //   expect(boardPosition).not.toBeNull();
  // });
  //
  // test('fromFenFree should throw an error for an invalid FEN string', () => {
  //   // Arrange
  //   const invalidFenFree = "This is an invalid FEN string";
  //
  //   // Act & Assert
  //   expect(() => Fen.fromFenFree(invalidFenFree)).toThrowError("This is an invalid FEN string doesn't contain 'Fen: '");
  // });
  //
  // test('fromFen should parse a complex position correctly', () => {
  //   // Arrange
  //   const fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 2";
  //
  //   // Act
  //   const state = Fen.fromFen(fen, null);
  //
  //   // Assert
  //   expect(state).not.toBeNull();
  //   expect(state.sideToPlay).toBe(Side.BLACK);
  //   expect((state.fullMoveNormalized / 2) + 1).toBe(2);
  //   expect(state.halfMoveClock).toBe(0);
  //   expect(Square.getName(state.enPassant.LSB())).toBe('e3');
  // });
  //
  // test('kingsideCastlingFlagsRookMoving should correctly update castling flags when the rook moves', () => {
  //   // Arrange
  //   const position = Fen.fromFenFree(`
  //       +---+---+---+---+---+---+---+---+
  //       | r |   |   |   | k |   |   | r |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       | r |   |   |   | K |   |   | r |
  //       +---+---+---+---+---+---+---+---+
  //
  //       Fen: r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 1 1
  //       `);
  //   let state = position.getState();
  //
  //   // Act
  //   state = position.doMove("h1g1");
  //
  //   // Assert
  //   expect(state.movements.AND(WHITE_KINGS_ROOK_MASK)).not.toBe(zeroBB());
  //   expect(state.toFen()).toContain("Qkq");
  // });
  //
  // test('kingsideCastlingFlagsKingMoving should correctly update castling flags when the king moves', () => {
  //   // Arrange
  //   const position = Fen.fromFenFree(`
  //       +---+---+---+---+---+---+---+---+
  //       | r |   |   |   | k |   |   | r |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   |
  //       +---+---+---+---+---+---+---+---+
  //       | r |   |   |   | K |   |   | r |
  //       +---+---+---+---+---+---+---+---+
  //
  //       Fen: r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 1 1
  //       `);
  //   let state = position.getState();
  //
  //   // Act
  //   state = position.doMove("e1d1");
  //
  //   // Assert
  //   expect(state.toFen()).toContain("kq");
  // });
  //
  // test('sanitizeCrappyCastlingFlags should sanitize invalid castling flags in FEN', () => {
  //   // Arrange
  //   const position = Fen.fromFenFree(`
  //       +---+---+---+---+---+---+---+---+
  //       | r |   |   | q |   | r | k |   | 8
  //       +---+---+---+---+---+---+---+---+
  //       | p | b | p |   |   | p | p | p | 7
  //       +---+---+---+---+---+---+---+---+
  //       |   | p | n | p |   | n |   |   | 6
  //       +---+---+---+---+---+---+---+---+
  //       |   |   |   |   |   |   |   |   | 5
  //       +---+---+---+---+---+---+---+---+
  //       |   |   | P | P | p |   |   |   | 4
  //       +---+---+---+---+---+---+---+---+
  //       |   |   | P |   | P |   |   |   | 3
  //       +---+---+---+---+---+---+---+---+
  //       | P |   | N |   |   | P | P | P | 2
  //       +---+---+---+---+---+---+---+---+
  //       | r |   | B | Q | K | B |   | r | 1
  //       +---+---+---+---+---+---+---+---+
  //         a   b   c   d   e   f   g   h
  //
  //       Fen: r2q1rk1/pbp2ppp/1pnp1n2/8/2PPp3/2P1P3/P1N2PPP/R1BQKB1R w kq - 0 10
  //       `);
  //
  //   // Assert
  //   expect(position.getState().toFen()).toBe("r2q1rk1/pbp2ppp/1pnp1n2/8/2PPp3/2P1P3/P1N2PPP/R1BQKB1R w - - 0 10");
  // });
});

