import {BoardState} from './BoardState';
import {Move} from './Move';

export class Perft {

  /**
   * Performs a perft search on the given board state up to the specified depth.
   * @param state The current board state.
   * @param depth The depth of the search.
   * @returns The number of nodes (legal moves) generated.
   */
  public static perft(state: BoardState, depth: number): number {
    const moves = state.generateLegalMoves();

    if (depth === 1) {
      return moves.length; // In TypeScript, use .length instead of .size()
    }

    let nodes = 0;

    moves.forEach((move: Move) => {
      const newBoardState = state.doMove(move);
      const count = Perft.perft(newBoardState, depth - 1);
      nodes += count;
    });

    return nodes;
  }

  /**
   * Generates a detailed perft result in a string format.
   * @param state The current board state.
   * @param depth The depth of the search.
   * @returns A formatted string with per-move breakdown and total nodes searched.
   */
  public static perftString(state: BoardState, depth: number): string {
    const moveList = state.generateLegalMoves();
    let nodes = 0;

    const list = moveList.map((move: Move) => {
      let count = 0;

      if (depth > 1) {
        const newBoardState = state.doMove(move);
        count = Perft.perft(newBoardState, depth - 1);
      } else {
        count = 1; // At depth 1, each legal move counts as 1
      }

      nodes += count;
      return `${move.toString()}: ${count}`;
    });

    const tableData = list.join("\n");
    return `${tableData}\n\nNodes searched: ${nodes}`;
  }

}
