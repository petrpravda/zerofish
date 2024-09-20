import {Move} from './Move';

export class TTEntry {
  public static readonly EXACT = 0;
  public static readonly LOWER_BOUND = 1;
  public static readonly UPPER_BOUND = 2;
  public static readonly SIZE = 10; // in bytes (if needed)

  score: number;
  depth: number;
  flag: number;
  private readonly bestMove: number; // store Move as bits (similar to Java's short representation)
  readonly key: number;

  constructor(key: number, score: number, depth: number, flag: number, bestMove: Move) {
    this.key = key;
    this.score = score;
    this.depth = depth;
    this.flag = flag;
    this.bestMove = bestMove.bitsValue(); // Assuming Move has a bits() method returning its bitwise representation
  }

  // Method to retrieve the Move from the stored bit representation
  move(): Move {
    return new Move(this.bestMove); // Assuming Move can be constructed from its bitwise representation
  }
}
