use crate::bitboard::*;
use crate::board::BoardState;
use crate::types::*;

pub const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn to_fen(state: &BoardState) -> String {
    let mut fen = String::new();
    for rank in (0..8).rev() {
        let mut empty = 0;
        for file in 0..8 {
            let piece = state.piece_at(rank * 8 + file);
            if piece == NONE {
                empty += 1;
            } else {
                if empty > 0 {
                    fen.push_str(&empty.to_string());
                    empty = 0;
                }
                fen.push(piece_notation(piece));
            }
        }
        if empty > 0 {
            fen.push_str(&empty.to_string());
        }
        if rank > 0 {
            fen.push('/');
        }
    }
    fen.push_str(if state.side_to_play() == WHITE { " w" } else { " b" });

    let mut rights = String::new();
    for side in [WHITE, BLACK] {
        for kingside in [true, false] {
            if state.has_castling_right(side, kingside) {
                rights.push(if state.is_chess960() {
                    (if side == WHITE { b'A' } else { b'a' } + (state.castling_rook_square(side, kingside) & 7) as u8) as char
                } else {
                    match (side, kingside) {
                        (WHITE, true) => 'K',
                        (WHITE, false) => 'Q',
                        (_, true) => 'k',
                        (_, false) => 'q',
                    }
                });
            }
        }
    }
    fen.push(' ');
    fen.push_str(if rights.is_empty() { "-" } else { &rights });

    fen.push(' ');
    if state.en_passant != 0 {
        fen.push_str(&square_name(lsb(state.en_passant)));
    } else {
        fen.push('-');
    }
    fen.push_str(&format!(" {} {}", state.half_move_clock, state.full_move_normalized / 2 + 1));
    fen
}

pub fn from_fen(fen: &str) -> Result<BoardState, String> {
    from_fen_with_chess960(fen, false)
}

pub fn from_fen_with_chess960(fen: &str, chess960: bool) -> Result<BoardState, String> {
    let parts: Vec<&str> = fen.split_whitespace().collect();
    if parts.len() < 4 {
        return Err(format!("FEN needs at least 4 fields: {fen}"));
    }

    let mut items = [NONE; 64];
    let ranks: Vec<&str> = parts[0].split('/').collect();
    if ranks.len() != 8 {
        return Err(format!("FEN needs 8 ranks: {}", parts[0]));
    }
    for (index, rank_text) in ranks.iter().enumerate() {
        let rank = 7 - index;
        let mut file = 0;
        for c in rank_text.chars() {
            if let Some(skip) = c.to_digit(10).filter(|d| (1..=8).contains(d)) {
                file += skip as usize;
                continue;
            }
            let piece = match c {
                'P' => WHITE_PAWN,
                'N' => WHITE_KNIGHT,
                'B' => WHITE_BISHOP,
                'R' => WHITE_ROOK,
                'Q' => WHITE_QUEEN,
                'K' => WHITE_KING,
                'p' => BLACK_PAWN,
                'n' => BLACK_KNIGHT,
                'b' => BLACK_BISHOP,
                'r' => BLACK_ROOK,
                'q' => BLACK_QUEEN,
                'k' => BLACK_KING,
                _ => return Err(format!("Character \"{c}\" not known.")),
            };
            if file >= 8 {
                return Err(format!("Rank too long: {rank_text}"));
            }
            items[rank * 8 + file] = piece;
            file += 1;
        }
        if file != 8 {
            return Err(format!("Rank must have 8 squares: {rank_text}"));
        }
    }
    for king in [WHITE_KING, BLACK_KING] {
        if items.iter().filter(|&&p| p == king).count() != 1 {
            return Err("FEN must have exactly one king per side".to_string());
        }
    }

    let side = match parts[1] {
        "w" | "W" => WHITE,
        "b" | "B" => BLACK,
        other => return Err(format!("Unknown side to move: {other}")),
    };

    let castling = parts[2];
    let chess960 = chess960 || castling.bytes().any(|letter| matches!(letter, b'A'..=b'H' | b'a'..=b'h'));
    let mut movements = 0u64;
    let rook_square = |mask: u64| lsb(mask);
    if !castling.contains('K') || items[WHITE_KING_INITIAL_SQUARE] != WHITE_KING || items[rook_square(WHITE_KINGS_ROOK_MASK)] != WHITE_ROOK {
        movements |= WHITE_KINGS_ROOK_MASK;
    }
    if !castling.contains('Q') || items[WHITE_KING_INITIAL_SQUARE] != WHITE_KING || items[rook_square(WHITE_QUEENS_ROOK_MASK)] != WHITE_ROOK {
        movements |= WHITE_QUEENS_ROOK_MASK;
    }
    if !castling.contains('k') || items[BLACK_KING_INITIAL_SQUARE] != BLACK_KING || items[rook_square(BLACK_KINGS_ROOK_MASK)] != BLACK_ROOK {
        movements |= BLACK_KINGS_ROOK_MASK;
    }
    if !castling.contains('q') || items[BLACK_KING_INITIAL_SQUARE] != BLACK_KING || items[rook_square(BLACK_QUEENS_ROOK_MASK)] != BLACK_ROOK {
        movements |= BLACK_QUEENS_ROOK_MASK;
    }

    let en_passant = if parts[3].len() < 2 {
        0
    } else {
        1u64 << square_from_name(parts[3]).ok_or_else(|| format!("Bad en passant square: {}", parts[3]))?
    };
    let half_move_clock = parts.get(4).map_or(Ok(0), |s| s.parse::<i32>()).map_err(|e| format!("Bad halfmove clock: {e}"))?;
    let full_move_count = parts.get(5).map_or(Ok(1), |s| s.parse::<i32>()).map_err(|e| format!("Bad fullmove number: {e}"))?;

    let mut state = BoardState::new(&items, side, if chess960 { 0 } else { movements }, en_passant, half_move_clock, full_move_count);
    if chess960 {
        state.configure_chess960_castling(chess960_castling(&items, castling)?);
    }
    Ok(state)
}

