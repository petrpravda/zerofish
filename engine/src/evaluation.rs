use crate::board_state::BoardState;

pub struct Evaluation {

}

impl Evaluation {
    pub fn evaluate_state(state: &BoardState) -> i32 {
        let score = state.interpolated_score();
        return score * state.side_to_play.multiplicator() as i32;
    }
}
