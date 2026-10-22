use crate::board::BoardState;
use crate::types::WHITE;

/// Classical evaluation: tapered piece-square tables, from the side to move's view.
pub fn evaluate_state(state: &BoardState) -> i32 {
    let score = state.interpolated_score();
    if state.side_to_play() == WHITE {
        score
    } else {
        -score
    }
}
