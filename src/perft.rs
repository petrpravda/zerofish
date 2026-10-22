use crate::board::BoardState;

pub fn perft(state: &BoardState, depth: u32) -> u64 {
    let moves = state.generate_legal_moves();
    if depth <= 1 {
        return moves.len() as u64;
    }
    moves.iter().map(|mv| perft(&state.do_move(mv), depth - 1)).sum()
}

/// Per-move node counts followed by the total, as printed by `go perft`.
pub fn perft_string(state: &BoardState, depth: u32) -> String {
    let mut total = 0;
    let lines: Vec<String> = state
        .generate_legal_moves()
        .iter()
        .map(|mv| {
            let count = if depth > 1 { perft(&state.do_move(mv), depth - 1) } else { 1 };
            total += count;
            format!("{mv}: {count}")
        })
        .collect();
    format!("{}\n\nNodes searched: {total}", lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::START_POS;

    fn fen(text: &str) -> BoardState {
        BoardState::from_fen(text).unwrap()
    }

    // http://www.talkchess.com/forum3/viewtopic.php?t=47318
    const TRICKY_PERFTS: &str = "
        8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1 perft 6 = 824064
        8/8/1k6/8/2pP4/8/5BK1/8 b - d3 0 1 perft 6 = 824064
        8/5k2/8/2Pp4/2B5/1K6/8/8 w - d6 0 1 perft 6 = 1440467
        8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1 perft 6 = 1440467
        5k2/8/8/8/8/8/8/4K2R w K - 0 1 perft 6 = 661072
        4k2r/8/8/8/8/8/8/5K2 b k - 0 1 perft 6 = 661072
        3k4/8/8/8/8/8/8/R3K3 w Q - 0 1 perft 6 = 803711
        r3k3/8/8/8/8/8/8/3K4 b q - 0 1 perft 6 = 803711
        r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1 perft 4 = 1274206
        r3k2r/7b/8/8/8/8/1B4BQ/R3K2R b KQkq - 0 1 perft 4 = 1274206
        r3k2r/8/5Q2/8/8/3q4/8/R3K2R w KQkq - 0 1 perft 4 = 1720476
        r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1 perft 4 = 1720476
        2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1 perft 6 = 3821001
        3K4/8/8/8/8/8/4p3/2k2R2 b - - 0 1 perft 6 = 3821001
        5K2/8/1Q6/2N5/8/1p2k3/8/8 w - - 0 1 perft 5 = 1004658
        8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1 perft 5 = 1004658
        4k3/1P6/8/8/8/8/K7/8 w - - 0 1 perft 6 = 217342
        8/k7/8/8/8/8/1p6/4K3 b - - 0 1 perft 6 = 217342
        8/P1k5/K7/8/8/8/8/8 w - - 0 1 perft 6 = 92683
        8/8/8/8/8/k7/p1K5/8 b - - 0 1 perft 6 = 92683
        8/k1P5/8/1K6/8/8/8/8 w - - 0 1 perft 7 = 567584
        8/8/8/8/1k6/8/K1p5/8 b - - 0 1 perft 7 = 567584
        8/5k2/8/5N2/5Q2/2K5/8/8 w - - 0 1 perft 4 = 23527
        8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1 perft 4 = 23527
    ";

    #[test]
    fn tricky_perfts() {
        for line in TRICKY_PERFTS.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let (position, rest) = line.split_once(" perft ").unwrap();
            let (depth, count) = rest.split_once(" = ").unwrap();
            assert_eq!(perft(&fen(position), depth.parse().unwrap()), count.parse::<u64>().unwrap(), "{line}");
        }
    }

    #[test]
    fn start_position() {
        let state = fen(START_POS);
        assert_eq!(perft(&state, 1), 20);
        assert_eq!(perft(&state, 3), 8902);
        assert_eq!(perft(&state, 4), 197281);
        assert_eq!(perft(&state, 5), 4865609);
    }

    #[test]
    fn kiwipete() {
        let state = fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1");
        assert_eq!(perft(&state, 4), 4085603);
    }

    #[test]
    fn special_positions() {
        assert_eq!(perft(&fen("rnbqkbnr/pppppppp/8/8/P7/8/1PPPPPPP/RNBQKBNR b KQkq - 0 1"), 5), 5363555);
        assert_eq!(perft(&fen("4n2k/3P4/3K4/8/8/8/8/8 w - - 0 1"), 1), 10);
        assert_eq!(perft(&fen("8/8/8/8/8/8/2kp4/K3N3 b - - 0 1"), 1), 8);
    }

    #[test]
    fn perft_string_lists_moves() {
        let text = perft_string(&fen(START_POS), 2);
        assert!(text.contains("e2e4: 20"));
        assert!(text.ends_with("Nodes searched: 400"));
    }
}
