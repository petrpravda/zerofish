use crate::board_state::BoardState;
use crate::transposition::Value;

pub struct Evaluation {

}

impl Evaluation {
    pub fn evaluate_state(state: &BoardState) -> Value {
        let score = state.interpolated_score();
        return score * state.side_to_play.multiplicator() as Value;
    }
}
