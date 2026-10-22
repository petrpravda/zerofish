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
    if castling_pieces_kingside_mask(WHITE) & state.movements == 0 {
        rights.push('K');
    }
    if castling_pieces_queenside_mask(WHITE) & state.movements == 0 {
        rights.push('Q');
    }
    if castling_pieces_kingside_mask(BLACK) & state.movements == 0 {
        rights.push('k');
    }
    if castling_pieces_queenside_mask(BLACK) & state.movements == 0 {
        rights.push('q');
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

    Ok(BoardState::new(&items, side, movements, en_passant, half_move_clock, full_move_count))
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
}
