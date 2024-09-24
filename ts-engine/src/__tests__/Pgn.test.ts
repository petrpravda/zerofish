import {Pgn} from '../Pgn';
import {START_POS} from '../Fen';
import * as fs from 'fs/promises';

describe('Pgn', () => {
  describe('uciToPgn', () => {
    it('should convert UCI moves to PGN format correctly', () => {
      const uciMoves: string[] = [
        "e2e4", "d7d5", "e4d5", "d8d5", "b1c3", "d5a5",
        "d1e2", "g8f6", "g1f3", "e7e6", "e2b5", "a5b5", "c3b5"
      ];

      const expectedPgnMoves: string[] = [
        "e4", "d5", "exd5", "Qxd5", "Nc3", "Qa5", "Qe2", "Nf6", "Nf3", "e6", "Qb5+", "Qxb5", "Nxb5"
      ];

      const resultPgnMoves: string[] = Pgn.uciToPgn(START_POS, uciMoves);
      expect(resultPgnMoves).toEqual(expectedPgnMoves);
    });

    // Test case for SIX_MOVES
    it('should convert SIX_MOVES UCI to PGN format correctly', () => {
      const uciMoves: string[] = ["d2d4", "g8f6", "g1f3", "e7e6", "c2c4", "c7c5"];

      const expectedPgnMoves: string[] = ["d4", "Nf6", "Nf3", "e6", "c4", "c5"];

      const resultPgnMoves: string[] = Pgn.uciToPgn(START_POS, uciMoves);
      expect(resultPgnMoves).toEqual(expectedPgnMoves);
    });

    // Test case for FULL_MOVES
    it('should convert FULL_MOVES UCI to PGN format correctly', () => {
      const uciMoves: string[] = 'd2d4 g8f6 g1f3 e7e6 c2c4 c7c5 e2e3 b8c6 d4d5 c6b4 b1c3 f6g4 a2a3 d8a5 c1d2 f8e7 h2h3 g4f6 a3b4 a5b4 c3b5 b4b2 d2c3 b2b5 c4b5 b7b6 d5d6 e7d8 f1d3 h7h6 e3e4 f6h5 e1g1 c8b7 f1e1 d8f6 e4e5 f6d8 d3e4 b7e4 e1e4 e8g8 e4a4 f7f6 g2g4 f6e5 g4h5 d8f6 f3e5 a8c8 a4g4 c5c4 a1a7 c8c5 a7d7 c5d5 d1f3 f6e5 d7g7 e5g7 g4g7 g8h8 f3f8'.split(/\s+/);
      const expectedPgnMoves: string[] = 'd4 Nf6 Nf3 e6 c4 c5 e3 Nc6 d5 Nb4 Nc3 Ng4 a3 Qa5 Bd2 Be7 h3 Nf6 axb4 Qxb4 Nb5 Qxb2 Bc3 Qxb5 cxb5 b6 d6 Bd8 Bd3 h6 e4 Nh5 O-O Bb7 Re1 Bf6 e5 Bd8 Be4 Bxe4 Rxe4 O-O Rea4 f6 g4 fxe5 gxh5 Bf6 Nxe5 Rac8 Rg4 c4 Rxa7 Rc5 Rxd7 Rd5 Qf3 Bxe5 Rdxg7+ Bxg7 Rxg7+ Kh8 Qxf8#'.split(/\s+/);

      const resultPgnMoves: string[] = Pgn.uciToPgn(START_POS, uciMoves);
      expect(resultPgnMoves).toEqual(expectedPgnMoves);
    });


    // Test case for PROMOTION_MOVES
    it('should convert PROMOTION_MOVES UCI to PGN format correctly', () => {
      const uciMoves: string[] = 'b2b3 c7c5 c1b2 b8c6 g2g3 d7d6 f1g2 g8f6 c2c4 a7a6 b1c3 e7e5 d2d3 c6d4 e2e3 c8g4 d1d2 d4f5 g1e2 g4e2 d2e2 g7g6 g2b7 a8b8 b7c6 f6d7 e1g1 f8g7 c6g2 e8g8 c3d5 d7b6 d5b6 b8b6 g2h3 d8f6 f2f4 b6b4 f4e5 d6e5 e3e4 f6e7 e4f5 g8h8 a1e1 b4b8 f5f6 g7f6 f1f6 e7f6 b2e5 f6e5 e2e5 h8g8 h3g2 b8e8 e5e8 f8e8 e1e8 g8g7 g2d5 h7h5 b3b4 c5b4 c4c5 b4b3 d5b3 f7f5 c5c6 f5f4 c6c7 f4g3 c7c8q g3h2 g1h2 h5h4 c8e6 g7h6 e8g8 h6g5 g8g6 g5h5 e6g4'.split(/\s+/);
      const expectedPgnMoves: string[] = 'b3 c5 Bb2 Nc6 g3 d6 Bg2 Nf6 c4 a6 Nc3 e5 d3 Nd4 e3 Bg4 Qd2 Nf5 Nge2 Bxe2 Qxe2 g6 Bxb7 Rb8 Bc6+ Nd7 O-O Bg7 Bg2 O-O Nd5 Nb6 Nxb6 Rxb6 Bh3 Qf6 f4 Rb4 fxe5 dxe5 e4 Qe7 exf5 Kh8 Rae1 Rbb8 f6 Bxf6 Rxf6 Qxf6 Bxe5 Qxe5 Qxe5+ Kg8 Bg2 Rbe8 Qxe8 Rxe8 Rxe8+ Kg7 Bd5 h5 b4 cxb4 c5 b3 Bxb3 f5 c6 f4 c7 fxg3 c8=Q gxh2+ Kxh2 h4 Qe6 Kh6 Rg8 Kg5 Rxg6+ Kh5 Qg4#'.split(/\s+/);

      const resultPgnMoves: string[] = Pgn.uciToPgn(START_POS, uciMoves);
      expect(resultPgnMoves).toEqual(expectedPgnMoves);
    });


    // Test case for COMPLEX_MOVES
    it('should convert COMPLEX_MOVES UCI to PGN format correctly', () => {
      const uciMoves: string[] = 'd2d4 d7d5 c2c4 d5c4 e2e4 b8c6 g1f3 c8g4 d4d5 c6e5 b1d2 g8f6 d2c4 e5c4 f1c4 f6e4 h2h3 g4f3 d1f3 e4d6 c4b3 g7g6 f3c3 f7f6 e1g1 a8c8 f1e1 f8g7 c1f4 e8g8 c3e3 b7b6 e3e7 f8e8 f4d6 e8e7 d6e7 d8d7 d5d6 g8h8 b3e6 d7e8 d6d7 e8e7 d7c8q e7f8 c8c7 f6f5 a1d1 g7f6 d1d7'.split(/\s+/);
      const expectedPgnMoves: string[] = 'd4 d5 c4 dxc4 e4 Nc6 Nf3 Bg4 d5 Ne5 Nbd2 Nf6 Nxc4 Nxc4 Bxc4 Nxe4 h3 Bxf3 Qxf3 Nd6 Bb3 g6 Qc3 f6 O-O Rc8 Re1 Bg7 Bf4 O-O Qe3 b6 Qxe7 Re8 Bxd6 Rxe7 Bxe7 Qd7 d6+ Kh8 Be6 Qe8 d7 Qxe7 dxc8=Q+ Qf8 Qxc7 f5 Rad1 Bf6 Rd7'.split(/\s+/);

      const resultPgnMoves: string[] = Pgn.uciToPgn(START_POS, uciMoves);
      expect(resultPgnMoves).toEqual(expectedPgnMoves);
    });


    // Test case for PROMOTION_WO_CAPTURE_MOVES
    it('should convert PROMOTION_WO_CAPTURE_MOVES UCI to PGN format correctly', () => {
      const uciMoves: string[] = 'e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4 a7a6 b1c3 b7b5 f1d3 d8b6 d4b3 b6c7 e1g1 g8f6 f1e1 f8d6 g2g3 b5b4 c3e2 b8c6 e2d4 c6d4 b3d4 h7h5 d3f1 h5h4 f1g2 h4g3 h2g3 c8b7 c1d2 d6c5 c2c3 c7b6 d1e2 d7d5 e4e5 f6e4 d2e3 b6c7 c3c4 c7e5 d4b3 c5e3 e2e3 e5h5 e3b6 a8b8 c4c5 h5h2 g1f1 e8g8 c5c6 b7a8 b6a6 f7f5 c6c7 b8e8 b3d4 e4d2 f1e2 h2g2 c7c8q g2e4 e2d1 e4d4 a6e6 e8e6 c8e6 f8f7 e6e8 f7f8 e8e6 f8f7 e6e8'.split(/\s+/);
      const expectedPgnMoves: string[] = 'e4 c5 Nf3 e6 d4 cxd4 Nxd4 a6 Nc3 b5 Bd3 Qb6 Nb3 Qc7 O-O Nf6 Re1 Bd6 g3 b4 Ne2 Nc6 Ned4 Nxd4 Nxd4 h5 Bf1 h4 Bg2 hxg3 hxg3 Bb7 Bd2 Bc5 c3 Qb6 Qe2 d5 e5 Ne4 Be3 Qc7 c4 Qxe5 Nb3 Bxe3 Qxe3 Qh5 Qb6 Rb8 c5 Qh2+ Kf1 O-O c6 Ba8 Qxa6 f5 c7 Rbe8 Nd4 Nd2+ Ke2 Qxg2 c8=Q Qe4+ Kd1 Qxd4 Qaxe6+ Rxe6 Qxe6+ Rf7 Qe8+ Rf8 Qe6+ Rf7 Qe8+'.split(/\s+/);

      const resultPgnMoves: string[] = Pgn.uciToPgn(START_POS, uciMoves);
      expect(resultPgnMoves).toEqual(expectedPgnMoves);
    });
  });
  describe('uciToPgn', () => {
    // Test case for FULL_MOVES
    it('should convert FULL_MOVES PGN to UCI format correctly', () => {
      const pgnMoves: string[] = 'd4 Nf6 Nf3 e6 c4 c5 e3 Nc6 d5 Nb4 Nc3 Ng4 a3 Qa5 Bd2 Be7 h3 Nf6 axb4 Qxb4 Nb5 Qxb2 Bc3 Qxb5 cxb5 b6 d6 Bd8 Bd3 h6 e4 Nh5 O-O Bb7 Re1 Bf6 e5 Bd8 Be4 Bxe4 Rxe4 O-O Rea4 f6 g4 fxe5 gxh5 Bf6 Nxe5 Rac8 Rg4 c4 Rxa7 Rc5 Rxd7 Rd5 Qf3 Bxe5 Rdxg7+ Bxg7 Rxg7+ Kh8 Qxf8#'.split(/\s+/);
      const expectedUciMoves: string[] = 'd2d4 g8f6 g1f3 e7e6 c2c4 c7c5 e2e3 b8c6 d4d5 c6b4 b1c3 f6g4 a2a3 d8a5 c1d2 f8e7 h2h3 g4f6 a3b4 a5b4 c3b5 b4b2 d2c3 b2b5 c4b5 b7b6 d5d6 e7d8 f1d3 h7h6 e3e4 f6h5 e1g1 c8b7 f1e1 d8f6 e4e5 f6d8 d3e4 b7e4 e1e4 e8g8 e4a4 f7f6 g2g4 f6e5 g4h5 d8f6 f3e5 a8c8 a4g4 c5c4 a1a7 c8c5 a7d7 c5d5 d1f3 f6e5 d7g7 e5g7 g4g7 g8h8 f3f8'.split(/\s+/);

      const resultUciMoves: string[] = Pgn.pgnToUci(START_POS, pgnMoves);
      expect(resultUciMoves).toEqual(expectedUciMoves);
    });

    // Test case for PROMOTION_MOVES
    it('should convert PROMOTION_MOVES PGN to UCI format correctly', () => {
      const pgnMoves: string[] = 'b3 c5 Bb2 Nc6 g3 d6 Bg2 Nf6 c4 a6 Nc3 e5 d3 Nd4 e3 Bg4 Qd2 Nf5 Nge2 Bxe2 Qxe2 g6 Bxb7 Rb8 Bc6+ Nd7 O-O Bg7 Bg2 O-O Nd5 Nb6 Nxb6 Rxb6 Bh3 Qf6 f4 Rb4 fxe5 dxe5 e4 Qe7 exf5 Kh8 Rae1 Rbb8 f6 Bxf6 Rxf6 Qxf6 Bxe5 Qxe5 Qxe5+ Kg8 Bg2 Rbe8 Qxe8 Rxe8 Rxe8+ Kg7 Bd5 h5 b4 cxb4 c5 b3 Bxb3 f5 c6 f4 c7 fxg3 c8=Q gxh2+ Kxh2 h4 Qe6 Kh6 Rg8 Kg5 Rxg6+ Kh5 Qg4#'.split(/\s+/);
      const expectedUciMoves: string[] = 'b2b3 c7c5 c1b2 b8c6 g2g3 d7d6 f1g2 g8f6 c2c4 a7a6 b1c3 e7e5 d2d3 c6d4 e2e3 c8g4 d1d2 d4f5 g1e2 g4e2 d2e2 g7g6 g2b7 a8b8 b7c6 f6d7 e1g1 f8g7 c6g2 e8g8 c3d5 d7b6 d5b6 b8b6 g2h3 d8f6 f2f4 b6b4 f4e5 d6e5 e3e4 f6e7 e4f5 g8h8 a1e1 b4b8 f5f6 g7f6 f1f6 e7f6 b2e5 f6e5 e2e5 h8g8 h3g2 b8e8 e5e8 f8e8 e1e8 g8g7 g2d5 h7h5 b3b4 c5b4 c4c5 b4b3 d5b3 f7f5 c5c6 f5f4 c6c7 f4g3 c7c8q g3h2 g1h2 h5h4 c8e6 g7h6 e8g8 h6g5 g8g6 g5h5 e6g4'.split(/\s+/);

      const resultUciMoves: string[] = Pgn.pgnToUci(START_POS, pgnMoves);
      expect(resultUciMoves).toEqual(expectedUciMoves);
    });

    // Test case for COMPLEX_MOVES
    it('should convert COMPLEX_MOVES PGN to UCI format correctly', () => {
      const pgnMoves: string[] = 'd4 d5 c4 dxc4 e4 Nc6 Nf3 Bg4 d5 Ne5 Nbd2 Nf6 Nxc4 Nxc4 Bxc4 Nxe4 h3 Bxf3 Qxf3 Nd6 Bb3 g6 Qc3 f6 O-O Rc8 Re1 Bg7 Bf4 O-O Qe3 b6 Qxe7 Re8 Bxd6 Rxe7 Bxe7 Qd7 d6+ Kh8 Be6 Qe8 d7 Qxe7 dxc8=Q+ Qf8 Qxc7 f5 Rad1 Bf6 Rd7'.split(/\s+/);
      const expectedUciMoves: string[] = 'd2d4 d7d5 c2c4 d5c4 e2e4 b8c6 g1f3 c8g4 d4d5 c6e5 b1d2 g8f6 d2c4 e5c4 f1c4 f6e4 h2h3 g4f3 d1f3 e4d6 c4b3 g7g6 f3c3 f7f6 e1g1 a8c8 f1e1 f8g7 c1f4 e8g8 c3e3 b7b6 e3e7 f8e8 f4d6 e8e7 d6e7 d8d7 d5d6 g8h8 b3e6 d7e8 d6d7 e8e7 d7c8q e7f8 c8c7 f6f5 a1d1 g7f6 d1d7'.split(/\s+/);

      const resultUciMoves: string[] = Pgn.pgnToUci(START_POS, pgnMoves);
      expect(resultUciMoves).toEqual(expectedUciMoves);
    });

    // Test case for PROMOTION_WO_CAPTURE_MOVES
    it('should convert PROMOTION_WO_CAPTURE_MOVES PGN to UCI format correctly', () => {
      const pgnMoves: string[] = 'e4 c5 Nf3 e6 d4 cxd4 Nxd4 a6 Nc3 b5 Bd3 Qb6 Nb3 Qc7 O-O Nf6 Re1 Bd6 g3 b4 Ne2 Nc6 Ned4 Nxd4 Nxd4 h5 Bf1 h4 Bg2 hxg3 hxg3 Bb7 Bd2 Bc5 c3 Qb6 Qe2 d5 e5 Ne4 Be3 Qc7 c4 Qxe5 Nb3 Bxe3 Qxe3 Qh5 Qb6 Rb8 c5 Qh2+ Kf1 O-O c6 Ba8 Qxa6 f5 c7 Rbe8 Nd4 Nd2+ Ke2 Qxg2 c8=Q Qe4+ Kd1 Qxd4 Qaxe6+ Rxe6 Qxe6+ Rf7 Qe8+ Rf8 Qe6+ Rf7 Qe8+'.split(/\s+/);
      const expectedUciMoves: string[] = 'e2e4 c7c5 g1f3 e7e6 d2d4 c5d4 f3d4 a7a6 b1c3 b7b5 f1d3 d8b6 d4b3 b6c7 e1g1 g8f6 f1e1 f8d6 g2g3 b5b4 c3e2 b8c6 e2d4 c6d4 b3d4 h7h5 d3f1 h5h4 f1g2 h4g3 h2g3 c8b7 c1d2 d6c5 c2c3 c7b6 d1e2 d7d5 e4e5 f6e4 d2e3 b6c7 c3c4 c7e5 d4b3 c5e3 e2e3 e5h5 e3b6 a8b8 c4c5 h5h2 g1f1 e8g8 c5c6 b7a8 b6a6 f7f5 c6c7 b8e8 b3d4 e4d2 f1e2 h2g2 c7c8q g2e4 e2d1 e4d4 a6e6 e8e6 c8e6 f8f7 e6e8 f7f8 e8e6 f8f7 e6e8'.split(/\s+/);

      const resultUciMoves: string[] = Pgn.pgnToUci(START_POS, pgnMoves);
      expect(resultUciMoves).toEqual(expectedUciMoves);
    });
  });
});

