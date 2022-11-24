use crate::board_state::BoardState;

pub struct Perft {}

impl Perft {
    pub fn normalize_perft_res(perft_stats: &str) -> String {
        let mut sorted = perft_stats.lines().collect::<Vec<&str>>();
        sorted.sort();
        let result = sorted.join("\n");
        result
    }

    pub fn perft(state: &BoardState, depth: u16) -> u64 {
        let moves = state.generate_legal_moves();

        if depth == 0 {
            return 1;
        } else if depth == 1 {
            return moves.len() as u64;
        }

        return moves.moves.iter()
            .map(|m| state.do_move(m))
            .map(|mut new_state| Perft::perft(&mut new_state, depth - 1))
            .reduce(|a, b| a + b)
            .unwrap();
    }

    pub fn perft_sf_string(mut state: BoardState, depth: u16) -> (String, u64) {
        let mut result = String::new();
        let moves = state.generate_legal_moves();
        let mut total: u64 = 0;

        for mowe in moves.moves {
            let mut moved_state = state.do_move(&mowe);
            let count = Perft::perft(&mut moved_state, depth - 1);
            result.push_str(&format!("{}: {}\n", &mowe.uci(), count));
            total += count;
        }

        (result, total)
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
    fn from_fen_startpos_depth_1() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS, &bitboard);
        let count = Perft::perft(&state, 1);
        let result = Perft::perft_sf_string(state, 1);
        println!("{}", result.0);
        let expected = r#"a2a3: 1
b2b3: 1
c2c3: 1
d2d3: 1
e2e3: 1
f2f3: 1
g2g3: 1
h2h3: 1
a2a4: 1
b2b4: 1
c2c4: 1
d2d4: 1
e2e4: 1
f2f4: 1
g2g4: 1
h2h4: 1
b1a3: 1
b1c3: 1
g1f3: 1
g1h3: 1
"#;
        assert_eq!(Perft::normalize_perft_res(&result.0), Perft::normalize_perft_res(expected));
        assert_eq!(count, 20);
    }

    #[test]
    fn from_fen_startpos_depth_1_n() {
        let bitboard = Bitboard::new();
        let state = from_fen_default("rnbqkbnr/pppppppp/8/8/8/2N5/PPPPPPPP/R1BQKBNR b KQkq - 1 1", &bitboard);
        let count = Perft::perft(&state, 1);
        let result = Perft::perft_sf_string(state, 1);
        println!("{}", result.0);
        let expected = r#"a7a6: 1
b7b6: 1
c7c6: 1
d7d6: 1
e7e6: 1
f7f6: 1
g7g6: 1
h7h6: 1
a7a5: 1
b7b5: 1
c7c5: 1
d7d5: 1
e7e5: 1
f7f5: 1
g7g5: 1
h7h5: 1
b8a6: 1
b8c6: 1
g8f6: 1
g8h6: 1
"#;
        assert_eq!(Perft::normalize_perft_res(&result.0), Perft::normalize_perft_res(expected));
        assert_eq!(count, 20);
    }

    #[test]
    fn from_fen_startpos_depth_2() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS, &bitboard);
        let (result, count) = Perft::perft_sf_string(state, 2);
        println!("{}", result);
        assert_eq!(400, count);
    }

    #[test]
    fn from_fen_startpos_depth_2_n() {
        let bitboard = Bitboard::new();
        let mut state = from_fen_default("rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR b KQkq - 1 1", &bitboard);
        let (result, count) = Perft::perft_sf_string(state, 2);
        println!("{}", result);
        assert_eq!(400, count);
    }

    #[test]
    fn from_fen_startpos_depth_3() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS, &bitboard);
        let (perft_sf, count) = Perft::perft_sf_string(state, 3);
        println!("{}", perft_sf);
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
