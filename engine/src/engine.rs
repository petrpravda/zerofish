use std::fs::File;
use std::io::Write;
use crate::board_position::BoardPosition;
use crate::board_state::BoardState;
use crate::fen::{START_POS, to_fen};
use crate::perft::Perft;
use crate::r#move::Move;
use crate::search::{Search, SearchLimit};
use crate::transposition::Depth;

pub enum UciMessage {
    UciCommand(String),
    #[allow(dead_code)]
    Stop
}

// pub enum Message {
//     NewGame,
//     SetPosition(String, Vec<UCIMove>),
//     SetTranspositionTableSize(i32),
//     Go {
//         depth: i32,
//         wtime: i32,
//         btime: i32,
//         winc: i32,
//         binc: i32,
//         movetime: i32,
//         movestogo: i32,
//     },
//     Perft(i32),
//     IsReady,
//     Stop,
//     PrepareEval(Vec<(String, f64)>),
//     PrepareQuiet(Vec<(String, f64)>),
//     Eval(f64),
//     Fen,
//     PrintTestPositions,
//     ResetTestPositions,
//     Profile,
//     SetOption(String, i32),
//     SetArrayOption(String, i32, i32),
//     Quit,
//     Display,
// }

// #[derive(Copy, Clone)]
// pub struct EvalBoardPos {
//     result: f64,
//     pieces: [i8; 64],
//     halfmove_count: u16,
//     castling_state: u8,
//     is_quiet: bool
// }

// impl EvalBoardPos {
//     pub fn apply(&self, board: &mut Board) {
//         board.eval_set_position(&self.pieces, self.halfmove_count, self.castling_state);
//     }
// }

pub struct Engine {
    // bitboard: &'a Bitboard,
    // board_state: BoardState,
    position: BoardPosition,
    search: Search,
    file: File,
}

impl Engine {
    #[allow(unused)]
    pub fn new_from_fen(fen: &str) -> Self {
        let mut file = File::create("zerofish.log").unwrap();
        let mut engine = Engine {
//            bitboard,
            position: BoardPosition::from_fen(fen),
            search: Search::new(),
            file,
            //board_state: from_fen_default(fen),
        };

        engine
    }

    fn get_board_state(&self) -> &BoardState {
        &self.position.state
    }

