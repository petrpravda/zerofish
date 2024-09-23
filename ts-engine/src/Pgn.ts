import {BoardState} from './BoardState';
import {Move} from './Move';
import {Side} from './Side';
import {PieceType} from './PieceType';
import {Square} from './Square';
import {MoveList} from './MoveList';

export class Pgn {
  /**
   * Converts a series of UCI moves into PGN (or SAN) notation based on the initial board state (startFen).
   *
   * @param startFen The FEN string representing the initial board state.
   * @param ucis An array of UCI moves (e.g., ['e2e4', 'e7e5']).
   * @returns An array of PGN moves (e.g., ['e4', 'e5']).
   */
  public static uciToPgn(startFen: string, ucis: string[]): string[] {
    let board = BoardState.fromFen(startFen);
    const result: string[] = [];

    for (const uci of ucis) {
      const san = Pgn.oneUciToSan(uci, board);
      const move = Move.fromUciString(uci, board);
      result.push(san);
      board = board.doMove(move);
    }

    return result;
  }


  /**
   * Parses a SAN (Standard Algebraic Notation) move into a corresponding UCI move.
   *
   * @param san The SAN move string (e.g., "Nf3", "O-O", "a4").
   * @param state The current board state.
   * @returns An Optional move that matches the given SAN notation.
   */
  public static parseOneSan(san: string, state: BoardState): Move | undefined {
    let checkingMove = san.endsWith('+');
    let checkmatingMove = san.endsWith('#');
    if (checkingMove || checkmatingMove) {
      san = san.substring(0, san.length - 1);
    }

    let destination: string;
    let piece: PieceType;
    let promotionPiece: PieceType | undefined = undefined;
    let fromFile: string | undefined = undefined;
    let fromRank: string | undefined = undefined;

    // Handle castling moves
    if (san === "O-O") {
      destination = state.sideToPlay === Side.WHITE ? "g1" : "g8";
      piece = PieceType.KING;
    } else if (san === "O-O-O") {
      destination = state.sideToPlay === Side.WHITE ? "c1" : "c8";
      piece = PieceType.KING;
    } else {
      // Handle promotion move
      if (san.charAt(san.length - 2) === '=') {
        promotionPiece = PieceType.fromSan(san.charAt(san.length - 1));
        san = san.substring(0, san.length - 2);
      }

      const pieceTypeOptional = PieceType.fromSan(san.charAt(0));
      piece = pieceTypeOptional ?? PieceType.PAWN;

      let sanWithoutPieceType = pieceTypeOptional ? san.substring(1) : san;

      let takingMoveFlagIndex = sanWithoutPieceType.indexOf('x');
      let source = "";
      if (takingMoveFlagIndex !== -1) {
        source = sanWithoutPieceType.substring(0, takingMoveFlagIndex);
        destination = sanWithoutPieceType.substring(takingMoveFlagIndex + 1);
      } else {
        let sourceLength = sanWithoutPieceType.length - 2;
        if (sourceLength < 0) {
          throw new Error(`Invalid SAN move: ${san}`);
        }
        source = sanWithoutPieceType.substring(0, sourceLength);
        destination = sanWithoutPieceType.substring(sourceLength);
      }

      switch (source.length) {
        case 0:
          break;
        case 1:
          let fromChar = source.charAt(0);
          if (/\d/.test(fromChar)) {
            fromRank = fromChar;
          } else {
            fromFile = fromChar;
          }
          break;
        case 2:
          fromFile = source.charAt(0);
          fromRank = source.charAt(1);
          break;
        default:
          throw new Error(`Unimplemented source format: ${san}`);
      }
    }

    // Convert destination to a number using Square helper
    const destinationNumber = Square.getSquareFromName(destination);

    // Filter and match legal moves
    const uciMoves = state.generateLegalMoves();
    const matchingMoves = uciMoves.filter((move: Move) => {
      const validDestination = move.to() === destinationNumber;
      const validPiece = move.isPromotion() ? move.getPieceType() === promotionPiece :
        state.pieceTypeAt(move.from()) === piece;
      const validFile = fromFile === undefined || Square.getFile(move.from()) === fromFile;
      const validRank = fromRank === undefined || Square.getRank(move.from()) === fromRank;
      return validDestination && validPiece && validFile && validRank;
    });

    if (matchingMoves.length > 1) {
      throw new Error(`Ambiguous move: ${san}, possible moves: ${matchingMoves}`);
    }

    return matchingMoves.length === 1 ? matchingMoves[0] : undefined;
  }


