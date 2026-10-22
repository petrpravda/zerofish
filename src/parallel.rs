//! Lazy SMP: every thread searches the root with its own state and shares the transposition table.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering::Relaxed};
use std::sync::Arc;
use std::thread;

use crate::correction::CorrectionHistory;
use crate::limits::Limits;
use crate::nnue::Nnue;
use crate::ordering::MoveOrdering;
use crate::position::BoardPosition;
use crate::search::{is_score_checkmate, Search, SearchResult};
use crate::tt::TranspositionTable;

pub const SEARCH_STACK_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Main,
    Deepest,
    Vote,
}

impl Selection {
    pub fn parse(text: &str) -> Option<Selection> {
        match text.to_ascii_lowercase().as_str() {
            "main" => Some(Selection::Main),
            "deepest" => Some(Selection::Deepest),
            "vote" => Some(Selection::Vote),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Options {
    pub threads: usize,
    pub lockless: bool,
    pub skip_depths: bool,
    pub defer_moves: bool,
    pub shared_history: bool,
    pub selection: Selection,
    pub nnue: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options { threads: 1, lockless: true, skip_depths: false, defer_moves: false, shared_history: false, selection: Selection::Vote, nnue: true }
    }
}

pub struct ParallelSearch {
    table: Arc<TranspositionTable>,
    options: Options,
    searches: Vec<Search>,
    nodes: u64,
}

impl ParallelSearch {
    pub fn new(table: Arc<TranspositionTable>, options: Options) -> Self {
        table.set_lockless(options.lockless && options.threads > 1);
        let main_ordering = MoveOrdering::new();
        let shared_histories = MoveOrdering::sharing(&main_ordering);
        let mut main_ordering = Some(main_ordering);
        let main_correction = Arc::new(CorrectionHistory::new());
        let mut searches = Vec::with_capacity(options.threads);
        for index in 0..options.threads {
            let shared = false;
            let ordering = if index == 0 {
                main_ordering.take().unwrap()
            } else if shared {
                MoveOrdering::sharing(&shared_histories)
            } else {
                MoveOrdering::new()
            };
            let correction = if shared || index == 0 { Arc::clone(&main_correction) } else { Arc::new(CorrectionHistory::new()) };
            let nnue = options.nnue.then(|| Box::new(Nnue::new()));
            let mut search = Search::new(Arc::clone(&table), index == 0, nnue, ordering, correction);
            search.thread_index = index;
            search.owns_table_generation = false;
            searches.push(search);
        }
        ParallelSearch { table, options, searches, nodes: 0 }
    }

    pub fn main(&self) -> &Search {
        &self.searches[0]
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    pub fn nodes(&self) -> u64 {
        self.nodes
    }

    /// Stop flags of all threads, to stop a running search from another thread.
    pub fn stop_flags(&self) -> Vec<Arc<AtomicBool>> {
        self.searches.iter().map(Search::stop_flag).collect()
    }

    pub fn stop(&self) {
        self.searches.iter().for_each(Search::stop);
    }

    /// Clears stop requests; call before starting `search`.
    pub fn reset_stop(&self) {
        self.searches.iter().for_each(Search::reset_stop);
    }

    pub fn set_table(&mut self, table: Arc<TranspositionTable>) {
        table.set_lockless(self.options.lockless && self.options.threads > 1);
        for search in &mut self.searches {
            search.set_table(Arc::clone(&table));
        }
        self.table = table;
    }

    pub fn search(&mut self, position: &BoardPosition, depth: i32, limits: Limits) -> SearchResult {
        self.table.new_search();
        let state = position.state();
        let keys = position.keys();
        let (main, helpers) = self.searches.split_first_mut().expect("at least one thread");
        let helper_stops: Vec<_> = helpers.iter().map(Search::stop_flag).collect();
        let main_result = thread::scope(|scope| {
            let running: Vec<_> = helpers
                .iter_mut()
                .map(|helper| {
                    thread::Builder::new()
                        .name("zerofish-helper".to_string())
                        .stack_size(SEARCH_STACK_BYTES)
                        .spawn_scoped(scope, move || {
                            helper.it_deep(state, keys, depth, Limits::infinite());
                        })
                })
                .collect();
            let result = main.it_deep(state, keys, depth, limits);
            helper_stops.iter().for_each(|stop| stop.store(true, Relaxed));
            for helper in running {
                match helper {
                    Ok(handle) => {
                        if handle.join().is_err() {
                            println!("info string Helper search failed");
                        }
                    }
                    Err(error) => println!("info string Helper search failed: {error}"),
                }
            }
            result
        });
        self.nodes = self.searches.iter().map(Search::searched_nodes).sum();
        self.select(main_result)
    }

    fn select(&self, main_result: SearchResult) -> SearchResult {
        if self.searches.len() == 1 || self.options.selection == Selection::Main || main_result.mv.is_none() {
            return main_result;
        }
        let mut best = &self.searches[0];
        if self.options.selection == Selection::Deepest {
            for search in &self.searches {
                if search.completed_result().is_some() && search.completed_depth() > best.completed_depth() {
                    best = search;
                }
            }
        } else {
            let minimum = self.searches.iter().filter_map(|s| s.completed_result()).map(|r| r.score).min().unwrap_or(i32::MAX);
            let mut votes: HashMap<u16, i64> = HashMap::new();
            for search in &self.searches {
                if let Some(SearchResult { mv: Some(mv), score }) = search.completed_result() {
                    *votes.entry(mv.bits()).or_default() += (score as i64 - minimum as i64 + 14) * search.completed_depth() as i64;
                }
            }
            for search in &self.searches {
                let (Some(result), Some(current)) = (search.completed_result(), best.completed_result()) else { continue };
                let (Some(mv), Some(current_mv)) = (result.mv, current.mv) else { continue };
                let winning = is_score_checkmate(result.score) && result.score > 0;
                let better = if winning {
                    result.score > current.score
                } else {
                    !(is_score_checkmate(current.score) && current.score > 0) && votes[&mv.bits()] > votes[&current_mv.bits()]
                };
                if better {
                    best = search;
                }
            }
        }
        best.completed_result().unwrap_or(main_result)
    }

    pub fn reset_histories(&mut self) {
        self.searches.iter_mut().for_each(Search::reset_histories);
    }

    pub fn clear(&mut self) {
        self.table.reset();
        self.reset_histories();
    }
}
