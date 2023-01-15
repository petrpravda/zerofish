use crate::board_state::BoardState;
use crate::piece::PieceType;
use crate::side::Side;
use crate::square::Square;
use crate::r#move::Move;

pub struct Pgn {
    // move_strings: Vec<String>
}

impl Pgn {
    // pub fn new(move_strings: Vec<String>) -> Self {
    //     PgnMoves {
    //         move_strings
    //     }
    // }
    //
    // pub fn as_san(&self) -> String {
    //     self.move_strings.join(" ")
    // }
    //
    // pub fn as_uci(&self) -> String {
    //     let mut board = create_from_fen(START_POS);
    //     let result = self.move_strings.iter()
    //         .map(|mowe | PgnMoves::one_san_to_uci(&mowe, &mut board))
    //         .collect::<Vec<String>>()
    //         .join(" ");
    //     result
    // }

    pub fn one_san_to_uci(sanx: &str, state: &BoardState) -> String {
        let checking_move = sanx.ends_with("+");
        let checkmating_move = sanx.ends_with("#");
        let mut san = if checking_move || checkmating_move { sanx[..sanx.len() - 1].to_string() } else { sanx.to_string() };
        let destination: String;
        let piece: PieceType;
        let mut promotion_piece: Option<PieceType> = None;
        let mut from_file: Option<char> = None;
        let mut from_rank: Option<char> = None;

        if san.eq("O-O") {
            destination = if state.side_to_play == Side::WHITE { "g1".to_string() } else { "g8".to_string() };
            piece = PieceType::KING;
        } else if san.eq("O-O-O") {
            destination = if state.side_to_play == Side::WHITE { "c1".to_string() } else { "c8".to_string() };
            piece = PieceType::KING;
        } else {
            if san.chars().nth(san.len() - 2).unwrap() == '=' {
                let piece_code = san[san.len() - 1..].to_string().chars().next().unwrap();
                promotion_piece = PieceType::from_san_code(piece_code);
                san = san[..san.len() - 2].to_string();
            }
            let piece_type_optional = PieceType::from_san_code(san.chars().next().unwrap());
            piece = piece_type_optional.unwrap_or(PieceType::PAWN);
            let san_without_piece_type: String;
            if piece_type_optional.is_some() {
                san_without_piece_type = san[1..].to_string();
            } else {
                san_without_piece_type = san;
            }
            let parts: Vec<String> = san_without_piece_type.split('x').map(|x|x.to_string()).collect();
            let source: String;
            if parts.len() > 1 {
                source = parts[0].clone();
                destination = parts[1].clone();
            } else {
                let source_length = san_without_piece_type.len() - 2;
                // if source_length < 0 {
                //     panic!("Weird move: {}", sanx);
                // }
                source = san_without_piece_type[..source_length].to_string();
                destination = san_without_piece_type[source_length..].to_string();
            }
            match source.len() {
                0 => {},
                1 => {
                    let from_char = source.chars().next().unwrap();
                    if from_char.is_digit(10) {
                        from_rank = Some(from_char);
                    } else {
                        from_file = Some(from_char);
                    }
                },
                2 => {
                    let mut chars_iter = source.chars();
                    from_file = chars_iter.next();
                    from_rank = chars_iter.next();
                },
                _ => {
                    panic!("{} is not implemented, length???, [{}]", san_without_piece_type, sanx);
                }
            }
        }
        let destination_number = Square::get_square_from_name(destination.as_str());
        let uci_moves = state.generate_legal_moves();
        let matching_moves = uci_moves.moves.iter()
            .filter(|mowe| mowe.to() == destination_number)
            .filter(|mowe| !mowe.is_promotion() || (promotion_piece.is_some() && mowe.get_piece_type() == promotion_piece.unwrap()))
            .filter(|mowe| state.piece_type_at(mowe.from()) == piece)
            .filter(|mowe| from_file.is_none() || Square::get_file(mowe.from() as usize) == from_file.unwrap())
            .filter(|mowe| from_rank.is_none() || Square::get_rank(mowe.from() as usize) == from_rank.unwrap())
            .collect::<Vec<&Move>>();
        return match matching_moves.len() {
            1 => {
                let the_move = matching_moves.first().unwrap();
                //state.perform_move(**the_move);
                the_move.to_string()
            },
            0 => {
                panic!("Move {} not found", sanx);
            },
            _ => {
                panic!("Ambiguous possible moves: {}", matching_moves.iter().map(|m| m.to_string()).collect::<Vec<String>>().join(" "));
            }
        }
    }
}

