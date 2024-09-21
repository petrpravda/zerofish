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
    const startTime = performance.now(); // Start time measurement (milliseconds)
    let nodes = 0;
    const moveList = state.generateLegalMoves();

    moveList.forEach((move: Move) => {
      //process.stdout.write(move.toString() + " "); // Print the move in UCI format on the same line
      console.info(move.toString() + " "); // Print the move in UCI format on the same line
      let moveNodes = 0;

      const newBoardState = state.doMove(move);
      if (depth > 1) {
        moveNodes = Perft.perft(newBoardState, depth - 1);
      } else {
        moveNodes = 1; // At depth 1, each legal move counts as 1
      }

      nodes += moveNodes;
      console.log(moveNodes); // Print moveNodes count for the move
    });

    const endTime = performance.now(); // End time measurement
    const timeTaken = (endTime - startTime) / 1000; // Convert to seconds

    // console.log(`Nodes: ${nodes}`);
    // console.log(`Time: ${timeTaken.toFixed(3)}s`);
    // console.log(`NPS: ${(nodes / timeTaken).toFixed(0)}`);

    return `Nodes: ${nodes}\nTime: ${timeTaken.toFixed(3)}s\nNPS: ${(nodes / timeTaken).toFixed(0)}`;
  }

}
