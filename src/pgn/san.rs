//! Standard algebraic notation: strict parsing against legal moves and canonical formatting.

use crate::board::BoardState;
use crate::moves::Move;
use crate::types::*;

const PIECES: &str = "PNBRQK";

struct Notation {
    piece: usize,
    from_file: Option<char>,
    from_rank: Option<char>,
    capture: bool,
    destination: usize,
    promotion: Option<usize>,
}

fn piece_index(c: char) -> usize {
    PIECES.find(c).unwrap()
}

/// `[KQRBN]?[a-h]?[1-8]?x?[a-h][1-8](=[QRBN])?`, the check suffix already removed.
fn parse_notation(base: &str) -> Option<Notation> {
    let mut chars: Vec<char> = base.chars().collect();
    let mut promotion = None;
    if chars.len() >= 2 && chars[chars.len() - 2] == '=' {
        let piece = chars[chars.len() - 1];
        if !"QRBN".contains(piece) {
            return None;
        }
        promotion = Some(piece_index(piece));
        chars.truncate(chars.len() - 2);
    }
    if chars.len() < 2 {
        return None;
    }
    let destination = square_from_name(&chars[chars.len() - 2..].iter().collect::<String>())?;
    chars.truncate(chars.len() - 2);
    let capture = chars.last() == Some(&'x');
    if capture {
        chars.pop();
    }
    let mut rest = &chars[..];
    let mut piece = PAWN;
    if let Some(&first) = rest.first() {
        if "KQRBN".contains(first) {
            piece = piece_index(first);
            rest = &rest[1..];
        }
    }
    let mut from_file = None;
    let mut from_rank = None;
    if let Some(&c) = rest.first().filter(|c| ('a'..='h').contains(c)) {
        from_file = Some(c);
        rest = &rest[1..];
    }
    if let Some(&c) = rest.first().filter(|c| ('1'..='8').contains(c)) {
        from_rank = Some(c);
        rest = &rest[1..];
    }
    rest.is_empty().then_some(Notation { piece, from_file, from_rank, capture, destination, promotion })
}

pub fn parse(board: &BoardState, san: &str) -> Result<Move, String> {
    find(board, san)?.ok_or_else(|| format!("Illegal SAN: {san}"))
}

/// The legal move matching `san`, `None` when there is none or its check/mate suffix is wrong.
pub fn find(board: &BoardState, san: &str) -> Result<Option<Move>, String> {
    let token = san.replace('0', "O");
    let suffix = token.chars().last().filter(|c| *c == '+' || *c == '#');
    let base = &token[..token.len() - suffix.map_or(0, |_| 1)];
    let legal = board.generate_legal_moves();
    let matches: Vec<Move> = if base == "O-O" || base == "O-O-O" {
        let flags = if base == "O-O" { Move::OO } else { Move::OOO };
        legal.iter().filter(|mv| mv.flags() == flags).collect()
    } else {
        let n = parse_notation(base).ok_or_else(|| format!("Invalid SAN: {san}"))?;
        if n.piece == PAWN && (n.from_rank.is_some() || n.capture != n.from_file.is_some()) {
            return Err(format!("Invalid pawn SAN: {san}"));
        }
        legal
            .iter()
            .filter(|mv| !mv.is_castling())
            .filter(|mv| board.piece_type_at(mv.from()) == n.piece && mv.to() == n.destination)
            .filter(|mv| (mv.flags() & Move::CAPTURE != 0) == n.capture)
            .filter(|mv| match n.promotion {
                None => !mv.is_promotion(),
                Some(piece) => mv.is_promotion() && mv.piece_type() == piece,
            })
            .filter(|mv| n.from_file.is_none_or(|f| file_char(mv.from()) == f))
            .filter(|mv| n.from_rank.is_none_or(|r| rank_char(mv.from()) == r))
            .collect()
    };
    if matches.len() > 1 {
        return Err(format!("Ambiguous SAN: {san}"));
    }
    Ok(matches.first().copied().filter(|&mv| {
        let Some(suffix) = suffix else { return true };
        let child = board.do_move(mv);
        child.is_king_attacked() && (suffix != '#' || !child.has_legal_move())
    }))
}

pub fn format(board: &BoardState, mv: Move) -> Result<String, String> {
    let legal = board.generate_legal_moves();
    if !legal.contains(mv) {
        return Err(format!("Illegal move: {mv}"));
    }
    let mut notation = String::new();
    if mv.is_castling() {
        notation.push_str(if mv.flags() == Move::OO { "O-O" } else { "O-O-O" });
    } else {
        let piece = board.piece_type_at(mv.from());
        let capture = mv.flags() & Move::CAPTURE != 0;
        if piece != PAWN {
            notation.push(PIECES.as_bytes()[piece] as char);
            let others: Vec<Move> =
                legal.iter().filter(|o| !o.is_castling() && o.from() != mv.from() && o.to() == mv.to() && board.piece_type_at(o.from()) == piece).collect();
            if !others.is_empty() {
                let file_unique = others.iter().all(|o| o.from() & 7 != mv.from() & 7);
                let rank_unique = others.iter().all(|o| o.from() >> 3 != mv.from() >> 3);
                if file_unique || !rank_unique {
                    notation.push(file_char(mv.from()));
                }
                if !file_unique {
                    notation.push(rank_char(mv.from()));
                }
            }
        } else if capture {
            notation.push(file_char(mv.from()));
        }
        if capture {
            notation.push('x');
        }
        notation.push_str(&square_name(mv.to()));
        if mv.is_promotion() {
            notation.push('=');
            notation.push(PIECES.as_bytes()[mv.piece_type()] as char);
        }
    }
    let child = board.do_move(mv);
    if child.is_king_attacked() {
        notation.push(if child.has_legal_move() { '+' } else { '#' });
    }
    Ok(notation)
}

