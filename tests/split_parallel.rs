//! Lazy SMP sanity: helper threads return legal moves, honour stop requests and count their nodes.

use std::sync::Arc;

use zerofish::limits::Limits;
use zerofish::parallel::{Options, ParallelSearch, Selection};
use zerofish::position::BoardPosition;
use zerofish::search::MAX_PLY;
use zerofish::tt::TranspositionTable;

#[test]
fn helper_searches_return_legal_moves_and_stop() {
    let position = BoardPosition::from_fen("r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3").unwrap();
    for selection in [Selection::Main, Selection::Deepest, Selection::Vote] {
        let options = Options {
            threads: 3,
            skip_depths: true,
            defer_moves: false,
            shared_history: false,
            selection,
            nnue: false,
            ..Options::default()
        };
        let mut search = ParallelSearch::new(Arc::new(TranspositionTable::default()), options);
        search.reset_stop();
        let result = search.search(&position, 5, Limits::infinite());
        assert!(position.state().is_legal_move(result.mv.unwrap()));
        assert!(search.nodes() >= search.main().searched_nodes());
        search.reset_stop();
        let result = search.search(&position, MAX_PLY as i32 - 1, Limits::with_millis(0, 0));
        assert!(position.state().is_legal_move(result.mv.unwrap()));
    }
}
