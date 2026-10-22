//! Sanity checks of the incremental rebuild; each test arrives with the feature it covers.

use zerofish::limits::Limits;
use zerofish::position::BoardPosition;
use zerofish::search::{Search, SearchResult, INF};

fn position(fen: &str) -> BoardPosition {
    BoardPosition::from_fen(fen).unwrap()
}

fn search_with(position: &BoardPosition, depth: i32, limits: Limits) -> SearchResult {
    Search::new().it_deep(position.state(), position.keys(), depth, limits)
}

fn search(fen: &str, depth: i32) -> SearchResult {
    search_with(&position(fen), depth, Limits::infinite())
}

#[test]
fn returns_legal_move_without_changing_root() {
    let start = position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let result = search_with(&start, 2, Limits::infinite());
    assert!(start.state().is_legal_move(result.mv.unwrap()));
    assert_eq!(start.state().to_fen(), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
}

#[test]
fn finds_mate_in_one() {
    let result = search("6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1", 2);
    assert_eq!(result.mv.unwrap().uci(), "d1d8");
    assert_eq!(result.score, INF - 1);
}

#[test]
fn captures_free_material() {
    assert_eq!(search("4k3/8/8/3r4/4P3/8/8/4K3 w - - 0 1", 2).mv.unwrap().uci(), "e4d5");
}

#[test]
fn terminal_roots_distinguish_mate_and_stalemate() {
    assert_eq!(search("7k/6Q1/5K2/8/8/8/8/8 b - - 0 1", 2), SearchResult { mv: None, score: -INF });
    assert_eq!(search("7k/5K2/6Q1/8/8/8/8/8 b - - 0 1", 2), SearchResult { mv: None, score: 0 });
}

#[test]
fn deeper_iterations_stop_at_a_found_mate() {
    let result = search("6k1/5ppp/8/8/8/8/5PPP/3R2K1 w - - 0 1", 64);
    assert_eq!(result.mv.unwrap().uci(), "d1d8");
    assert_eq!(result.score, INF - 1);
}

#[test]
fn expired_search_and_pre_start_cancellation_keep_legal_fallback() {
    let start = position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    assert!(start.state().is_legal_move(search_with(&start, 100, Limits::with_millis(0, 0)).mv.unwrap()));
    let mut engine = Search::new();
    engine.stop();
    let result = engine.it_deep(start.state(), start.keys(), 100, Limits::infinite());
    assert!(start.state().is_legal_move(result.mv.unwrap()));
}

#[test]
fn avoids_losing_the_queen_beyond_the_depth_one_horizon() {
    assert_ne!(search("4k3/8/4p3/3p4/8/8/3Q4/4K3 w - - 0 1", 1).mv.unwrap().uci(), "d2d5");
}

#[test]
fn adjudicates_rule_draw_before_searching() {
    let drawn = position("4k3/8/8/8/8/8/8/3QK3 w - - 100 90");
    let result = search_with(&drawn, 3, Limits::infinite());
    assert_eq!(result.score, 0);
    assert!(drawn.state().is_legal_move(result.mv.unwrap()));
}
