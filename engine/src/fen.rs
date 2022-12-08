#![allow(unused_variables, unused_imports)]

use std::iter::Scan;
use crate::bitboard::Bitboard;
use crate::board_state::BoardState;
use crate::piece::{BLACK_KING, BLACK_ROOK, parse_piece, Piece, to_piece_char, WHITE_KING, WHITE_ROOK};
use crate::side::Side;
use crate::side::Side::{BLACK, WHITE};
use crate::square::Square;

pub const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn from_fen_default(fen: &str) -> BoardState { // TODO remove
    from_fen(fen, 100)
}

pub fn from_fen(fen: &str, max_search_depth: usize) -> BoardState {
    let mut fen_parts = fen.split(' ');
    let pieces_part = fen_parts.next().unwrap();

    let pieces: Vec<Piece> = pieces_part.split('/').rev().flat_map(|x| x.chars())
        .map(|x| if x.is_ascii_digit() {vec!['1'; x.to_digit(10).unwrap() as usize]} else {vec![x]})
        .map(|y| y.into_iter())
        .flat_map(|z| z)
        .map(|x| parse_piece(x))
        .collect();
        //.for_each(|x| { println!("{}", x) });
    let items: [Piece; 64] = pieces.try_into().unwrap();

    let side_part = fen_parts.next().unwrap();
    let side: Side = Side::try_from(side_part.chars().next().unwrap()).unwrap();

    let castling_part = fen_parts.next().unwrap();
    let mut movements: u64 = 0;
    if !castling_part.contains("K") || items[Bitboard::WHITE_KING_INITIAL_SQUARE] != WHITE_KING
        || items[Bitboard::WHITE_KINGS_ROOK_MASK.trailing_zeros() as usize] != WHITE_ROOK
    { movements |= Bitboard::WHITE_KINGS_ROOK_MASK }
    if !castling_part.contains("Q") || items[Bitboard::WHITE_KING_INITIAL_SQUARE] != WHITE_KING
        || items[Bitboard::WHITE_QUEENS_ROOK_MASK.trailing_zeros() as usize] != WHITE_ROOK
    { movements |= Bitboard::WHITE_QUEENS_ROOK_MASK };
    if !castling_part.contains("k") || items[Bitboard::BLACK_KING_INITIAL_SQUARE] != BLACK_KING
        || items[Bitboard::BLACK_KINGS_ROOK_MASK.trailing_zeros() as usize] != BLACK_ROOK
    { movements |= Bitboard::BLACK_KINGS_ROOK_MASK };
    if !castling_part.contains("q") || items[Bitboard::BLACK_KING_INITIAL_SQUARE] != BLACK_KING
        || items[Bitboard::BLACK_QUEENS_ROOK_MASK.trailing_zeros() as usize] != BLACK_ROOK
    { movements |= Bitboard::BLACK_QUEENS_ROOK_MASK };


    let en_passant_part = fen_parts.next().unwrap();
    let en_passant_parsed = if en_passant_part.eq("-") { None } else { Some(Square::get_square_from_name(en_passant_part)) };
    let en_passant_mask = en_passant_parsed.map(|s| 1u64 << s).unwrap_or(0u64);

    let halfmove_clock_part = fen_parts.next().unwrap();
    let halfmove_clock = halfmove_clock_part.parse::<usize>().unwrap();
    let fullmove_part = fen_parts.next().unwrap();
    let fullmove = fullmove_part.parse::<usize>().unwrap();

    BoardState::new(&items, side, movements, en_passant_mask, halfmove_clock, fullmove, 100)
    //println!("{:?}", state);
    //return state;
    // BoardState::new(&[0; 64], WHITE, 0, 0, 0, 0, 100)
}

