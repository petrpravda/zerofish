import { BoardState } from './BoardState';
import {Side} from './Side'; // Example imports, adjust as needed

// class ParamEvaluation {
//   constructor(public method: Function, public parameter: Parameter) {}
// }

export class Evaluation {
  // private static PARAMETERS: ParamEvaluation[];
  public static whiteKingSq: number;
  public static blackKingSq: number;
  public static allPieces: bigint;

  // static {
  //   const methods = Object.getOwnPropertyNames(Evaluation.prototype).filter(
  //     (name) => typeof Evaluation.prototype[name] === 'function'
  //   );
  //
  //   this.PARAMETERS = methods
  //     .map((methodName) => {
  //       const method = Evaluation.prototype[methodName];
  //       const parameter = Reflect.getMetadata('Parameter', method);
  //       return new ParamEvaluation(method, parameter);
  //     })
  //     .filter((pe) => pe.parameter !== null);
  // }

  public static evaluateState(state: BoardState): number {
    let score = state.interpolatedScore();
    return state.sideToPlay === Side.WHITE ? score : -score;
  }

  // public static paramHeaders(deltas: boolean): string[] {
  //   const headers: string[] = [];
  //   for (const parameter of this.PARAMETERS) {
  //     if (parameter.parameter.absolute) {
  //       if (!deltas) {
  //         headers.push(parameter.parameter.name);
  //       }
  //     } else if (parameter.parameter.signed) {
  //       headers.push(parameter.parameter.name);
  //     } else if (parameter.parameter.perSide) {
  //       headers.push(`${parameter.parameter.name}1`);
  //       headers.push(`${parameter.parameter.name}2`);
  //     }
  //   }
  //   return deltas
  //     ? headers.map((name) => `${name}-D`)
  //     : headers;
  // }

  // public static getParams(state: BoardState): number[] {
  //   const params: number[] = [];
  //   try {
  //     for (const parameter of this.PARAMETERS) {
  //       if (parameter.parameter.absolute) {
  //         const value = parameter.method.call(null, state) as number;
  //         params.push(value);
  //       } else if (parameter.parameter.signed) {
  //         const value = parameter.method.call(null, state) as number;
  //         params.push(
  //           state.sideToPlay === Side.WHITE ? value : -value
  //         );
  //       } else if (parameter.parameter.perSide) {
  //         params.push(parameter.method.call(null, state, state.sideToPlay) as number);
  //         params.push(parameter.method.call(null, state, 1 - state.sideToPlay) as number);
  //       }
  //     }
  //   } catch (e) {
  //     throw new Error(e);
  //   }
  //   return params;
  // }

  // public static makeDeltas(params: number[], rootParams: number[]): number[] {
  //   const deltas: number[] = [];
  //   let index = 0;
  //   for (const parameter of this.PARAMETERS) {
  //     if (parameter.parameter.absolute) {
  //       index++;
  //     } else if (parameter.parameter.signed) {
  //       const value = -(rootParams[index] + params[index]);
  //       deltas.push(value);
  //       index++;
  //     } else if (parameter.parameter.perSide) {
  //       deltas.push(params[index + 1] - rootParams[index]);
  //       deltas.push(params[index] - rootParams[index + 1]);
  //       index += 2;
  //     }
  //   }
  //   return deltas;
  // }

  // @Parameter({ name: 'move count', absolute: true })
  // public static moveCount(state: BoardState): number {
  //   return state.fullMoveNormalized;
  // }
  //
  // @Parameter({ name: 'pieces', perSide: true })
  // public static countOfPieces(state: BoardState, side: number): number {
  //   return Long.bitCount(state.allPieces(side));
  // }
  //
  // @Parameter({ name: 'pawn count', perSide: true })
  // public static pawnCount(state: BoardState, side: number): number {
  //   return Long.bitCount(state.bitboardOf(side, PieceType.PAWN));
  // }
  //
  // @Parameter({ name: 'bishop count', perSide: true })
  // public static bishopCount(state: BoardState, side: number): number {
  //   return Long.bitCount(state.bitboardOf(side, PieceType.BISHOP));
  // }
  //
  // @Parameter({ name: 'queen count', perSide: true })
  // public static queenCount(state: BoardState, side: number): number {
  //   return Long.bitCount(state.bitboardOf(side, PieceType.QUEEN));
  // }
}
