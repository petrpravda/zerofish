// Import the PgnUtils class or functions
import {Pgn} from '../Pgn';

describe('PgnUtils', () => {
  describe('uciToPgn', () => {
    it('should convert UCI moves to PGN format correctly', () => {
      // Given FEN and UCI moves
      const startFen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
      const uciMoves: string[] = [
        "e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5a5",
        "d1e2", "g8f6", "g1f3", "e7e6", "e2b5", "a5b5", "c3b5"
      ];

      // Expected result
      const expectedPgnMoves: string[] = [
        "e4", "d5", "exd5", "Qxd5", "Nc3", "Qa5", "Qe2", "Nf6", "Nf3", "e6", "Qb5+", "Qxb5", "Nxb5"
      ];

      // Call the function
      const resultPgnMoves: string[] = Pgn.uciToPgn(startFen, uciMoves);

      // Assert the result is as expected
      expect(resultPgnMoves).toEqual(expectedPgnMoves);
    });
  });
});
