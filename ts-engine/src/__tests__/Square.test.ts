import {Square} from '../Square';
import {Side} from '../Side';

describe('Square Tests', () => {

  test('testGetName', () => {
    // Test a few known squares
    expect(Square.getName(Square.A1)).toBe('a1');
    expect(Square.getName(Square.H1)).toBe('h1');
    expect(Square.getName(27)).toBe('d4'); // d4 corresponds to index 27
    expect(Square.getName(Square.E8)).toBe('e8');
  });

  test('testGetSquareFromName', () => {
    // Test getting squares from UCI notation
    expect(Square.getSquareFromName('a1')).toBe(Square.A1);
    expect(Square.getSquareFromName('h1')).toBe(Square.H1);
    expect(Square.getSquareFromName('d4')).toBe(27); // d4 corresponds to index 27
    expect(Square.getSquareFromName('e8')).toBe(Square.E8);
  });

  test('testGetFile', () => {
    // Test file extraction
    expect(Square.getFile(Square.A1)).toBe('a');
    expect(Square.getFile(Square.H1)).toBe('h');
    expect(Square.getFile(27)).toBe('d'); // d4 corresponds to index 27
    expect(Square.getFile(Square.E8)).toBe('e');
  });

  test('testGetRank', () => {
    // Test rank extraction
    expect(Square.getRank(Square.A1)).toBe('1');
    expect(Square.getRank(Square.A8)).toBe('8');
    expect(Square.getRank(27)).toBe('4'); // d4 corresponds to index 27
    expect(Square.getRank(Square.E8)).toBe('8');
  });

  test('testGetRankIndex', () => {
    // Test rank index extraction
    expect(Square.getRankIndex(Square.A1)).toBe(0);  // Rank 1 (index 0)
    expect(Square.getRankIndex(Square.A8)).toBe(7);  // Rank 8 (index 7)
    expect(Square.getRankIndex(27)).toBe(3);         // d4 corresponds to rank 4 (index 3)
  });

  test('testGetFileIndex', () => {
    // Test file index extraction
    expect(Square.getFileIndex(Square.A1)).toBe(0);  // File 'a' (index 0)
    expect(Square.getFileIndex(Square.H1)).toBe(7);  // File 'h' (index 7)
    expect(Square.getFileIndex(27)).toBe(3);         // d4 corresponds to file 'd' (index 3)
  });

  test('testGetDiagonalIndex', () => {
    // Test diagonal index calculation
    expect(Square.getDiagonalIndex(Square.A1)).toBe(7);  // Main diagonal
    expect(Square.getDiagonalIndex(Square.H8)).toBe(7);  // Main diagonal
    expect(Square.getDiagonalIndex(Square.E4)).toBe(6);
  });

  test('testGetAntiDiagonalIndex', () => {
    // Test anti-diagonal index calculation
    expect(Square.getAntiDiagonalIndex(Square.A1)).toBe(0);  // Main anti-diagonal
    expect(Square.getAntiDiagonalIndex(Square.H8)).toBe(14); // Main anti-diagonal
    expect(Square.getAntiDiagonalIndex(Square.E4)).toBe(7);  // Anti-diagonal index for e4
  });

  test('testDirection', () => {
    // Test direction adjustment based on side
    expect(Square.direction(Square.FORWARD, Side.WHITE)).toBe(Square.FORWARD);
    expect(Square.direction(Square.FORWARD, Side.BLACK)).toBe(-Square.FORWARD);

    expect(Square.direction(Square.RIGHT, Side.WHITE)).toBe(Square.RIGHT);
    expect(Square.direction(Square.RIGHT, Side.BLACK)).toBe(-Square.RIGHT);
  });

});
