use crate::board_state::BoardState;

pub struct Perft {

}

impl Perft {
    pub fn perft(mut state: BoardState, depth: u16) -> u64 {
    let moves = state.generate_legal_moves();

    if depth == 1 {
        return moves.len() as u64;
    }

    return moves.moves.iter()
        .map(|m| state.do_move(m))
        .map(|new_state| Perft::perft(new_state, depth - 1))
        .reduce(|a, b| a + b)
        .unwrap();
    }

    // public static long perft2(BoardState state, int depth) {
    // MoveList moves = state.generateLegalMoves();
    //
    // if (depth == 0) {
    // return 1;
    // }
    // AtomicLong nodes = new AtomicLong();
    // moves
    // //.stream()
    // .forEach(move -> {
    // BoardState newBoardState = state.do_move(move);
    // if (depth >= 1) {
    // long c1 = perft2(newBoardState, depth - 1);
    // nodes.addAndGet(c1);
    // }
    // });
    //
    // return nodes.get();
    // }
    //
    // public static String perftString(BoardState state, int depth) {
    // MoveList moveList = state.generateLegalMoves();
    // AtomicLong nodes = new AtomicLong();
    //
    // List<String> list = moveList.stream()
    // .map(move -> {
    // long count = 0;
    //
    // if (depth > 1) {
    // BoardState newBoardState = state.do_move(move);
    // count += perft(newBoardState, depth - 1);
    // // board.undo_move(move, moveResult);
    // } else {
    // count += 1;
    // }
    //
    // nodes.addAndGet(count);
    // return String.format("%s: %d", move, count);
    // }).toList();
    //
    // String tableData = String.join("\n", list);
    // return String.format("%s\n\nNodes searched: %d", tableData, nodes.get());
    // }
    //
    // public static void main(String[] args) {
    // //        Board board = new Board(fromFen(START_POS));
    // //Board board = new Board(fromFen("r6r/3k4/8/8/3Q4/3q4/8/3RK2R b K - 3 2"));
    // BoardState state = fromFen("r6r/3k4/8/8/3Q4/3q4/8/3RK2R b K - 3 2");
    // //System.out.println(perftString(board, 5));
    // System.out.println(perft(state, 5));
    // }

}

#[cfg(test)]
mod tests {
    use crate::bitboard::Bitboard;
    use crate::fen::{from_fen_default, START_POS};
    use crate::perft::Perft;

    #[test]
    fn from_fen_startpos_depth_2() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS,  &bitboard);
        let count = Perft::perft(state, 2);
        assert_eq!(400, count);
    }

    #[test]
    fn from_fen_startpos_depth_3() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS,  &bitboard);
        let count = Perft::perft(state, 3);
        assert_eq!(8902, count);
    }


//     @Test
//     void simplePerft3() {
// BoardState state = fromFen(START_POS);
// assertEquals(8902, perft(state, 3));
// }
//
//     @Test
//     void simplePerft4() {
// BoardState state = fromFen(START_POS);
// assertEquals(197281, perft(state, 4));
// }
//
//     @Test
//     void simplePerft5() {
// BoardState board = fromFen(START_POS);
// assertEquals(4865609, perft(board, 5));
// }
//
//     @Test
//     void simplePerft5b() {
// BoardState board = fromFen("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1");
// assertEquals(5363555, perft(board, 5));
// }

}
