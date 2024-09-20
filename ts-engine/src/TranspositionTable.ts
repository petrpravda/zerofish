import {TTEntry} from './TTEntry';
import {Move} from './Move';

export class TranspositionTable {
  private table: Map<number, TTEntry> = new Map();

  // Method to set an entry in the transposition table
  set(key: number, score: number, depth: number, flag: number, bestMove: Move): void {
    this.table.set(key, new TTEntry(key, score, depth, flag, bestMove));
  }

  // Method to probe (retrieve) an entry from the table
  probe(key: number): TTEntry | undefined {
    return this.table.get(key);
  }

  // Method to reset (clear) the table
  reset(): void {
    this.table.clear();
  }
}
