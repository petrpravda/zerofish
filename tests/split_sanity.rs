//! Sanity checks of the incremental rebuild; each test arrives with the feature it covers.

use zerofish::position::BoardPosition;
use zerofish::search::{Search, SearchResult, INF};

fn position(fen: &str) -> BoardPosition {
    BoardPosition::from_fen(fen).unwrap()
}

fn search_with(position: &BoardPosition, depth: i32) -> SearchResult {
    Search::new().it_deep(position.state(), depth)
}

fn search(fen: &str, depth: i32) -> SearchResult {
    search_with(&position(fen), depth)
}

#[test]
fn returns_legal_move_without_changing_root() {
    let start = position("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let result = search_with(&start, 2);
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
