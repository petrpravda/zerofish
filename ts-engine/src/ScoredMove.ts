import {Move} from './Move';

export class ScoredMove {
  readonly move: Move;
  readonly score: number;

  constructor(move: Move, score: number) {
    this.move = move;
    this.score = score;
  }

  // toString method
  toString(): string {
    return `ScoredMove[move=${this.move}, score=${this.score}]`;
  }
}