fn chess960_castling(items: &[usize; 64], flags: &str) -> Result<[u8; 6], String> {
    let mut squares = [64; 6];
    for side in [WHITE, BLACK] {
        for file in 0..8 {
            let square = side * 56 + file;
            if items[square] == make_piece(side, KING) {
                squares[4 + side] = square as u8;
            }
        }
    }
    for flag in flags.chars().filter(|&flag| flag != '-') {
        let side = if flag.is_ascii_uppercase() { WHITE } else { BLACK };
        let letter = flag.to_ascii_uppercase();
        if !matches!(letter, 'K' | 'Q' | 'A'..='H') {
            return Err(format!("Invalid Chess960 castling right: {flag}"));
        }
        let king = squares[4 + side] as usize;
        if king >= 64 {
            continue;
        }
        let rook = if letter == 'K' {
            ((king + 1)..(side * 56 + 8)).rev().find(|&square| items[square] == make_piece(side, ROOK))
        } else if letter == 'Q' {
            ((side * 56)..king).find(|&square| items[square] == make_piece(side, ROOK))
        } else {
            let square = side * 56 + (letter as u8 - b'A') as usize;
            (items[square] == make_piece(side, ROOK)).then_some(square)
        };
        if let Some(rook) = rook {
            let index = side * 2 + usize::from(rook < king);
            if squares[index] < 64 && squares[index] as usize != rook {
                return Err(format!("Conflicting Chess960 castling rights: {flags}"));
            }
            squares[index] = rook as u8;
        }
    }
    Ok(squares)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        for fen in [
            START_POS,
            "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 100 1",
            "rnbqkbnr/p2ppppp/2p5/Pp6/8/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 3",
            "8/5bk1/8/2Pp4/8/1K6/8/8 w - d6 0 1",
        ] {
            assert_eq!(from_fen(fen).unwrap().to_fen(), fen);
        }
    }

    #[test]
    fn rejects_garbage() {
        assert!(from_fen("rnbqkbnr/pppppppp/8/8 w - -").is_err());
        assert!(from_fen("8/8/8/8/8/8/8/8 w - - 0 1").is_err());
        assert!(from_fen("rnbqkbnr/ppppXppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").is_err());
    }

    #[test]
    fn chess960_shredder_and_xfen() {
        let fen = "rk1r4/8/8/8/8/8/8/RK1R4 w DAda - 0 1";
        let shredder = from_fen(fen).unwrap();
        let xfen = from_fen_with_chess960(&fen.replace("DAda", "KQkq"), true).unwrap();
        assert!(shredder.is_chess960());
        assert_eq!(shredder.to_fen(), fen);
        assert_eq!(xfen.to_fen(), fen);
        assert_eq!(shredder.hash(), xfen.hash());
        assert_eq!(shredder.castling_rook_square(WHITE, true), 3);
        assert_eq!(shredder.castling_rook_square(WHITE, false), 0);
        assert_eq!(shredder.castling_rook_square(BLACK, true), 59);
        assert!(!from_fen(START_POS).unwrap().is_chess960());
    }

    #[test]
    fn chess960_castling_obstructions_and_checks() {
        for fen in [
            "k3r3/8/8/8/8/8/8/4K2R w H - 0 1",
            "k4r2/8/8/8/8/8/8/4K2R w H - 0 1",
            "k5r1/8/8/8/8/8/8/4K2R w H - 0 1",
            "7k/8/8/8/8/8/8/rRK5 w B - 0 1",
            "7k/8/8/8/8/8/8/1rR1K3 w C - 0 1",
            "k7/8/8/8/8/8/8/4KBR1 w G - 0 1",
            "k7/8/8/8/8/8/8/RN2K3 w A - 0 1",
        ] {
            assert!(!from_fen(fen).unwrap().generate_legal_moves().iter().any(|mv| mv.is_castling()), "{fen}");
        }
        let state = from_fen("k7/8/8/8/8/8/1r6/R2K4 w A - 0 1").unwrap();
        assert_eq!(state.do_move_uci("d1a1").unwrap().to_fen(), "k7/8/8/8/8/8/1r6/2KR4 b - - 1 1");
        let black = from_fen("1r1k2r1/8/8/8/8/8/8/7K b gb - 0 1").unwrap();
        assert_eq!(black.do_move_uci("d8g8").unwrap().to_fen(), "1r3rk1/8/8/8/8/8/8/7K w - - 1 2");
        assert!(black.find_legal_move("d8b8").unwrap().is_castling());
    }

    #[test]
    fn chess960_rights_and_rook_identity() {
        let state = from_fen("4k3/8/8/8/8/8/8/1R1K2R1 w GB - 0 1").unwrap();
        let rook_moved = state.do_move_uci("g1g2").unwrap();
        assert!(!rook_moved.has_castling_right(WHITE, true));
        assert!(rook_moved.has_castling_right(WHITE, false));
        let returned = rook_moved.do_move_uci("e8e7").unwrap().do_move_uci("g2g1").unwrap();
        assert!(!returned.has_castling_right(WHITE, true));
        let king_moved = state.do_move_uci("d1d2").unwrap();
        assert!(!king_moved.has_castling_right(WHITE, true));
        assert!(!king_moved.has_castling_right(WHITE, false));
        let captured = from_fen("k5r1/8/8/8/8/8/8/1R1K2R1 b GB - 0 1").unwrap().do_move_uci("g8g1").unwrap();
        assert!(!captured.has_castling_right(WHITE, true));
        assert!(captured.has_castling_right(WHITE, false));
        assert!(state.do_null_move().has_castling_right(WHITE, true));
        let inner = from_fen("4k3/8/8/8/8/8/8/1K1R3R w D - 0 1").unwrap();
        let outer = from_fen("4k3/8/8/8/8/8/8/1K1R3R w H - 0 1").unwrap();
        assert_ne!(inner.hash(), outer.hash());
        assert_ne!(inner.transposition_key(), outer.transposition_key());
        assert_eq!(outer.hash(), from_fen_with_chess960(&outer.to_fen().replace(" H ", " K "), true).unwrap().hash());
        assert!(from_fen("4k3/8/8/8/8/8/8/1K1R3R w DH - 0 1").is_err());
        assert!(from_fen_with_chess960("4k3/8/8/8/8/8/8/1K1R3R w Z - 0 1", true).is_err());
    }
}
