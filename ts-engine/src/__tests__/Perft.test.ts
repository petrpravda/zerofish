// import { BoardState, BoardState.fromFen, perft } from './chess'; // assuming these functions are imported from your project
// import { DynamicTest } from '@jest/types';
// import { TestFactory } from '@jest/test-factory'; // hypothetical import for test factory in Jest

// Tricky perft cases as a string, similar to the Java version
import {BoardState} from '../BoardState';
import {START_POS} from '../Fen';
import {Perft} from '../Perft';

const TRICKY_PERFTS = `avoid illegal en passant capture:
8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1 perft 6 = 824064
8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1 perft 6 = 824064
en passant capture checks opponent:
8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1 perft 6 = 1440467
8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1 perft 6 = 1440467
short castling gives check:
5k2/8/8/8/8/8/8/4K2R w K - 0 1 perft 6 = 661072
4k2r/8/8/8/8/8/8/5K2 b k - 0 1 perft 6 = 661072
long castling gives check:
3k4/8/8/8/8/8/8/R3K3 w Q - 0 1 perft 6 = 803711
r3k3/8/8/8/8/8/8/3K4 b q - 0 1 perft 6 = 803711
castling (including losing cr due to rook capture):
r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1 perft 4 = 1274206
r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1 perft 4 = 1274206
castling prevented:
r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1 perft 4 = 1720476
r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1 perft 4 = 1720476
promote out of check:
2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1 perft 6 = 3821001
3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1 perft 6 = 3821001
discovered check:
5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1 perft 5 = 1004658
8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1 perft 5 = 1004658
promote to give check:
4k3/1P6/8/8/8/8/K7/8 w - - 0 1 perft 6 = 217342
8/k7/8/8/8/8/1p6/4K3 b - - 0 1 perft 6 = 217342
underpromote to check:
8/P1k5/K7/8/8/8/8/8 w - - 0 1 perft 6 = 92683
8/8/8/8/8/k7/p1K5/8 b - - 0 1 perft 6 = 92683
self stalemate:
K1k5/8/P7/8/8/8/8/8 w - - 0 1 perft 6 = 2217
8/8/8/8/8/p7/8/k1K5 b - - 0 1 perft 6 = 2217
stalemate/checkmate:
8/k1P5/8/1K6/8/8/8/8 w - - 0 1 perft 7 = 567584
8/8/8/8/1k6/8/K1p5/8 b - - 0 1 perft 7 = 567584
double check:
8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1 perft 4 = 23527
8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1 perft 4 = 23527
`;

// Function to parse perft cases
interface PerftCase {
  fen: string;
  depth: number;
  count: number;
}

const PERFT_CASE_REGEX = /(.+) perft (\d+) = (\d+)/;

function parsePerftCase(line: string): PerftCase {
  const match = line.match(PERFT_CASE_REGEX);
  if (match) {
    return {
      fen: match[1],
      depth: parseInt(match[2], 10),
      count: parseInt(match[3], 10),
    };
  } else {
    throw new Error(`Line "${line}" cannot be parsed.`);
  }
}

// Unit tests using Jest

describe('PerftTest', () => {

  test('simplePerft2', () => {
    const state = BoardState.fromFen(START_POS);
    expect(Perft.perft(state, 2)).toBe(400);
  });

  test('simplePerft3', () => {
    const state = BoardState.fromFen( START_POS);
    expect(Perft.perft(state, 3)).toBe(8902);
  });

  test('simplePerft4', () => {
    const state = BoardState.fromFen( START_POS);
    expect(Perft.perft(state, 4)).toBe(197281);
  });

  // test('simplePerft6Temp', () => {
  //   const state = BoardState.fromFen("8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1");
  //   expect(Perft.perft(state, 6)).toBe(824064);
  // });
  //
  test('simplePerft5', () => {
    const board = BoardState.fromFen( START_POS);
    expect(Perft.perft(board, 5)).toBe(4865609);
  });

  test('simplePerft5b', () => {
    const board = BoardState.fromFen('rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1');
    expect(Perft.perft(board, 5)).toBe(5363555);
  });

  test('kingIsNotMovingInRust', () => {
    const board = BoardState.fromFen('rnbqkbnr/1ppppppp/8/p7/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2');
    expect(Perft.perft(board, 1)).toBe(28);
  });

  test('pawnIsNotTakingInRust', () => {
    const board = BoardState.fromFen('rnbqkb1r/pppppppp/8/8/4n3/3P4/PPPKPPPP/RNBQ1BNR w kq - 3 3');
    expect(Perft.perft(board, 1)).toBe(3);
  });

  test('simpleEnPassant', () => {
    const state = BoardState.fromFen('rnbqkbnr/p2ppppp/2p5/Pp6/8/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 3');
    expect(Perft.perft(state, 1)).toBe(23);
  });

  test('simpleEnPassant2', () => {
    const state = BoardState.fromFen('r1b2rk1/p3npbp/2p1p1p1/2qpP3/8/2P2N2/PP1N1PPP/R1BQR1K1 w Qq d6 1 1');
    expect(Perft.perft(state, 1)).toBe(29);
  });

  // Dynamic tests for tricky perft cases
  test.each(generateDynamicTests())('tricky perfts - %s', (description, perftWhite, perftBlack) => {
    expect(Perft.perft(BoardState.fromFen(perftWhite.fen), perftWhite.depth)).toBe(perftWhite.count);
    expect(Perft.perft(BoardState.fromFen(perftBlack.fen), perftBlack.depth)).toBe(perftBlack.count);
  });
});

// Function to generate dynamic test cases
function generateDynamicTests(): [string, PerftCase, PerftCase][] {
  const lines = TRICKY_PERFTS.split('\n').filter(line => line.trim());
  const pattern = /(.*?):\n(.*?)\n(.*?)\n/;
  const results: [string, PerftCase, PerftCase][] = [];

  for (let i = 0; i < lines.length; i += 3) {
    const description = lines[i].trim();
    const testWhite = parsePerftCase(lines[i + 1]);
    const testBlack = parsePerftCase(lines[i + 2]);

    results.push([description, testWhite, testBlack]);
  }
  return results;
}

