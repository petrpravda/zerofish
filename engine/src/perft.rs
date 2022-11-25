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

        match moves.moves.len() {
            0 => 0,
            1 => {
                let new_state = state.do_move(moves.moves.get(0).unwrap());
                Perft::perft(&new_state, depth - 1)
            },
            _ => moves.moves.iter()
                .map(|m| state.do_move(m))
                .map(|mut new_state| Perft::perft(&mut new_state, depth - 1))
                .reduce(|a, b| a + b)
                .unwrap()
        }
    }

    pub fn perft_sf_string(state: &BoardState, depth: u16) -> (String, u64) {
        let mut result = String::new();
        let moves = state.generate_legal_moves();
        let mut total: u64 = 0;

        for mowe in moves.moves {
            let moved_state = state.do_move(&mowe);
            let count = Perft::perft(&moved_state, depth - 1);
            // if mowe.uci().eq("b1c3") {
            //     println!("DEBUG {}", moved_state.generate_legal_moves());
            // }
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
        //let count = Perft::perft(&state, 1);
        let (result, count) = Perft::perft_sf_string(&state, 1);
        // println!("{}", result);
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
        assert_eq!(Perft::normalize_perft_res(&result), Perft::normalize_perft_res(expected));
        assert_eq!(count, 20);
    }

    #[test]
    fn from_fen_startpos_depth_1_n() {
        let bitboard = Bitboard::new();
        let state = from_fen_default("rnbqkbnr/pppppppp/8/8/8/2N5/PPPPPPPP/R1BQKBNR b KQkq - 1 1", &bitboard);
        //let count = Perft::perft(&state, 1);
        let (result, count) = Perft::perft_sf_string(&state, 1);
        // println!("{}", result);
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
        assert_eq!(Perft::normalize_perft_res(&result), Perft::normalize_perft_res(expected));
        assert_eq!(count, 20);
    }

    #[test]
    fn from_fen_startpos_depth_2() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS, &bitboard);
        let (result, count) = Perft::perft_sf_string(&state, 2);
        let expected = r#"a2a3: 20
b2b3: 20
c2c3: 20
d2d3: 20
e2e3: 20
f2f3: 20
g2g3: 20
h2h3: 20
a2a4: 20
b2b4: 20
c2c4: 20
d2d4: 20
e2e4: 20
f2f4: 20
g2g4: 20
h2h4: 20
b1a3: 20
b1c3: 20
g1f3: 20
g1h3: 20"#;
        // println!("{}", result);
        assert_eq!(Perft::normalize_perft_res(&result), Perft::normalize_perft_res(expected));
        assert_eq!(400, count);
    }

    #[test]
    fn from_fen_startpos_depth_2_p() {
        let bitboard = Bitboard::new();
        let state = from_fen_default("rnbqkbnr/pppppppp/8/8/8/P7/1PPPPPPP/RNBQKBNR b KQkq - 0 1", &bitboard);
        let (result, count) = Perft::perft_sf_string(&state, 1);
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
        assert_eq!(Perft::normalize_perft_res(&result), Perft::normalize_perft_res(expected));
//        println!("{}", result);
        assert_eq!(20, count);
    }

    #[test]
    fn from_fen_startpos_depth_2_n() {
        let bitboard = Bitboard::new();
        let state = from_fen_default("rnbqkbnr/pppppppp/8/8/8/N7/PPPPPPPP/R1BQKBNR b KQkq - 1 1", &bitboard);
        let (_result, count) = Perft::perft_sf_string(&state, 2);
        // println!("{}", result);
        assert_eq!(400, count);
    }

    #[test]
    fn from_fen_startpos_depth_3() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS, &bitboard);
        let (perft_sf, count) = Perft::perft_sf_string(&state, 3);
        let expected = r#"a2a3: 380