describe('UCI to PGN conversion from file', () => {
  const uciFilePath = 'src/__tests__/games/uci.5000.games.txt';
  const pgnFilePath = 'src/__tests__/games/pgn.5000.games.txt';

  // Increase the timeout for this long-running test
  it('should correctly translate UCI moves to PGN for all games in the files (stop after N tests)', async () => {
    let testCount = 0;
    const maxTests = 200;

    const uciData = await fs.readFile(uciFilePath, 'utf-8');
    const pgnData = await fs.readFile(pgnFilePath, 'utf-8');

    // Split the content by line
    const uciLines: string[] = uciData.split('\n').map(line => line.trim()).filter(line => line.length > 0);
    const pgnLines: string[] = pgnData.split('\n').map(line => line.trim()).filter(line => line.length > 0);

    expect(uciLines.length).toEqual(pgnLines.length);

    for (let i = 0; i < uciLines.length; i++) {
      if (testCount >= maxTests) {
        break;
      }

      const lineUci = uciLines[i];
      const linePgn = pgnLines[i];

      // Split moves by whitespace
      const uciMoves: string[] = lineUci.split(/\s+/);
      const pgnMoves: string[] = linePgn.split(/\s+/);

      // Convert UCI moves to PGN moves
      const resultPgnMoves: string[] = Pgn.uciToPgn(START_POS, uciMoves);

      // Compare the translated PGN moves to the expected ones
      try {
        expect(resultPgnMoves).toEqual(pgnMoves);
      } catch (error) {
        throw new Error(`Test failed for game ${i + 1}. \nUCI: ${lineUci}. \nExpected PGN: ${linePgn}. \n         Got: ${resultPgnMoves.join(' ')}`);
      }


      // Convert PGN moves to UCI moves
      const resultUciMoves: string[] = Pgn.pgnToUci(START_POS, pgnMoves);

      // Compare the translated PGN moves to the expected ones
      try {
        expect(resultUciMoves).toEqual(uciMoves);
      } catch (error) {
        throw new Error(`Test failed for game ${i + 1}. \nPGN: ${linePgn}. \nExpected UCIs: ${lineUci}. \n          Got: ${resultUciMoves.join(' ')}`);
      }


      // Increment the test count
      testCount++;
    }
  }, 300000); // 30 seconds timeout for this long-running test
});
