import { BoardState } from './BoardState';
import { TranspositionTable } from './TranspositionTable';
import { Move} from './Move';
import {TTEntry} from './TTEntry';
import {Statistics} from './Statistics';
import {Limits} from './Limits';
import {Evaluation} from './Evaluation';
import {START_POS} from './Fen';

export class Search {
  public static readonly GIGA = 1000000000;
  public static readonly MEGA = 1000000.;
  static readonly INF = 999999;
  private static readonly NULL_MIN_DEPTH = 2;
  private static readonly LMR_MIN_DEPTH = 2;
  private static readonly LMR_MOVES_WO_REDUCTION = 1;
  private static readonly ASPIRATION_WINDOW = 25;
  private static LMR_TABLE: number[][] = Array.from({ length: 64 }, () => Array(64).fill(0));

  private stop: boolean = false;
  private selDepth: number = 0;
  private transpositionTable: TranspositionTable;
  private statistics: Statistics;
  private searchPosition!: BoardState;

  // Initialize the LMR table
  static {
    for (let depth = 1; depth < 64; depth++) {
      for (let moveNumber = 1; moveNumber < 64; moveNumber++) {
        Search.LMR_TABLE[depth][moveNumber] = Math.floor(0.75 + Math.log(depth) * Math.log(moveNumber) / 2.25);
      }
    }
  }

  constructor(transpositionTable: TranspositionTable) {
    this.transpositionTable = transpositionTable;
    this.statistics = new Statistics();
  }

  public async itDeep(position: BoardState, searchDepth: number): Promise<SearchResult> {
    this.searchPosition = position;
    Limits.startTime = performance.now();
    this.selDepth = 0;
    this.stop = false;
    let alpha = -Search.INF;
    let beta = Search.INF;
    let depth = 1;
    let result: SearchResult = new SearchResult(null, 0);

    while (depth <= searchDepth) {
      // Time control can be implemented here
      result = await this.negaMaxRoot(position, depth, alpha, beta);

      if (result.score <= alpha) {
        alpha = -Search.INF;
      } else if (result.score >= beta) {
        beta = Search.INF;
      } else {
        this.printInfo(position, result, depth);
        alpha = result.score - Search.ASPIRATION_WINDOW;
        beta = result.score + Search.ASPIRATION_WINDOW;
        depth++;
        this.statistics.reset();
      }
    }

    return result;
  }

  private async negaMaxRoot(state: BoardState, depth: number, alpha: number, beta: number): Promise<SearchResult> {
    const moves = state.generateLegalMoves();
    let bestMove: Move | null = null;

    for (const move of moves.overSorted(state, this.transpositionTable)) {
      const newBoardState = state.doMove(move);
      const value = -await this.negaMax(newBoardState, depth - 1, 1, -beta, -alpha, true);
      if (value > alpha) {
        bestMove = move;
        if (value >= beta) {
          this.transpositionTable.set(state.hash, beta, depth, TTEntry.LOWER_BOUND, bestMove);
          return new SearchResult(bestMove, beta);
        }
        alpha = value;
        this.transpositionTable.set(state.hash, alpha, depth, TTEntry.UPPER_BOUND, bestMove);
      }
    }

    return new SearchResult(bestMove, alpha);
  }

  private async negaMax(state: BoardState, depth: number, ply: number, alpha: number, beta: number, canApplyNull: boolean): Promise<number> {
    const mateValue = Search.INF - ply;
    let ttFlag = TTEntry.UPPER_BOUND;

    if (this.stop || Limits.checkLimits()) {
      this.stop = true;
      return 0;
    }

    const inCheck = state.isKingAttacked();
    if (depth <= 0 && !inCheck) {
      return await this.quiescence(state, depth, ply, alpha, beta);
    }

    this.statistics.nodes++;

    const ttEntry = this.transpositionTable.probe(state.hash);
    if (ttEntry && ttEntry.depth >= depth) {
      this.statistics.ttHits++;
      switch (ttEntry.flag) {
        case TTEntry.EXACT:
          this.statistics.leafs++;
          return ttEntry.score;
        case TTEntry.LOWER_BOUND:
          alpha = Math.max(alpha, ttEntry.score);
          break;
        case TTEntry.UPPER_BOUND:
          beta = Math.min(beta, ttEntry.score);
          break;
      }
      if (alpha >= beta) {
        this.statistics.leafs++;
        return ttEntry.score;
      }
    }

    // NULL MOVE
    if (Search.canApplyNullWindow(state, depth, beta, inCheck, canApplyNull)) {
      const R = depth > 6 ? 3 : 2;
      const newBoardState = state.doNullMove();
      const value = -await this.negaMax(newBoardState, depth - R - 1, ply, -beta, -beta + 1, false);
      if (this.stop) return 0;
      if (value >= beta) {
        this.statistics.betaCutoffs++;
        return beta;
      }
    }

    const moves = state.generateLegalMoves();
    let value;
    let bestMove: Move = Move.NULL_MOVE;
    let i = 0;

    for (const move of moves.overSorted(state, this.transpositionTable)) {
      let reducedDepth = depth;
      if (Search.canApplyLMR(depth, move, i++)) {
        reducedDepth -= Search.LMR_TABLE[Math.min(depth, 63)][Math.min(i, 63)];
      }

      if (inCheck) reducedDepth++;

      const newBoardState = state.doMove(move);
      value = -await this.negaMax(newBoardState, reducedDepth - 1, ply + 1, -beta, -alpha, true);
      if (this.stop) return 0;

      if (value > alpha) {
        bestMove = move;
        if (value >= beta) {
          this.statistics.betaCutoffs++;
          ttFlag = TTEntry.LOWER_BOUND;
          alpha = beta;
          break;
        }
        ttFlag = TTEntry.EXACT;
        alpha = value;
      }
    }

    if (moves.length === 0) {
      alpha = inCheck ? -mateValue : 0;
    }

    if (!bestMove.equals(Move.NULL_MOVE) && !this.stop) {
      this.transpositionTable.set(state.hash, alpha, depth, ttFlag, bestMove);
    }

    return alpha;
  }