pub fn to_fen(state: &BoardState) -> String {
    let pieces = state.items.chunks(8).rev()
        .map(|r| {
            let line: Vec<char> = r.iter().map(|p| to_piece_char(*p)).map(|p| if p.eq(&' ') { '1'} else { p } ).collect();
            // iterator's method .scan() is somehow not working for me, therefore I resort to
            line.into_iter().collect::<String>()
                .replace("11111111", "8")
                .replace("1111111", "7")
                .replace("111111", "6")
                .replace("11111", "5")
                .replace("1111", "4")
                .replace("111", "3")
                .replace("11", "2")
        }).collect::<Vec<String>>().join("/");

    let side = state.side_to_play;

    let mut castling = String::new();
    if (Bitboard::castling_pieces_kingside_mask(WHITE) & state.movements) == 0 {
        castling.push('K');
    }
    if (Bitboard::castling_pieces_queenside_mask(WHITE) & state.movements) == 0 {
        castling.push('Q');
    }
    if (Bitboard::castling_pieces_kingside_mask(BLACK) & state.movements) == 0 {
        castling.push('k');
    }
    if (Bitboard::castling_pieces_queenside_mask(BLACK) & state.movements) == 0 {
        castling.push('q');
    }

    format!("{} {} {} {} {} {}",
            pieces,
            side,
            if castling.len() > 0 { castling } else {"-".to_string()},
            if state.en_passant > 0 {Square::get_name(state.en_passant.trailing_zeros() as usize)} else {"-".to_string()},
            state.half_move_clock,
            (state.full_move_normalized / 2) + 1
    )
}

#[cfg(test)]
mod tests {
    use crate::bitboard::Bitboard;
    use crate::fen::{from_fen_default, START_POS, to_fen};

    #[test]
    fn from_fen_startpos() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS);
        assert_eq!(state.to_string(), "+---+---+---+---+---+---+---+---+\n\
                                       | r | n | b | q | k | b | n | r |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       | p | p | p | p | p | p | p | p |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       |   |   |   |   |   |   |   |   |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       |   |   |   |   |   |   |   |   |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       |   |   |   |   |   |   |   |   |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       |   |   |   |   |   |   |   |   |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       | P | P | P | P | P | P | P | P |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       | R | N | B | Q | K | B | N | R |\n\
                                       +---+---+---+---+---+---+---+---+\n");
    }

    #[test]
    fn from_fen_developed() {
        let bitboard = Bitboard::new();
        let state = from_fen_default("r2q1rk1/pbp2ppp/1pnp1n2/8/2PPp3/2P1P3/P1N2PPP/R1BQKB1R w kq - 0 10");
        assert_eq!(state.to_string(), "+---+---+---+---+---+---+---+---+\n\
                                       | r |   |   | q |   | r | k |   |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       | p | b | p |   |   | p | p | p |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       |   | p | n | p |   | n |   |   |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       |   |   |   |   |   |   |   |   |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       |   |   | P | P | p |   |   |   |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       |   |   | P |   | P |   |   |   |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       | P |   | N |   |   | P | P | P |\n\
                                       +---+---+---+---+---+---+---+---+\n\
                                       | R |   | B | Q | K | B |   | R |\n\
                                       +---+---+---+---+---+---+---+---+\n");
    }

    #[test]
    fn to_fen_startpos() {
        let bitboard = Bitboard::new();
        let state = from_fen_default(START_POS);
        let fen = to_fen(&state);
        assert_eq!(fen, START_POS);
    }

    #[test]
    fn to_fen_no_castling() {
        let bitboard = Bitboard::new();
        let fen_without_castling = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - - 0 1";
        let state = from_fen_default(fen_without_castling);
        let fen = to_fen(&state);
        assert_eq!(fen, fen_without_castling);
    }

    #[test]
    fn to_fen_with_en_passant() {
        let bitboard = Bitboard::new();
        let fen_with_en_passant = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1";
        let state = from_fen_default(fen_with_en_passant);
        let fen = to_fen(&state);
        println!("{}", fen);
        assert_eq!(fen, fen_with_en_passant);
    }
}
