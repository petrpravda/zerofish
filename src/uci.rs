//! UCI protocol front end. Commands run on the caller's thread, searches on a worker thread.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use crate::evaluation;
use crate::fen::START_POS;
use crate::limits::Limits;
use crate::nnue::Nnue;
use crate::parallel::{Options, ParallelSearch, Selection, SEARCH_STACK_BYTES};
use crate::perft::perft_string;
use crate::pgn::san;
use crate::position::BoardPosition;
use crate::search::MAX_PLY;
use crate::tt::TranspositionTable;
use crate::types::square_name;

pub const BANNER: &str = concat!("zerofish ", env!("CARGO_PKG_VERSION"), " by Petr Pravda");

const UCI_INFO: &str = concat!("id name zerofish ", env!("CARGO_PKG_VERSION"), "
id author Petr Pravda
option name Hash type spin default 16 min 1 max 512
option name Threads type spin default 1 min 1 max 64
option name SMP Lockless type check default true
option name SMP Skip Depths type check default false
option name SMP Defer Moves type check default false
option name SMP Shared History type check default false
option name SMP Select type combo default vote var main var deepest var vote
option name Move Overhead type spin default 20 min 0 max 5000
option name Evaluation type combo default NNUE var NNUE var Classical
option name UCI_Chess960 type check default false
uciok");

pub struct Uci {
    table: Arc<TranspositionTable>,
    position: BoardPosition,
    engine: Option<ParallelSearch>,
    worker: Option<JoinHandle<ParallelSearch>>,
    stop_flags: Vec<Arc<AtomicBool>>,
    move_overhead: i64,
    nnue: Option<Box<Nnue>>,
    options: Options,
    nnue_evaluation: bool,
    chess960: bool,
}

impl Default for Uci {
    fn default() -> Self {
        Self::new()
    }
}

fn check(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err("Expected true or false".to_string()),
    }
}

/// The non-negative number following `name`, or `fallback` when `name` is absent.
fn option(args: &[&str], name: &str, fallback: i64) -> Result<i64, String> {
    match args.iter().position(|a| *a == name) {
        None => Ok(fallback),
        Some(index) => {
            let text = args.get(index + 1).ok_or_else(|| format!("Missing value for {name}"))?;
            Ok(text.parse::<i64>().map_err(|e| format!("Bad value for {name}: {e}"))?.max(0))
        }
    }
}

fn parse_number<T: std::str::FromStr>(text: &str) -> Result<T, String>
where
    T::Err: std::fmt::Display,
{
    text.parse::<T>().map_err(|e| format!("For input string \"{text}\": {e}"))
}

impl Uci {
    pub fn new() -> Self {
        let table = Arc::new(TranspositionTable::default());
        let options = Options::default();
        let engine = ParallelSearch::new(Arc::clone(&table), options.clone());
        Uci {
            table,
            position: BoardPosition::start_pos(),
            engine: Some(engine),
            worker: None,
            stop_flags: Vec::new(),
            move_overhead: 20,
            nnue: Nnue::for_backend(crate::nnue::default_backend()).map(Box::new),
            options,
            nnue_evaluation: crate::nnue::default_backend() != crate::nnue::EvaluationBackend::Classical,
            chess960: false,
        }
    }

    /// Handles one input line; returns false once `quit` was received.
    pub fn handle(&mut self, line: &str) -> bool {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        let Some((&command, args)) = tokens.split_first() else { return true };
        if command == "quit" {
            self.stop_search();
            return false;
        }
        if let Err(error) = self.execute(command, args) {
            println!("info string Command failed: {error}");
        }
        true
    }