b2b3: 420
c2c3: 420
d2d3: 539
e2e3: 599
f2f3: 380
g2g3: 420
h2h3: 380
a2a4: 420
b2b4: 421
c2c4: 441
d2d4: 560
e2e4: 600
f2f4: 401
g2g4: 421
h2h4: 420
b1a3: 400
b1c3: 440
g1f3: 440
g1h3: 400"#;
        assert_eq!(Perft::normalize_perft_res(&perft_sf), Perft::normalize_perft_res(expected));
        assert_eq!(8902, count);
    }

    #[test]
    fn from_fen_startpos_depth_3_pp() {
        let bitboard = Bitboard::new();
        let state = from_fen_default("rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b KQkq - 0 1", &bitboard);
        let (perft_sf, count) = Perft::perft_sf_string(&state, 2);
        let expected = r#"a7a6: 28
b7b6: 28
c7c6: 28
d7d6: 28
e7e6: 28
f7f6: 28
g7g6: 28
h7h6: 28
a7a5: 28
b7b5: 28
c7c5: 29
d7d5: 27
e7e5: 29
f7f5: 28
g7g5: 27
h7h5: 28
b8a6: 28
b8c6: 28
g8f6: 28
g8h6: 28"#;
        assert_eq!(Perft::normalize_perft_res(&perft_sf), Perft::normalize_perft_res(expected));
        assert_eq!(560, count);
    }

    #[test]
    fn from_fen_startpos_depth_3_pp1() {
        let bitboard = Bitboard::new();
        let state = from_fen_default("rnbqkbnr/1ppppppp/8/p7/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 2", &bitboard);
        let (perft_sf, _count) = Perft::perft_sf_string(&state, 1);
        let expected = r#"a2a3: 1
b2b3: 1
c2c3: 1
e2e3: 1
f2f3: 1
g2g3: 1
h2h3: 1
d4d5: 1
a2a4: 1
b2b4: 1
c2c4: 1
e2e4: 1
f2f4: 1
g2g4: 1
h2h4: 1
b1d2: 1
b1a3: 1
b1c3: 1
g1f3: 1
g1h3: 1
c1d2: 1
c1e3: 1
c1f4: 1
c1g5: 1
c1h6: 1
d1d2: 1
d1d3: 1
e1d2: 1"#;
        assert_eq!(Perft::normalize_perft_res(&perft_sf), Perft::normalize_perft_res(expected));
    }

    #[test]
    fn from_fen_startpos_depth_4() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS, &bitboard);
        let (_perft_sf, count) = Perft::perft_sf_string(&state, 4);
        assert_eq!(197281, count);
    }

    #[test]
    fn from_fen_startpos_depth_5() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS, &bitboard);
        let (_perft_sf, count) = Perft::perft_sf_string(&state, 5);
        assert_eq!(4865609, count);
    }

    #[test]
    fn from_fen_startpos_depth_5_detail() {
        // position startpos moves d2d3 g8f6 e1d2 f6e4
        let bitboard = Bitboard::new();
        let state = from_fen_default("rnbqkb1r/pppppppp/8/8/4n3/3P4/PPPKPPPP/RNBQ1BNR w kq - 3 3", &bitboard);
        let (perft_sf, _count) = Perft::perft_sf_string(&state, 1);
        let expected = r#"d2e1: 1
d3e4: 1
d2e3: 1"#;
        assert_eq!(Perft::normalize_perft_res(&perft_sf), Perft::normalize_perft_res(expected));
    }

    // http://www.talkchess.com/forum3/viewtopic.php?t=47318
    const TRICKY_PERFTS: &str =
r#"avoid illegal en passant capture:
8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1 perft 6 = 824064
8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1 perft 6 = 824064
en passant capture checks opponent:
8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1 perft 6 = 1440467
8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1 perft 6 = 1440467
short castling gives check:
5k2/8/8/8/8/8/8/4K2R w K - 0 1 perft 6 = 661072
4k2r/8/8/8/8/8/8/5K2 b k - 0 1 perft 6 = 661072
long castling gives check:
3k4/8/8/8/8/8/8/R3K3 w Q - 0 1 perft 6 = 803711
r3k3/8/8/8/8/8/8/3K4 b q - 0 1 perft 6 = 803711
castling (including losing cr due to rook capture):
r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1 perft 4 = 1274206
r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1 perft 4 = 1274206
castling prevented:
r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1 perft 4 = 1720476
r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1 perft 4 = 1720476
promote out of check:
2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1 perft 6 = 3821001
3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1 perft 6 = 3821001
discovered check:
5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1 perft 5 = 1004658
8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1 perft 5 = 1004658
promote to give check:
4k3/1P6/8/8/8/8/K7/8 w - - 0 1 perft 6 = 217342
8/k7/8/8/8/8/1p6/4K3 b - - 0 1 perft 6 = 217342
underpromote to check:
8/P1k5/K7/8/8/8/8/8 w - - 0 1 perft 6 = 92683
8/8/8/8/8/k7/p1K5/8 b - - 0 1 perft 6 = 92683
self stalemate:
K1k5/8/P7/8/8/8/8/8 w - - 0 1 perft 6 = 2217
8/8/8/8/8/p7/8/k1K5 b - - 0 1 perft 6 = 2217
stalemate/checkmate:
8/k1P5/8/1K6/8/8/8/8 w - - 0 1 perft 7 = 567584
8/8/8/8/1k6/8/K1p5/8 b - - 0 1 perft 7 = 567584
double check:
8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1 perft 4 = 23527
8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1 perft 4 = 23527"#;

    struct TestPosition {
        fen: String,
        depth: u16,
        count: u64,
    }

    fn parse_line(line: &str) -> TestPosition {
        let splits = line.split_whitespace().collect::<Vec<&str>>();
        let fen = &splits[0..6].join(" ");
        let depth = splits.get(7).unwrap().parse::<u16>().unwrap();
        let count = splits.get(9).unwrap().parse::<u64>().unwrap();
        TestPosition {
            fen: String::from(fen),
            depth,
            count,
        }
    }

    fn test_tricky(_name: &str, line: &str, bitboard: &Bitboard) {
        let position = parse_line(line);
        let state = from_fen_default(&position.fen, bitboard);
        let (_perft_sf, count) = Perft::perft_sf_string(&state, position.depth);
        assert_eq!(position.count, count);
    }

    #[test]
    fn tricky_perfts() {
        let bitboard = Bitboard::new();
        TRICKY_PERFTS.lines().collect::<Vec<&str>>().chunks(3).for_each(|ch| {
            let (name, white, black) = (ch[0], ch[1], ch[2]);
            // println!("{}", name);
            test_tricky(name, white, &bitboard);
            test_tricky(name, black, &bitboard);
        });
    }

}