  // public static oneUciToSan(uciMove: string, state: BoardState): string {
  //   const uciMoves: MoveList = state.generateLegalMoves();
  //
  //   // Try to find the exact move by its UCI string
  //   const moveCheating = uciMoves.find(move => move.uci() === uciMove);
  //
  //   const piece = moveCheating ? moveCheating.getPieceType() : null;
  //   const uciDestination = uciMove.substring(2, 4);
  //   const sourceFile = uciMove.charAt(0);
  //   const sourceRank = uciMove.charAt(1);
  //
  //   // Step 1: Find all moves going to the same destination and with the same piece type
  //   let matchingMoves = uciMoves
  //     .filter(move => Square.getName(move.to()) === uciDestination)
  //     .filter(move => piece === null || piece === move.getPieceType());
  //
  //   if (matchingMoves.length === 1) {
  //     return this.moveToPgn(matchingMoves[0], false, false, state);
  //   }
  //
  //   // Step 2: Narrow down using the source file
  //   matchingMoves = matchingMoves
  //     .filter(move => Square.getFile(move.from()) === sourceFile);
  //
  //   if (matchingMoves.length === 1) {
  //     return this.moveToPgn(matchingMoves[0], true, false, state);
  //   }
  //
  //   // Step 3: Narrow down using the source rank
  //   matchingMoves = uciMoves
  //     .filter(move => Square.getName(move.to()) === uciDestination)
  //     .filter(move => piece === null || piece === move.getPieceType())
  //     .filter(move => Square.getRank(move.from()) === sourceRank);
  //
  //   if (matchingMoves.length === 1) {
  //     return this.moveToPgn(matchingMoves[0], false, true, state);
  //   }
  //
  //   // Step 4: Finally, check both source file and rank
  //   matchingMoves = matchingMoves
  //     .filter(move => Square.getFile(move.from()) === sourceFile)
  //     .filter(move => Square.getRank(move.from()) === sourceRank);
  //
  //   if (matchingMoves.length === 1) {
  //     return this.moveToPgn(matchingMoves[0], true, true, state);
  //   }
  //
  //   if (matchingMoves.length > 1) {
  //     throw new Error(`Ambiguous possible moves: ${matchingMoves} for ${uciMove}`);
  //   } else {
  //     throw new Error(`Move ${uciMove} not found`);
  //   }
  // }