/// Plays SAN moves from `start`.
pub fn decode<S: AsRef<str>>(start: &BoardState, sans: &[S]) -> Result<Vec<Move>, String> {
    let mut board = start.clone();
    let mut moves = Vec::with_capacity(sans.len());
    for san in sans {
        let mv = parse(&board, san.as_ref())?;
        board = board.do_move(mv);
        moves.push(mv);
    }
    Ok(moves)
}

/// Formats UCI moves played from `start` as SAN.
pub fn encode<S: AsRef<str>>(start: &BoardState, ucis: &[S]) -> Result<Vec<String>, String> {
    let mut board = start.clone();
    let mut sans = Vec::with_capacity(ucis.len());
    for uci in ucis {
        let mv = board.find_legal_move(uci.as_ref()).ok_or_else(|| format!("Illegal move: {}", uci.as_ref()))?;
        sans.push(format(&board, mv)?);
        board = board.do_move(mv);
    }
    Ok(sans)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fen(text: &str) -> BoardState {
        BoardState::from_fen(text).unwrap()
    }

    fn round_trip(board: &BoardState, san: &str) -> String {
        format(board, parse(board, san).unwrap()).unwrap()
    }

    #[test]
    fn checks_capture_promotion_and_check_markers() {
        let start = BoardState::start_pos();
        for san in ["exe4", "e4+", "e4#", "e4=Q", "Nxf3", "e", "", "Pxe4"] {
            assert!(parse(&start, san).is_err(), "{san}");
        }
        let promotion = fen("7k/P7/8/8/8/8/8/7K w - - 0 1");
        assert!(parse(&promotion, "a8").is_err());
        assert_eq!(parse(&promotion, "a8=N").unwrap().uci(), "a7a8n");
    }

    #[test]
    fn disambiguates_only_legal_pieces() {
        let board = fen("4r2k/8/8/8/8/8/R3R3/4K3 w - - 0 1");
        assert_eq!(format(&board, board.find_legal_move("a2d2").unwrap()).unwrap(), "Rd2");
        let knights = fen("7k/8/8/8/8/8/8/KN3N2 w - - 0 1");
        assert!(parse(&knights, "Nd2").is_err());
        assert_eq!(round_trip(&knights, "Nbd2"), "Nbd2");
    }

    #[test]
    fn en_passant_underpromotion_and_castling() {
        assert_eq!(round_trip(&fen("3r1rk1/1R4pp/p3p3/4Pp2/2p1Q3/2q5/2B3PP/4RK2 w - f6 0 25"), "exf6"), "exf6");
        assert_eq!(round_trip(&fen("7k/P7/8/8/8/8/8/7K w - - 0 1"), "a8=N"), "a8=N");
        let castles = fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1");
        assert_eq!(round_trip(&castles, "0-0"), "O-O");
        assert_eq!(round_trip(&castles, "O-O-O"), "O-O-O");
        assert_eq!(round_trip(&fen("7k/8/8/8/8/R7/8/R6K w - - 0 1"), "R1a2"), "R1a2");
        assert_eq!(round_trip(&fen("7k/8/8/8/8/1N6/8/1N3N1K w - - 0 1"), "Nb1d2"), "Nb1d2");
    }

    #[test]
    fn distinguishes_mate_from_stalemate() {
        let start = BoardState::start_pos();
        assert_eq!(encode(&start, &["f2f3", "e7e5", "g2g4", "d8h4"]).unwrap().join(" "), "f3 e5 g4 Qh4#");
        assert_eq!(round_trip(&fen("7k/8/5KQ1/8/8/8/8/8 w - - 0 1"), "Qf7"), "Qf7");
    }

    #[test]
    fn decodes_and_encodes_sequences() {
        let start = BoardState::start_pos();
        let sans = ["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Bxc6", "dxc6", "O-O", "Bg4", "h3", "Bxf3", "Qxf3"];
        let ucis: Vec<String> = decode(&start, &sans).unwrap().iter().map(|m| m.uci()).collect();
        assert_eq!(ucis.join(" "), "e2e4 e7e5 g1f3 b8c6 f1b5 a7a6 b5c6 d7c6 e1g1 c8g4 h2h3 g4f3 d1f3");
        assert_eq!(encode(&start, &ucis).unwrap(), sans);
        assert!(decode(&start, &["e5"]).is_err());
    }
}