    fn execute(&mut self, command: &str, args: &[&str]) -> Result<(), String> {
        match command {
            "uci" => println!("{UCI_INFO}"),
            "isready" => println!("readyok"),
            "d" => {
                let state = self.position.state();
                let moves: Vec<String> = state.generate_legal_moves().iter().map(|m| m.uci()).collect();
                let checkers: Vec<String> = (0..64).filter(|sq| state.checkers() & 1 << sq != 0).map(square_name).collect();
                println!("{state}\nCheckers: {}\nLegal uci moves: {}\n", checkers.join(" "), moves.join(" "));
            }
            "position" => {
                self.stop_search();
                self.position = Self::parse_position(args, self.chess960)?;
            }
            "go" => {
                self.stop_search();
                if args.len() > 1 && args[0] == "perft" {
                    let depth: u32 = parse_number(args[1])?;
                    println!("{}", perft_string(self.position.state(), depth));
                    return Ok(());
                }
                let depth = option(args, "depth", MAX_PLY as i64 - 1)?.min(i32::MAX as i64) as i32;
                if depth < 1 {
                    return Err("depth must be positive".to_string());
                }
                let mut limits = Limits::infinite();
                if args.contains(&"movetime") {
                    let millis = (option(args, "movetime", 0)? - self.move_overhead).max(0);
                    limits = Limits::with_millis(millis, millis);
                } else if !args.contains(&"infinite") {
                    let white = self.position.state().side_to_play() == 0;
                    let remaining = option(args, if white { "wtime" } else { "btime" }, i64::MAX)?;
                    if remaining != i64::MAX {
                        let increment = option(args, if white { "winc" } else { "binc" }, 0)?;
                        limits = Limits::clock(remaining, increment, option(args, "movestogo", 30)?, self.move_overhead);
                    }
                }
                self.start_search(depth, limits);
            }
            "eval" => {
                self.stop_search();
                let state = self.position.state();
                let (mode, value) =
                    if self.nnue_evaluation { ("NNUE", self.nnue.as_mut().ok_or("No NNUE backend configured")?.evaluate(state, 0)) } else { ("Classical", evaluation::evaluate_state(state)) };
                println!("info string Evaluation {mode} {value}");
            }
            "stop" => self.stop_search(),
            "setoption" => {
                self.stop_search();
                self.set_option(args)?;
            }
            "ucinewgame" => {
                self.stop_search();
                self.table.reset();
                self.engine_mut().reset_histories();
                self.position = BoardPosition::from_fen_with_chess960(START_POS, self.chess960)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn parse_position(args: &[&str], chess960: bool) -> Result<BoardPosition, String> {
        if args.len() >= 2 && args[0] == "startpos" && args[1] == "movespgn" {
            let mut position = BoardPosition::from_fen_with_chess960(START_POS, chess960)?;
            let moves = san::decode(position.state(), &args[2..])?;
            println!("{}", moves.iter().map(|mv| mv.uci()).collect::<Vec<_>>().join(" "));
            for mv in moves {
                position.do_move(mv);
            }
            return Ok(position);
        }
        let moves_index = args.iter().position(|a| *a == "moves");
        let mut position = if args.first() == Some(&"startpos") {
            BoardPosition::from_fen_with_chess960(START_POS, chess960)?
        } else if args.len() >= 7 && args[0] == "fen" {
            BoardPosition::from_fen_with_chess960(&args[1..7].join(" "), chess960)?
        } else {
            return Err("Expected startpos or six-field FEN".to_string());
        };
        if let Some(index) = moves_index {
            for mv in &args[index + 1..] {
                position.do_move_uci(mv)?;
            }
        }
        Ok(position)
    }

    fn set_option(&mut self, args: &[&str]) -> Result<(), String> {
        let value_index = args.iter().position(|a| *a == "value");
        let (Some(value_index), true) = (value_index, args.len() >= 4 && args[0] == "name") else {
            return Err("Expected setoption name ... value ...".to_string());
        };
        if value_index < 2 {
            return Err("Expected setoption name ... value ...".to_string());
        }
        let name = args[1..value_index].join(" ");
        let raw = *args.get(value_index + 1).ok_or("Missing option value")?;
        match name.as_str() {
            "UCI_Chess960" => {
                self.chess960 = check(raw)?;
                self.table.reset();
                self.engine_mut().reset_histories();
                self.position = BoardPosition::from_fen_with_chess960(START_POS, self.chess960)?;
            }
            "Hash" => {
                self.table = Arc::new(TranspositionTable::new(parse_number(raw)?)?);
                let table = Arc::clone(&self.table);
                self.engine_mut().set_table(table);
            }
            "Threads" => {
                let threads: usize = parse_number(raw)?;
                if !(1..=64).contains(&threads) {
                    return Err("Threads must be 1..64".to_string());
                }
                self.options.threads = threads;
                self.rebuild();
            }
            "SMP Lockless" => {
                self.options.lockless = check(raw)?;
                self.rebuild();
            }
            "SMP Skip Depths" => {
                self.options.skip_depths = check(raw)?;
                self.rebuild();
            }
            "SMP Defer Moves" => {
                self.options.defer_moves = check(raw)?;
                self.rebuild();
            }
            "SMP Shared History" => {
                self.options.shared_history = check(raw)?;
                self.rebuild();
            }
            "SMP Select" => {
                self.options.selection = Selection::parse(raw).ok_or_else(|| format!("Unknown selection {raw}"))?;
                self.rebuild();
            }
            "Move Overhead" => self.move_overhead = parse_number::<i64>(raw)?.clamp(0, 5000),
            "Evaluation" => {
                self.nnue_evaluation = match raw {
                    "NNUE" if self.nnue.is_some() => true,
                    "Classical" => false,
                    _ => return Err("Evaluation must be NNUE or Classical".to_string()),
                };
                self.table.reset();
                self.rebuild();
            }
            _ => println!("info string Unknown option {name}"),
        }
        Ok(())
    }

    fn rebuild(&mut self) {
        self.options.nnue = self.nnue_evaluation;
        self.engine = Some(ParallelSearch::new(Arc::clone(&self.table), self.options.clone()));
    }

    /// The idle engine; only called after `stop_search`.
    fn engine_mut(&mut self) -> &mut ParallelSearch {
        if self.engine.is_none() {
            self.rebuild();
        }
        self.engine.as_mut().unwrap()
    }

    fn start_search(&mut self, depth: i32, limits: Limits) {
        let mut engine = match self.engine.take() {
            Some(engine) => engine,
            None => ParallelSearch::new(Arc::clone(&self.table), self.options.clone()),
        };
        engine.reset_stop();
        self.stop_flags = engine.stop_flags();
        let snapshot = self.position.clone();
        let spawned = thread::Builder::new().name("zerofish-search".to_string()).stack_size(SEARCH_STACK_BYTES).spawn(move || {
            let mut best = snapshot.state().generate_legal_moves().iter().next();
            match catch_unwind(AssertUnwindSafe(|| engine.search(&snapshot, depth, limits))) {
                Ok(result) => best = result.mv,
                Err(_) => println!("info string Search failed"),
            }
            println!("info string {} threads={} totalnodes={}", engine.main().selective_summary(), engine.options().threads, engine.nodes());
            println!("bestmove {}", best.map_or_else(|| "0000".to_string(), |m| m.uci()));
            engine
        });
        match spawned {
            Ok(handle) => self.worker = Some(handle),
            Err(error) => println!("info string Search failed: {error}"),
        }
    }

    /// Stops a running search and waits for its `bestmove`.
    pub fn stop_search(&mut self) {
        let Some(worker) = self.worker.take() else { return };
        self.stop_flags.iter().for_each(|stop| stop.store(true, Relaxed));
        match worker.join() {
            Ok(engine) => self.engine = Some(engine),
            Err(_) => self.rebuild(),
        }
    }
}