// pub struct UciMoves {
//     move_strings: Vec<String>
// }
//
// impl UciMoves {
//     pub fn new(move_strings: Vec<String>) -> Self {
//         UciMoves {
//             move_strings
//         }
//     }
//
//     pub fn as_uci(&self) -> String {
//         self.move_strings.join(" ")
//     }
//
//     pub fn as_san(&self, board: &mut Board) -> String {
//         //let mut board = create_from_fen(START_POS);
//         let result = self.move_strings.iter()
//             .map(|m| UciMoves::one_uci_to_san(&m, board))
//             .collect::<Vec<String>>()
//             .join(" ");
//         result
//     }
//
//     fn one_uci_to_san(uci_move: &String, board: &mut Board) -> String {
//         //println!("uci_move: {}", uci_move);
//         let uci_moves = board.generate_legal_moves();
//         //uci_moves.iter().for_each(|m|println!("list: {}", m.to_string()));
//         let move_cheating = uci_moves.iter()
//             .filter(|m| m.to_string().eq(uci_move))
//             .next();
//
//         let piece = move_cheating.map(|m| m.piece_id());
//         let uci_destination = uci_move[2..4].to_string();
//         let mut chars_iterator = uci_move.chars();
//         let source_file = chars_iterator.next().unwrap();
//         let source_rank = chars_iterator.next().unwrap();
//
//         let mut matching_moves = uci_moves.iter()
//             .filter(|m| Square::get_name(m.end() as usize).eq(&uci_destination))
//             .filter(|m| piece.is_none() || piece.unwrap() == m.piece_id())
//             .filter(|m| move_cheating.is_none() || m.typ() == move_cheating.unwrap().typ())
//             .collect::<Vec<&Move>>();
//         if matching_moves.len() == 1 { // TODO predelat na first()
//             return UciMoves::move_to_pgn(matching_moves.first().unwrap(), board, false, false);
//         }
//
//         // file only
//         matching_moves = uci_moves.iter()
//             .filter(|m| Square::get_name(m.end() as usize).eq(&uci_destination))
//             .filter(|m| piece.is_none() || piece.unwrap() == m.piece_id())
//             .filter(|m| source_file == Square::get_file(m.start() as usize))
//             .filter(|m| move_cheating.is_none() || m.typ() == move_cheating.unwrap().typ())
//             .collect::<Vec<&Move>>();
//         if matching_moves.len() == 1 {
//             return UciMoves::move_to_pgn(matching_moves.first().unwrap(), board, true, false);
//         }
//
//         // rank only
//         matching_moves = uci_moves.iter()
//             .filter(|m| Square::get_name(m.end() as usize).eq(&uci_destination))
//             .filter(|m| piece.is_none() || piece.unwrap() == m.piece_id())
//             .filter(|m| source_rank == Square::get_rank(m.start() as usize))
//             .filter(|m| move_cheating.is_none() || m.typ() == move_cheating.unwrap().typ())
//             .collect::<Vec<&Move>>();
//         if matching_moves.len() == 1 {
//             return UciMoves::move_to_pgn(matching_moves.first().unwrap(), board, false, true);
//         }
//
//         // both file and rank
//         matching_moves = uci_moves.iter()
//             .filter(|m| Square::get_name(m.end() as usize).eq(&uci_destination))
//             .filter(|m| piece.is_none() || piece.unwrap() == m.piece_id())
//             .filter(|m| source_file == Square::get_file(m.start() as usize))
//             .filter(|m| source_rank == Square::get_rank(m.start() as usize))
//             .filter(|m| move_cheating.is_none() || m.typ() == move_cheating.unwrap().typ())
//             .collect::<Vec<&Move>>();
//         if matching_moves.len() == 1 {
//             return UciMoves::move_to_pgn(matching_moves.first().unwrap(), board, true, true);
//         }
//
//         let moves_string = matching_moves.iter().map(|m| m.to_string()).collect::<Vec<String>>().join(" ");
//         if matching_moves.len() > 1 {
//             panic!("Ambiguous possible moves: {} for {}", moves_string, uci_move)
//         } else {
//             panic!("Move {} not found", uci_move);
//         }
//     }
//
//     fn move_to_pgn(the_move: &Move, board: &mut Board, file_needed_param: bool, rank_needed: bool) -> String {
//         let destination = Square::get_name(the_move.end() as usize);
//         let mut piece_identification: Option<String> = None;
//         let mut file_needed = file_needed_param;
//         let half_result: String;
//         let capturing: bool; // = false;
//         let move_string = the_move.to_string();
//         if the_move.piece_id() == K && (move_string.eq("e1g1") || move_string.eq("e8g8")) {
//             half_result = "O-O".to_string();
//         } else if the_move.piece_id() == K && (move_string.eq("e1c1") || move_string.eq("e8c8")) {
//             half_result = "O-O-O".to_string();
//         } else {
//             if (the_move.typ() != Promotion && the_move.piece_id() == P)
//                 || the_move.typ() == Promotion {
//                 capturing = Square::get_file(the_move.start() as usize) != Square::get_file(the_move.end() as usize);
//                 if capturing {
// //                        sourceFile = Square.getFile(mowe.start());
//                     file_needed = true;
//                 }
//             } else {
//                 capturing = the_move.typ() == Capture
//                     || the_move.typ() == KingCapture;
//                 piece_identification = Some(get_piece_string(the_move.piece_id()).to_string());
//             }
//             half_result = format!("{}{}{}{}{}",
//                                   piece_identification.unwrap_or("".to_string()),
//                                   if file_needed { Square::get_file(the_move.start() as usize).to_string() } else {"".to_string()},
//                                   if rank_needed { Square::get_rank(the_move.start() as usize).to_string() } else {"".to_string()},
//                                   if capturing { "x" } else { "" },
//                                   destination);
//         }
//         let checking = board.is_move_checking(*the_move, false);
//         let promotion_suffix = match the_move.typ() { Promotion => ["=", get_piece_string(the_move.piece_id()).to_string().as_str()].concat(), _ => "".to_string()};
//         // StringBuilder result = new StringBuilder(half_result);
//         // if (the_move.typ() == EnumMoveType.PROMOTION) {
//         //     result.append('=');
//         //     result.append(the_move.piece_id().name());
//         // }
//
//         board.perform_move(*the_move);
//
//         let legal_moves_for_opponent = board.generate_legal_moves();
//
//         let check_or_mate_suffix = if legal_moves_for_opponent.len() == 0 && checking {
//             // we need to be sure that it is the checkmate, therefore "checking"
//             "#".to_string()
//         } else if checking {
//             "+".to_string()
//         } else {
//             "".to_string()
//         };
//
//         let result = format!("{}{}{}", half_result, promotion_suffix, check_or_mate_suffix);
// //        if (result.toString().equals("cc8=Q")) {
// //            //checking = board.isMoveChecking(mowe, false);
// //            System.out.println();
// //        }
//
//
//         result
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fen::{Fen, START_POS};
    //use web_sys::target;

    #[test]
    fn one_san_to_uci_d4() {
        let state = Fen::from_fen_default(START_POS);
        let san = "d4".to_string();
        let result = Pgn::one_san_to_uci(&san, &state);
        assert_eq!("d2d4", result);
    }

    #[test]
    fn one_san_to_uci_b3() {
        let state = Fen::from_fen_default(START_POS);
        let san = "b3".to_string();
        let result = Pgn::one_san_to_uci(&san, &state);
        assert_eq!("b2b3", result);
    }

    #[test]
    fn one_san_to_uci_piece_ambiguity() {
        let state = Fen::from_fen_default("r1bqkbnr/pp2pppp/2np4/2p5/8/1P4P1/PBPPPPBP/RN1QK1NR b KQkq - 1 4");
        let san = "Nf6".to_string();
        let result = Pgn::one_san_to_uci(&san, &state);
        assert_eq!("g8f6", result);
    }

    #[test]
    fn one_san_to_uci_promotion() {
        let state = Fen::from_fen_default("4R3/2P3k1/p5p1/7p/8/1B1P2p1/P6P/6K1 w - - 0 37");
        let san = "c8=Q".to_string();
        let result = Pgn::one_san_to_uci(&san, &state);
        assert_eq!("c7c8q", result);
    }

    #[test]
    fn pgn_to_uci_promotion() {
        let pgn = "b3 c5 Bb2 Nc6 g3 d6 Bg2 Nf6 c4 a6 Nc3 e5 d3 Nd4 e3 Bg4 Qd2 Nf5 Nge2 Bxe2 Qxe2 g6 Bxb7 Rb8 Bc6+ Nd7 O-O Bg7 Bg2 O-O Nd5 Nb6 Nxb6 Rxb6 Bh3 Qf6 f4 Rb4 fxe5 dxe5 e4 Qe7 exf5 Kh8 Rae1 Rbb8 f6 Bxf6 Rxf6 Qxf6 Bxe5 Qxe5 Qxe5+ Kg8 Bg2 Rbe8 Qxe8 Rxe8 Rxe8+ Kg7 Bd5 h5 b4 cxb4 c5 b3 Bxb3 f5 c6 f4 c7 fxg3 c8=Q gxh2+ Kxh2 h4 Qe6 Kh6 Rg8 Kg5 Rxg6+ Kh5 Qg4#";
        let mut state = Fen::from_fen_default(START_POS);

        let uci_vec: Vec<String> = pgn.split(" ")
            .map(|san_move| {
                let uci = Pgn::one_san_to_uci(san_move, &state);
                let moov = Move::from_uci_string(&uci, &state);
                state = state.do_move_no_history(&moov);
                uci
            })
            .collect();

        let all_uci_moves = uci_vec.join(" ");
        assert_eq!("b2b3 c7c5 c1b2 b8c6 g2g3 d7d6 f1g2 g8f6 c2c4 a7a6 b1c3 e7e5 d2d3 c6d4 e2e3 c8g4 d1d2 d4f5 g1e2 g4e2 d2e2 g7g6 g2b7 a8b8 b7c6 f6d7 e1g1 f8g7 c6g2 e8g8 c3d5 d7b6 d5b6 b8b6 g2h3 d8f6 f2f4 b6b4 f4e5 d6e5 e3e4 f6e7 e4f5 g8h8 a1e1 b4b8 f5f6 g7f6 f1f6 e7f6 b2e5 f6e5 e2e5 h8g8 h3g2 b8e8 e5e8 f8e8 e1e8 g8g7 g2d5 h7h5 b3b4 c5b4 c4c5 b4b3 d5b3 f7f5 c5c6 f5f4 c6c7 f4g3 c7c8q g3h2 g1h2 h5h4 c8e6 g7h6 e8g8 h6g5 g8g6 g5h5 e6g4", all_uci_moves);
    }

    // #[test]
    // fn uci_to_pgn_promotion() {
    //     let uci = "b2b3 c7c5 c1b2 b8c6 g2g3 d7d6 f1g2 g8f6 c2c4 a7a6 b1c3 e7e5 d2d3 c6d4 e2e3 c8g4 d1d2 d4f5 g1e2 g4e2 d2e2 g7g6 g2b7 a8b8 b7c6 f6d7 e1g1 f8g7 c6g2 e8g8 c3d5 d7b6 d5b6 b8b6 g2h3 d8f6 f2f4 b6b4 f4e5 d6e5 e3e4 f6e7 e4f5 g8h8 a1e1 b4b8 f5f6 g7f6 f1f6 e7f6 b2e5 f6e5 e2e5 h8g8 h3g2 b8e8 e5e8 f8e8 e1e8 g8g7 g2d5 h7h5 b3b4 c5b4 c4c5 b4b3 d5b3 f7f5 c5c6 f5f4 c6c7 f4g3 c7c8q g3h2 g1h2 h5h4 c8e6 g7h6 e8g8 h6g5 g8g6 g5h5 e6g4";
    //     let splits = uci.split(" ").map(|s|s.to_string()).collect::<Vec<String>>();
    //     let uci_moves = UciMoves::new(splits);
    //     //println!("{}", pgn_moves.as_uci());
    //     let mut board = create_from_fen(START_POS);
    //     let result = uci_moves.as_san(&mut board);
    //     assert_eq!("b3 c5 Bb2 Nc6 g3 d6 Bg2 Nf6 c4 a6 Nc3 e5 d3 Nd4 e3 Bg4 Qd2 Nf5 Nge2 Bxe2 Qxe2 g6 Bxb7 Rb8 Bc6+ Nd7 O-O Bg7 Bg2 O-O Nd5 Nb6 Nxb6 Rxb6 Bh3 Qf6 f4 Rb4 fxe5 dxe5 e4 Qe7 exf5 Kh8 Rae1 Rbb8 f6 Bxf6 Rxf6 Qxf6 Bxe5 Qxe5 Qxe5+ Kg8 Bg2 Rbe8 Qxe8 Rxe8 Rxe8+ Kg7 Bd5 h5 b4 cxb4 c5 b3 Bxb3 f5 c6 f4 c7 fxg3 c8=Q gxh2+ Kxh2 h4 Qe6 Kh6 Rg8 Kg5 Rxg6+ Kh5 Qg4#", result);
    // }
}