  private async quiescence(state: BoardState, depth: number, ply: number, alpha: number, beta: number): Promise<number> {
    if (this.stop || Limits.checkLimits()) {
      this.stop = true;
      return 0;
    }

    this.selDepth = Math.max(ply, this.selDepth);
    this.statistics.qnodes++;

    let value = Evaluation.evaluateState(state);

    if (value >= beta) {
      this.statistics.qleafs++;
      return beta;
    }

    if (alpha < value) {
      alpha = value;
    }

    const moves = state.generateLegalQuiescence();
    for (const move of moves.overSorted(state, this.transpositionTable)) {
      if (move.isPromotion() && move.flags() !== Move.PR_QUEEN && move.flags() !== Move.PC_QUEEN) {
        continue;
      }

      const newBoardState = state.doMove(move);
      value = -await this.quiescence(newBoardState, depth - 1, ply + 1, -beta, -alpha);

      if (this.stop) return 0;

      if (value > alpha) {
        if (value >= beta) {
          this.statistics.qbetaCutoffs++;
          return beta;
        }
        alpha = value;
      }
    }

    return alpha;
  }

  public static isScoreCheckmate(score: number): boolean {
    return Math.abs(score) >= Search.INF / 2;
  }

  public static canApplyNullWindow(state: BoardState, depth: number, beta: number, inCheck: boolean, canApplyNull: boolean): boolean {
    return canApplyNull &&
      !inCheck &&
      depth >= Search.NULL_MIN_DEPTH &&
      state.hasNonPawnMaterial(state.sideToPlay) &&
      Evaluation.evaluateState(state) >= beta;
  }

  public static canApplyLMR(depth: number, move: Move, moveIndex: number): boolean {
    return depth > Search.LMR_MIN_DEPTH &&
      moveIndex > Search.LMR_MOVES_WO_REDUCTION &&
      move.flags() === Move.QUIET;
  }

  public getPv(state: BoardState, depth: number): string {
    const bestEntry = this.transpositionTable.probe(state.hash);
    if (!bestEntry || depth === 0) {
      return "";
    }
    const bestMove = bestEntry.move();
    const newBoardState = state.doMove(bestMove);
    return bestMove.uci() + " " + this.getPv(newBoardState, depth - 1);
  }

  public stopSearch(): void {
    this.stop = true;
  }

  public printInfo(state: BoardState, result: SearchResult, depth: number): void {
    const streamOut = process.stdout;  // Assuming you're writing to the console

    streamOut.write("info");

    const currMove = result.move ? result.move.toString() : "(none)";
    streamOut.write(` currmove ${currMove}`);

    streamOut.write(` depth ${depth}`);

    streamOut.write(` seldepth ${this.selDepth}`);

    const timeElapsed = Math.floor(Limits.timeElapsed() / Search.MEGA);
    streamOut.write(` time ${timeElapsed}`);

    streamOut.write(` score cp ${result.score}`);

    streamOut.write(` nodes ${this.statistics.totalNodes()}`);

    const nps = this.statistics.totalNodes() / (Limits.timeElapsed() / Search.GIGA);
    streamOut.write(` nps ${nps.toFixed(0)}`);

    const pv = this.getPv(state, depth);
    streamOut.write(` pv ${pv}\n`);
  }
}

class SearchResult {
  public move: Move | null;
  public score: number;

  constructor(move: Move | null, score: number) {
    this.move = move;
    this.score = score;
  }
}

const state = BoardState.fromFen(START_POS);
new Search(new TranspositionTable()).itDeep(state, 8);