  public static oneUciToSan(uciMove: string, state: BoardState): string {
    const uciMoves: MoveList = state.generateLegalMoves();

    // Step 1: Try to find the exact move by its UCI string
    const moveFromLegalOptional = uciMoves.find(move => move.uci() === uciMove);

    if (!moveFromLegalOptional) {
      throw new Error(`Invalid move ${uciMove} for ${state.toFen()}`);
    }

    const moveFromLegal = moveFromLegalOptional;
    const piece = state.pieceTypeAt(moveFromLegal.from());
    const promotionPiece = moveFromLegal.isPromotion() ? moveFromLegal.getPieceType() : null;
    const uciDestination = uciMove.substring(2, 4);
    const sourceFile = uciMove.charAt(0);
    const sourceRank = uciMove.charAt(1);

    // Step 2: Find all moves going to the same destination and with the same piece type or promotion
    let matchingMoves = uciMoves
      .filter(move => Square.getName(move.to()) === uciDestination)
      .filter(move => promotionPiece === null || promotionPiece === move.getPieceType())
      .filter(move => piece === state.pieceTypeAt(move.from()));

    // If there's only one match, return the move in PGN format
    if (matchingMoves.length === 1) {
      return this.moveToPgn(matchingMoves[0], false, false, state);
    }

    // Step 3: Narrow down using the source file
    matchingMoves = uciMoves
      .filter(move => Square.getName(move.to()) === uciDestination)
      .filter(move => promotionPiece === null || promotionPiece === move.getPieceType())
      .filter(move => piece === state.pieceTypeAt(move.from()))
      .filter(move => Square.getFile(move.from()) === sourceFile);

    if (matchingMoves.length === 1) {
      return this.moveToPgn(matchingMoves[0], true, false, state);
    }

    // Step 4: Narrow down using the source rank
    matchingMoves = uciMoves
      .filter(move => Square.getName(move.to()) === uciDestination)
      .filter(move => promotionPiece === null || promotionPiece === move.getPieceType())
      .filter(move => piece === state.pieceTypeAt(move.from()))
      .filter(move => Square.getRank(move.from()) === sourceRank);

    if (matchingMoves.length === 1) {
      return this.moveToPgn(matchingMoves[0], false, true, state);
    }

    // Step 5: Finally, check both source file and rank
    matchingMoves = matchingMoves
      .filter(move => Square.getFile(move.from()) === sourceFile)
      .filter(move => Square.getRank(move.from()) === sourceRank);

    if (matchingMoves.length === 1) {
      return this.moveToPgn(matchingMoves[0], true, true, state);
    }

    // If there are still multiple matches, throw an ambiguity error
    if (matchingMoves.length > 1) {
      const movesString = matchingMoves.map(m => m.uci()).join(' ');
      throw new Error(`Ambiguous possible moves: ${movesString} for ${uciMove} for position ${state.toFen()}`);
    } else {
      // If no move matches, throw a not found error
      throw new Error(`Move ${uciMove} not found`);
    }
  }

  // Helper method to convert a move to PGN format
  private static moveToPgn(move: Move, fileNeeded: boolean, rankNeeded: boolean, state: BoardState): string {
    const destination = Square.getName(move.to());
    const pieceType = state.pieceTypeAt(move.from());
    // const isPawn = pieceType === PieceType.PAWN;
    let pieceIdentification: string | undefined = '';
    let halfResult = '';
    let capturing = false;

    // Check if the move involves a capture
    if (move.isCapture()) {
      capturing = true;
    }

    // Identify the piece type for PGN (except for pawns)
    if (pieceType !== PieceType.PAWN) {
      pieceIdentification = PieceType.toSan(pieceType);
    }

    // Handle castling moves
    const moveString = move.uci();
    if (pieceType === PieceType.KING && (moveString === 'e1g1' || moveString === 'e8g8')) {
      halfResult = 'O-O';
    } else if (pieceType === PieceType.KING && (moveString === 'e1c1' || moveString === 'e8c8')) {
      halfResult = 'O-O-O';
    } else {
      // Handle normal moves
      if (pieceType === PieceType.PAWN) {
        // Pawn move
        if (capturing) {
          halfResult = Square.getFile(move.from()) + 'x' + destination; // Example: exd5
        } else {
          halfResult = destination; // Example: e4
        }
      } else {
        // Non-pawn move
        halfResult = pieceIdentification!;

        if (fileNeeded) {
          halfResult += Square.getFile(move.from());
        }
        if (rankNeeded) {
          halfResult += Square.getRank(move.from());
        }

        if (capturing) {
          halfResult += 'x';
        }

        halfResult += destination;
      }

      // Handle pawn promotion (e.g., e8=Q)
      if (move.isPromotion()) {
        const promotionPiece = PieceType.toSan(move.getPieceType());
        halfResult += '=' + promotionPiece;
      }
    }

    // Apply the move to the board state
    const stateAfterMove = state.doMove(move);

    // Check if the move results in check or checkmate
    if (stateAfterMove.isInCheckMate()) {
      halfResult += '#'; // Checkmate
    } else if (stateAfterMove.isKingAttacked()) {
      halfResult += '+'; // Check
    }

    return halfResult;
  }

}