    pub fn process_uci_command(&mut self, uci_command: String) -> String {
        let msg = format!("Processing: {}", uci_command);
        self.file.write(msg.as_ref()).expect("TODO: panic message");
        let parts: Vec<&str> = uci_command.split_whitespace().collect();
        let part = parts.get(0);
        let sub_part = parts.get(1);
        if part.is_some() {
            let result: String = match part.unwrap().to_lowercase().as_str() {
                "uci" => {
                    println!(r#"id name {}
id author Petr Pravda
uciok"#, "zerofish 0.1.0 64\
");
                    "OK".to_string()
                },
                "go" => {
                    let depth = parts.get(2).map(|d| d.parse::<u32>()).map(|e| e.unwrap());
                    if parts.len() == 3 && sub_part.unwrap().eq(&"perft") {
                        let (result, _count) = Perft::perft_sf_string(&self.get_board_state(), depth.unwrap() as u16);
                        println!("{}", result);
                    } else if parts.len() == 3 && sub_part.unwrap().eq(&"depth") {
                        // TODO unify common logic for moves and depth
                        let depth = parts[2].parse::<Depth>().unwrap();
                        let result = self.search.it_deep(&self.position, SearchLimit::for_depth(depth));
                        println!("bestmove {}", result.moov.map(|m| m.uci()).unwrap_or(String::from("(none)")));
                    } else if parts.len() == 3 && sub_part.unwrap().eq(&"nodes") {
                        let move_count = parts[2].parse::<u32>().unwrap();
                        let result = self.search.it_deep(&self.position, SearchLimit::for_move_count(move_count));
                        println!("bestmove {}", result.moov.map(|m| m.uci()).unwrap_or(String::from("(none)")));
                    }
                    String::from("go")
                    // let depth = extract_option(&parts, "depth", 3);
                    //
                    // self.go(depth, 0, 0, 0, 0, 0, 0)
                },

                "d" => {
                    // let (legal_moves_string, checker_moves_string) = generateMoves(&mut self.board);
                    // //String checkers = checkerMoves.stream().map(m -> Square.getName(m.start())).collect(Collectors.joining(" "));
                    //
                    let state = self.get_board_state();
                    let mut output = state.to_string();
                    output.push_str(format!("Fen: {}\n", to_fen(state)).as_str());
                    // output.push_str(format!("Checkers:{}\n", checker_moves_string).as_str());
                    //output.push_str(format!("Legal uci moves:{}\n", legal_moves_string).as_str());
                    println!("{}", output);
                    output.to_string()
                }

                "isready" => self.is_ready(),

                "quit" => "quitting".to_string(),

                "ucinewgame" => {
                    // self.reset();
                    "OK".to_string()
                },

                "position" => {
                    if parts.len() >= 3 && sub_part.unwrap().eq(&"fen") {
                        let fen = parts[2..].to_vec().join(" ");
                        self.position = BoardPosition::from_fen(&*fen);
                    } else if parts.contains(&"startpos") {
                        self.position = BoardPosition::from_fen(START_POS);
                    }

                    let moves = match parts.iter().position(|&part| part == "moves") {
                        Some(idx) => Engine::parse_moves(idx, &parts, &self.position.state),
                        None => Vec::new(),
                    };

                    for moov in moves {
                        self.position.do_move(&moov);
                    }


                    "OK".to_string()
                },

                // "setoption" => {
                //     if parts.len() != 5 || parts[1] != "name" || parts[3] != "value" {
                //         eprintln!("Expecting 4 arguments, in form name XXXXX value YYY, got: {}", parts[1..].to_vec().join(" "));
                //         return "Failed".to_string();
                //     }
                //
                //     self.board.options.set_option(parts[2], parts[4]); //set_position_from_uci(&parts[1..].to_vec());
                //     "OK".to_string()
                // },
                //
                // "ucitopgn" => {
                //     if parts.len() >= 9 && (*parts.get(1).unwrap()).eq("fen") && (*parts.get(8).unwrap()).eq("moves") {
                //         let fen_parts = parts[2..8].join(" ");
                //         let mut board = create_from_fen(&fen_parts);
                //
                //         let all = parts.len() >= 10 && (*parts.get(9).unwrap()).eq("all");
                //         let all_uci_moves = parse_moves(if all { 9 } else { 8 }, &parts);
                //         let result;
                //         if all_uci_moves.is_empty() {
                //             result = "(empty)".to_string();
                //         } else if all {
                //             let last_uci_moves = UciMoves::new(parts[10..].iter().map(|m| m.to_string()).collect());
                //             result = last_uci_moves.as_san(&mut board);
                //         } else {
                //             for i in 0..all_uci_moves.len() - 1 {
                //                 let uci_move = all_uci_moves.get(i).unwrap();
                //                 let the_move = uci_move.to_move(&board);
                //                 //println!("moving: {}", the_move);
                //                 board.perform_move(the_move);
                //             }
                //             let last_uci_moves = UciMoves::new(parts[parts.len() - 1..].iter().map(|m| m.to_string()).collect());
                //             result = last_uci_moves.as_san(&mut board);
                //         }
                //         println!("{}", result);
                //         result
                //     } else {
                //         "unsupported format".to_string()
                //     }
                // },
                //
                // "pgntouci" => {
                //     let splits = parts[1..].iter().map(|s|s.to_string()).collect::<Vec<String>>();
                //     let pgn_moves = PgnMoves::new(splits);
                //     let result = pgn_moves.as_uci();
                //     println!("{}", result);
                //     result
                // },


                _ => {
                    //println!("Skipping execution of: {}", uci_command);
                    // Skip unknown commands
                    let result = format!("Unsupported command: {}", uci_command);
                    result
                }
            };
            result
        } else {
            "Empty command".to_string()
        }
    }

    // fn set_position_from_uci(&mut self, parts: &Vec<&str>) {
    //     let fen = parse_position_cmd(parts);
    //
    //     let moves = match parts.iter().position(|&part| part == "moves") {
    //         Some(idx) => parse_moves(idx, &parts),
    //         None => Vec::new(),
    //     };
    //
    //     self.set_position(fen, moves);
    // }
    //
    // fn go(&mut self, depth: i32, wtime: i32, btime: i32, winc: i32, binc: i32, movetime: i32, movestogo: i32) -> String {
    //     self.timelimit_ms = if self.board.active_player() == WHITE {
    //         calc_timelimit(movetime, wtime, winc, movestogo)
    //     } else {
    //         calc_timelimit(movetime, btime, binc, movestogo)
    //     };
    //
    //     let time_left = if self.board.active_player() == WHITE {
    //         wtime
    //     } else {
    //         btime
    //     };
    //
    //     let is_strict_timelimit =
    //         movetime != 0 || (time_left - (TIMEEXT_MULTIPLIER * self.timelimit_ms) <= 20) || movestogo == 1;
    //
    //     let m = self.find_best_move(depth, is_strict_timelimit);
    //     if m == NO_MOVE {
    //         let msg = "bestmove (none)";
    //         println!("{}", msg);
    //         self.search_result.push_str(msg);
    //     } else {
    //         let msg = format!(
    //             "bestmove {}",
    //             UCIMove::from_encoded_move(&self.board, m).to_uci()
    //         );
    //         println!("{}", msg);
    //         self.search_result.push_str(msg.as_str());
    //     }
    //     let result = &self.search_result.clone();
    //     self.search_result = "".to_string();
    //     result.to_string()
    // }

    fn is_ready(&mut self) -> String {
        // if self.options_modified {
        //     self.options_modified = false;
        //     self.board.pst.recalculate(&self.board.options);
        // }
        println!("readyok");
        "readyok".to_string()
    }

    fn parse_moves(idx: usize, parts: &Vec<&str>, original_state: &BoardState) -> Vec<Move> {
        let mut state = original_state.clone();
        let mut moves: Vec<Move> = Vec::new();

        for i in (idx + 1)..parts.len() {
            let moov = Move::from_uci_string(parts[i], &state);
            state = state.do_move(&moov);
            moves.push(moov);
        }

        moves
    }
}
