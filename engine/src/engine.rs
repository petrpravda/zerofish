use std::fs::File;
use crate::transposition::{TranspositionTable};
use std::io::{stdout, Stdout, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use crate::board_position::BoardPosition;
use crate::board_state::BoardState;
use crate::fen::{Fen, START_POS};
use crate::perft::Perft;
use crate::search::{Search, SearchLimitParams};
use crate::util::extract_parameter;

pub enum UciMessage {
    UciCommand(String),
}

pub trait EnvironmentContext {
    fn writeln(&mut self, output: &str);
    fn is_stop_signalled(&self) -> bool;
    fn set_stop_signal(&self, new_stop_signal_value: bool);
}

pub struct StdOutEnvironmentContext {
    out: Stdout,
    stop_signal: Arc<AtomicBool>,
}

impl StdOutEnvironmentContext {
    pub fn new_w_signal(stop_signal: Arc<AtomicBool>) -> Self {
        Self {
            out: stdout(),
            stop_signal
        }
    }

    pub fn new() -> Self {
        Self {
            out: stdout(),
            stop_signal: Arc::new(AtomicBool::new(false))
        }
    }
}

impl EnvironmentContext for StdOutEnvironmentContext {
    fn writeln(&mut self, output: &str) {
        self.out.write(output.as_ref()).expect("Cannot write to output stream!");
        self.out.write(b"\n").expect("Cannot write to output stream!");
        self.out.flush().expect("Flush error");
    }

    fn is_stop_signalled(&self) -> bool {
        self.stop_signal.load(Ordering::SeqCst)
        // false
    }

    fn set_stop_signal(&self, new_stop_signal_value: bool) {
        self.stop_signal.store(new_stop_signal_value, Ordering::SeqCst);
    }
}

pub struct StringEnvironmentContext {
    string_buffer: String,
}

impl StringEnvironmentContext {
    pub fn new() -> Self {
        Self {
            string_buffer: String::new()
        }
    }
}

impl EnvironmentContext for StringEnvironmentContext {
    fn writeln(&mut self, output: &str) {
        self.string_buffer.push_str(output);
        self.string_buffer.push('\n');
    }

    fn is_stop_signalled(&self) -> bool {
        false
    }

    fn set_stop_signal(&self, _new_stop_signal_value: bool) {
    }
}

impl ToString for StringEnvironmentContext {
    fn to_string(&self) -> String {
        self.string_buffer.clone()
    }
}

#[derive(Debug, Clone)]
pub struct EngineOptions {
    pub log_filename: Option<String>,
    pub multi_pv: u8,
    // threads: Option<u8>,
}

impl EngineOptions {
    pub fn for_filename(log_filename: Option<String>) -> EngineOptions {
        Self {
            log_filename,
            multi_pv: 1
        }
    }
}

impl EngineOptions {
    pub fn default() -> Self {
        EngineOptions { log_filename: None, multi_pv: 1 }
    }

    pub(crate) fn set_option(&mut self, name: &str, value: &str) {
        match name {
            "MultiPV" => {
                if let Ok(val) = value.parse::<u8>() {
                    self.multi_pv = val;
                }
            }
            _ => { /* Handle other options here */ }
        }
    }
}

pub struct Engine {
    pub position: BoardPosition,
    pub search: Search,
    file: Option<File>,
    engine_options: EngineOptions,
}

impl Engine {
    pub fn new(engine_options: EngineOptions, environment_context: Box<dyn EnvironmentContext>) -> Self {
        let file = engine_options.log_filename.clone().map(|filename| File::create(filename).unwrap());
        let transposition_table = TranspositionTable::new(1);
        let search = Search::new(transposition_table, environment_context);

        let engine = Engine {
            position: BoardPosition::from_fen(START_POS),
            search,
            file,
            engine_options,
        };

        engine
    }

    fn get_board_state(&self) -> &BoardState {
        &self.position.state
    }

    pub fn process_uci_command(&mut self, uci_command: String/*, output_adapter: &mut dyn OutputAdapter*/) {
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
                "stop" => {
                    // has alrady been handled via signal, just let through
                },
                "uci" => {
                    self.writeln(&*format!(r#"id name {}
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
                        self.writeln(&*result);
                    } else {
                        let search_limit = search_limit_params.prepare(self.position.state.side_to_play, &self.engine_options);
                        // println!("search_limit: {:?}", search_limit);
                        let result = self.search.it_deep(&self.position, search_limit);
                        self.writeln(&*format!("bestmove {}", result.moov.map(|m| m.uci()).unwrap_or(String::from("(none)"))));
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
                    self.writeln(&*output);
                }

                "isready" => {
                    self.writeln(&"readyok");
                },

                "quit" => {
                    self.writeln("quitting");
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

                "setoption" => {
                    if parts.len() != 5 || parts[1] != "name" || parts[3] != "value" {
                        let result = format!("Expecting 4 arguments, in form name XXXXX value YYY, got: {}", parts[1..].to_vec().join(" "));
                        self.writeln(&*result);
                    }

                    self.engine_options.set_option(parts[2], parts[4]);
                },

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
                    self.writeln(&*result);
                }
            };
        } else {
            self.writeln("Empty command");
        };
    }

    fn writeln(&mut self, output: &str) {
        self.search.environment_context.writeln(output);
    }

    pub fn uci_new_game(&mut self) {
        self.uci_new_game_from_fen(START_POS);
    }

    pub fn uci_new_game_from_fen(&mut self, fen: &str) {
        self.search.transposition_table.clear();
        self.position = BoardPosition::from_fen(fen);
    }

    pub fn parse_pgn_moves(&self, pgn_moves: &str) -> Vec<String> {
        self.get_board_state().parse_pgn_moves(pgn_moves)
        // let mut state = self.get_board_state().clone();
        // let mut move_vec= Vec::new();
        // let moves = pgn_moves.split_whitespace();
        // for moov in moves {
        //     let uci = Pgn::one_san_to_uci(moov, &state);
        //     let parsed_move = Move::from_uci_string(&uci, &state);
        //     state = state.do_move_no_history(&parsed_move);
        //     move_vec.push(uci);
        // }
        // move_vec
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


#[cfg(test)]
mod tests {
    use crate::engine::{Engine, EngineOptions, StdOutEnvironmentContext};

    #[test]
    fn parse_pgn_moves() {
        let options = EngineOptions::default();
        let engine = Engine::new(options, Box::new(StdOutEnvironmentContext::new()));
        let moves = engine.parse_pgn_moves("d4 Nf6 c4 e6 Nc3 Bb4 e3 O-O Bd3 c5");
        for moov in moves {
            println!("{}", moov);
        }
    }

    // #[test]
    // fn from_failing_sts() {
    //     let state = Fen::from_fen_default("2r5/p3k1p1/1p5p/4Pp2/1PPnK3/PB1R2P1/7P/8 w - f6 0 4");
    //     let moves = state.generate_legal_moves();
    //     println!("{}", moves);
    // }
    //
    // #[test]
    // fn failing_cute_chess() {
    //     let state = Fen::from_fen_default("rnbqkbnr/pppp2pp/5p2/8/5P2/8/PPP1PPPP/RN1QKBNR w KQkq - 0 4");
    //     let moves = state.generate_legal_moves();
    //     let tt = TranspositionTable::new(1);
    //     for moov in moves.over_sorted(&state, &tt) {
    //         println!("{}", moov.uci());
    //     }
    //     println!("{}", moves);
    // }
    //
    // #[test]
    // fn mg_value_test() {
    //     let mut state = Fen::from_fen_default("8/8/8/8/8/8/8/8 w KQkq - 0 1");
    //     assert_eq!(state.mg, 0);
    //     state.set_piece_at(WHITE_ROOK, Square::get_square_from_name("d5") as usize);
    //     assert_eq!(state.mg, 482);
    //     state.set_piece_at(BLACK_ROOK, Square::get_square_from_name("d2") as usize);
    //     assert_eq!(state.mg, -20);
    // }
}
