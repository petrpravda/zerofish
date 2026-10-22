//! Selective-search heuristics are exercised: each counter arrives with its heuristic.

use std::sync::Arc;

use zerofish::limits::Limits;
use zerofish::ordering::MoveOrdering;
use zerofish::position::BoardPosition;
use zerofish::search::Search;
use zerofish::tt::TranspositionTable;

fn counters() -> std::collections::HashMap<String, u64> {
    let mut search = Search::new(Arc::new(TranspositionTable::default()), false, MoveOrdering::new());
    for fen in [
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
    ] {
        let position = BoardPosition::from_fen(fen).unwrap();
        let result = search.it_deep(position.state(), position.keys(), 9, Limits::infinite());
        assert!(position.state().is_legal_move(result.mv.unwrap()));
    }
    search.selective_summary().split_whitespace().filter_map(|pair| pair.split_once('=')).map(|(k, v)| (k.to_string(), v.parse().unwrap())).collect()
}

#[test]
fn reached_heuristics_are_exercised() {
    let counters = counters();
    assert!(counters["rfp"] > 0, "reverse-futility: {counters:?}");
    assert!(counters["razor"] > 0, "razoring: {counters:?}");
    assert!(counters["null"] > 0, "null-move: {counters:?}");
    assert!(counters["iir"] > 0, "internal-reduction: {counters:?}");
}
