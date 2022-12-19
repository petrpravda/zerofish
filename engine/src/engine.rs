use std::fs::File;
use crate::transposition::{TranspositionTable};
use std::io::{stdout, Stdout, Write};

use crate::board_position::BoardPosition;
use crate::board_state::BoardState;
use crate::fen::{Fen, START_POS};
use crate::perft::Perft;
use crate::search::{Search, SearchLimitParams};
use crate::util::extract_parameter;

pub enum UciMessage {
    UciCommand(String),
    Stop
}

pub trait OutputAdapter {
    fn writeln(&mut self, output: &str);
}

pub struct StdOutOutputAdapter {
    out: Stdout,
}

impl StdOutOutputAdapter {
    pub fn new() -> Self {
        Self {
            out: stdout()
        }
    }
}

impl OutputAdapter for StdOutOutputAdapter {
    fn writeln(&mut self, output: &str) {
        self.out.write(output.as_ref()).expect("Cannot write to output stream!");
        self.out.write(b"\n").expect("Cannot write to output stream!");
        self.out.flush().expect("Flush error");
    }
}

pub struct StringOutputAdapter {
    string_buffer: String,
}

impl StringOutputAdapter {
    pub fn new() -> Self {
        Self {
            string_buffer: String::new()
        }
    }
}

impl OutputAdapter for StringOutputAdapter {
    fn writeln(&mut self, output: &str) {
        self.string_buffer.push_str(output);
        self.string_buffer.push('\n');
    }
}

impl ToString for StringOutputAdapter {
    fn to_string(&self) -> String {
        self.string_buffer.clone()
    }
}

#[derive(Debug, Clone)]
pub struct EngineOptions {
    pub log_filename: Option<String>,
}

pub struct Engine {
    pub position: BoardPosition,
    pub search: Search,
    file: Option<File>,
}

impl Engine {
    pub fn new(engine_options: EngineOptions) -> Self {
        let file = engine_options.log_filename.map(|filename| File::create(filename).unwrap());
        let transposition_table = TranspositionTable::new(1);
        let search = Search::new(transposition_table);

        let engine = Engine {
            position: BoardPosition::from_fen(START_POS),
            search,
            file,
        };

        engine
    }

    fn get_board_state(&self) -> &BoardState {
        &self.position.state
    }

    pub fn process_uci_command(&mut self, uci_command: String, output_adapter: &mut dyn OutputAdapter) {
        if self.file.is_some() {
            let msg = format!("{}", uci_command);
            self.file.as_ref().unwrap().write(msg.as_ref()).expect("TODO: panic message");
        }
        // println!("{}", uci_command);
        let parts: Vec<&str> = uci_command.split_whitespace().collect();
        let part = parts.get(0);
        let sub_part = parts.get(1);
        if part.is_some() {
            match part.unwrap().to_lowercase().as_str() {
                "uci" => {
                    output_adapter.writeln(&*format!(r#"id name {}
id author Petr Pravda
uciok"#, "zerofish 0.1.0 64\
"));
                },
                "go" => {
                    let search_limit_params: SearchLimitParams = SearchLimitParams {
                        perft_depth: extract_parameter(&parts, "perft"),
                        depth: extract_parameter(&parts, "depth"),
                        max_nodes: extract_parameter(&parts, "nodes"),
                        move_time: extract_parameter(&parts, "movetime"),
                        moves_to_go: extract_parameter(&parts, "movestogo"),
                        w_time: extract_parameter(&parts, "wtime"),
                        b_time: extract_parameter(&parts, "btime"),
                    };


                    if search_limit_params.perft_depth.is_some() {
                        let (result, _count) = Perft::perft_sf_string(&self.get_board_state(), search_limit_params.perft_depth.unwrap());
                        output_adapter.writeln(&*result);
                    } else {
                        let search_limit = search_limit_params.prepare(self.position.state.side_to_play);
                        // println!("search_limit: {:?}", search_limit);
                        let result = self.search.it_deep(&self.position, search_limit,  output_adapter);
                        output_adapter.writeln(&*format!("bestmove {}", result.moov.map(|m| m.uci()).unwrap_or(String::from("(none)"))));
                    }
                },

                "d" => {
                    // let (legal_moves_string, checker_moves_string) = generateMoves(&mut self.board);
                    // //String checkers = checkerMoves.stream().map(m -> Square.getName(m.start())).collect(Collectors.joining(" "));
                    //
                    let state = self.get_board_state();
                    let mut output = state.to_string();
                    output.push_str(format!("Fen: {}\n", Fen::compute_fen(state)).as_str());
                    // output.push_str(format!("Checkers:{}\n", checker_moves_string).as_str());
                    //output.push_str(format!("Legal uci moves:{}\n", legal_moves_string).as_str());
                    output_adapter.writeln(&*output);
                }

                "isready" => {
                    output_adapter.writeln(&"readyok");
                },

                "quit" => {
                    output_adapter.writeln("quitting");
                },

                "ucinewgame" => {
                    self.uci_new_game();
                },

                "position" => {
                    if parts.len() >= 3 && sub_part.unwrap().eq(&"fen") {
                        let fen = parts[2..].to_vec().join(" ");
                        self.position = BoardPosition::from_fen(&*fen);
                    } else if parts.contains(&"startpos") {
                        self.position = BoardPosition::from_fen(START_POS);
                    }

                    let moves = match parts.iter().position(|&part| part == "moves") {
                        Some(idx) => {
                            // let (sublist_before, sublist_after) = parts.split_at(idx + 1);
                            // let parts_after = sublist_after.to_vec();
                            // self.position.state.parse_moves(&parts_after)
                            let (_, parts_after) = parts.split_at(idx + 1);
                            self.position.state.parse_moves(&parts_after.to_vec())
                        },
                        None => Vec::new(),
                    };

                    for moov in moves {
                        self.position.do_move(&moov);
                    }
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
                    let result = format!("Unsupported command: {}", uci_command);
                    output_adapter.writeln(&*result);
                }
            };
        } else {
            output_adapter.writeln("Empty command");
        };
    }

    pub fn uci_new_game(&mut self) {
        self.search.transposition_table.clear();
        self.position = BoardPosition::from_fen(START_POS);
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

    // fn is_ready(&mut self) -> String {
    //     // if self.options_modified {
    //     //     self.options_modified = false;
    //     //     self.board.pst.recalculate(&self.board.options);
    //     // }
    //     self.answer(String::from("readyok"));
    //     "readyok".to_string()
    // }
    // fn extract_parameter_or<T: FromStr>(parts: &Vec<&str>, name: &str, default_value: T) -> T {
    //     extract_parameter(parts, name).unwrap_or(default_value)
    // }
    // fn answer(&mut self, output: &str) {
    //     output_adapter
    // }
}
