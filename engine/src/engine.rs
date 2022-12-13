use crate::board_position::BoardPosition;
use crate::board_state::BoardState;
use crate::fen::{from_fen_default, to_fen};
use crate::perft::Perft;
use crate::search::Search;
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
}

impl Engine {
    #[allow(unused)]
    pub fn new_from_fen(fen: &str) -> Self {
        let mut engine = Engine {
//            bitboard,
            position: BoardPosition::from_fen(fen),
            search: Search::new()

            //board_state: from_fen_default(fen),
        };

        engine
    }

    fn get_board_state(&self) -> &BoardState {
        &self.position.state
    }

    pub(crate) fn process_uci_command(&mut self, uci_command: String) -> String {
        let parts: Vec<&str> = uci_command.split_whitespace().collect();
        let part = parts.get(0);
        let sub_part = parts.get(1);
        if part.is_some() {
            let result: String = match part.unwrap().to_lowercase().as_str() {
                //"fen" => fen(tx),

                "go" => {
                    let depth = parts.get(2).map(|d| d.parse::<u16>()).map(|e| e.unwrap());
                    if parts.len() == 3 && sub_part.unwrap().eq(&"perft") {
                        // println!("PERFT, depth {}", depth.unwrap());
                        let (result, _count) = Perft::perft_sf_string(&self.get_board_state(), depth.unwrap());
                        println!("{}", result);
                    } else if parts.len() == 3 && sub_part.unwrap().eq(&"depth") {
                        let depth = parts[2].parse::<Depth>().unwrap();
                        println!("depth {}", depth);
                        let result = self.search.it_deep(&self.position, depth);
                        println!("{:?}", result);
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

                "ucinewgame" => {
                    // self.reset();
                    "OK".to_string()
                },

                "position" => {
                    // self.set_position_from_uci(&parts[1..].to_vec())che
                    if parts.len() >= 3 && sub_part.unwrap().eq(&"fen") {
                        // let fen = parts[2];
                        let fen = parts[2..].to_vec().join(" ");
                        println!("treti: {}", fen);
                        self.position = BoardPosition::from_fen(&*fen);
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

    // fn parse_position_cmd(parts: &Vec<&str>) -> String {
    //     if parts.is_empty() {
    //         eprintln!("position command: missing fen/startpos");
    //     }
    //
    //     let pos_end = parts
    //         .iter()
    //         .position(|&part| part.to_lowercase().as_str() == "moves")
    //         .unwrap_or_else(|| parts.len());
    //
    //     let pos_option = parts[1..pos_end].join(" ");
    //
    //     if pos_option.is_empty() {
    //         String::from(START_POS)
    //     } else {
    //         pos_option
    //     }
    // }
    //
    // fn parse_moves(idx: usize, parts: &Vec<&str>) -> Vec<UCIMove> {
    //     let mut moves: Vec<UCIMove> = Vec::new();
    //
    //     for i in (idx + 1)..parts.len() {
    //         match UCIMove::from_uci(parts[i]) {
    //             Some(m) => moves.push(m),
    //             None => {
    //                 eprintln!("could not parse move notation: {}", parts[i]);
    //                 return moves;
    //             }
    //         }
    //     }
    //
    //     moves
    // }

    // fn set_position(&mut self, fen: String, moves: Vec<UCIMove>) {
    //     match read_fen(&mut self.board, &fen) {
    //         Ok(_) => (),
    //         Err(err) => println!("position cmd: {}", err),
    //     }
    //
    //     for m in moves {
    //         self.board.perform_move(m.to_move(&self.board));
    //     }
    // }
    //
    // fn prepare_eval(&mut self, fens_with_result: Vec<(String, f64)>) {
    //     for (fen, result) in fens_with_result {
    //         match read_fen(&mut self.board, &fen) {
    //             Ok(_) => (),
    //             Err(err) => println!("prepare_eval cmd: {}", err),
    //         }
    //
    //         if self.board.is_in_check(WHITE) || self.board.is_in_check(BLACK) {
    //             continue;
    //         }
    //
    //         let mut pieces: [i8; 64] = [0; 64];
    //         for i in 0..64 {
    //             pieces[i] = self.board.get_item(i as i32);
    //         }
    //
    //
    //         self.test_positions.push(EvalBoardPos {
    //             result,
    //             pieces,
    //             halfmove_count: self.board.fullmove_count(),
    //             castling_state: self.board.get_castling_state(),
    //             is_quiet: true
    //         });
    //     }
    //
    //     println!("prepared");
    // }
    //
    // fn prepare_quiet(&mut self, fens_with_result: Vec<(String, f64)>) {
    //     for (fen, result) in fens_with_result {
    //         match read_fen(&mut self.board, &fen) {
    //             Ok(_) => (),
    //             Err(err) => println!("prepare_quiet cmd: {}", err),
    //         }
    //
    //         if self.board.is_in_check(-self.board.active_player()) {
    //             continue;
    //         }
    //
    //         let play_moves = (self.rnd.rand64() % 12) as i32;
    //         for _ in 0..play_moves {
    //             let m = self.find_best_move(9, true);
    //             if m == NO_MOVE {
    //                 break;
    //             }
    //
    //             self.board.perform_move(m);
    //         }
    //
    //         if !self.make_quiet() {
    //             continue;
    //         }
    //
    //         let mut pieces: [i8; 64] = [0; 64];
    //         for i in 0..64 {
    //             pieces[i] = self.board.get_item(i as i32);
    //         }
    //
    //         let pos = EvalBoardPos {
    //             result,
    //             pieces,
    //             halfmove_count: self.board.halfmove_count,
    //             castling_state: self.board.get_castling_state(),
    //             is_quiet: true
    //         };
    //
    //         self.test_positions.push(pos);
    //     }
    //
    //     println!("prepared");
    // }
    //
    // fn make_quiet(&mut self) -> bool {
    //     for _ in 0..15 {
    //         if self.board.is_in_check(-self.board.active_player()) {
    //             return false;
    //         }
    //
    //         if self.board.get_static_score().abs() > get_piece_value(Q as usize) as i32 {
    //             return false;
    //         }
    //
    //         let mut is_quiet = self.is_quiet_position();
    //         if !is_quiet && self.make_quiet_position() && self.is_quiet_position() && self.board.get_static_score().abs() <= get_piece_value(Q as usize) as i32 {
    //             is_quiet = true;
    //         }
    //
    //         let m = self.find_best_move(6, true);
    //         if m == NO_MOVE {
    //             return false;
    //         }
    //
    //         if is_quiet && self.is_quiet_pv(m, 4) {
    //             return true;
    //         }
    //
    //         self.board.perform_move(m);
    //     }
    //
    //     false
    // }
    //
    // fn eval(&mut self, k: f64) {
    //
    //     let mut errors: f64 = 0.0;
    //     let k_div = k / 400.0;
    //     for pos in self.test_positions.to_vec().iter() {
    //         pos.apply(&mut self.board);
    //         let score = if pos.is_quiet {
    //             self.board.get_score()
    //         } else {
    //             self.quiescence_search(self.board.active_player(), MIN_SCORE, MAX_SCORE, 0) * self.board.active_player() as i32
    //         };
    //
    //         let win_probability = 1.0 / (1.0 + 10.0f64.powf(-(score as f64) * k_div));
    //         let error = pos.result - win_probability;
    //         errors += error * error;
    //     }
    //
    //     println!("result {}:{}", self.test_positions.len(), errors);
    // }
    //
    // fn print_test_positions(&mut self) {
    //
    //     print!("testpositions ");
    //     let mut is_first = true;
    //     for pos in self.test_positions.to_vec().iter() {
    //         pos.apply(&mut self.board);
    //         self.board.reset_half_move_clock();
    //         let fen = write_fen(&self.board);
    //
    //         if !is_first {
    //             print!(";");
    //         } else {
    //             is_first = false;
    //         }
    //
    //         print!("{}", fen);
    //     }
    //
    //     println!();
    // }
    //
    // fn reset_test_positions(&mut self) {
    //     self.test_positions.clear();
    //     println!("reset completed");
    // }
    //
    // fn set_tt_size(&mut self, size_mb: i32) {
    //     self.tt.resize(size_mb as u64, false);
    // }
    //
    // fn reset(&mut self) {
    //     self.tt.clear();
    //     self.hh.clear();
    // }
    //
    // pub fn perform_move(&mut self, m: Move) {
    //     self.board.perform_move(m);
    // }
    //
    // pub fn profile(&mut self) {
    //     println!("Profiling ...");
    //     self.go(10, 500, 500, 0, 0, 500, 2);
    //     exit(0);
    // }
    //
    // pub fn is_search_stopped(&self) -> bool {
    //     false
    // }
}

// pub fn generateMoves(board: &mut Board) -> (String, String) {
//     let moves: Vec<Move> = board.generate_legal_moves();
//     let legal_moves_string = moves.iter().fold("".to_string(), |mut i, j|
//         {
//             i.push_str(" ");
//             i.push_str(&j.to_string());
//             i
//         });
//
//     let checker_moves: Vec<Move> = board.generate_checker_moves();
//     let checker_moves_string = checker_moves.iter().fold("".to_string(), |mut i, j|
//         {
//             i.push_str(" ");
//             i.push_str(&*Square::get_name(*(&j.start()) as usize));
//             i
//         });
//     (legal_moves_string, checker_moves_string)
// }
