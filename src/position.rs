use crate::board::BoardState;
use crate::fen::START_POS;
use crate::moves::Move;

/// A game: the current state plus the position keys of every state so far, for repetition detection.
#[derive(Clone)]
pub struct BoardPosition {
    state: BoardState,
    keys: Vec<u64>,
}

impl BoardPosition {
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let state = BoardState::from_fen(fen)?;
        Ok(BoardPosition { keys: vec![state.hash()], state })
    }

    pub fn start_pos() -> Self {
        Self::from_fen(START_POS).expect("start position")
    }

    pub fn state(&self) -> &BoardState {
        &self.state
    }

    /// Keys of all states of the game, the current one last.
    pub fn keys(&self) -> &[u64] {
        &self.keys
    }

    pub fn do_move(&mut self, mv: Move) {
        self.state = self.state.do_move(mv);
        self.keys.push(self.state.hash());
    }

    pub fn do_move_uci(&mut self, uci: &str) -> Result<(), String> {
        let mv = self.state.find_legal_move(uci).ok_or_else(|| format!("Illegal move {uci} in {}", self.state.to_fen()))?;
        self.do_move(mv);
        Ok(())
    }
}

/// Position-only repetition since the last irreversible move, at even distance (same side to move).
pub fn is_repetition(state: &BoardState, keys: &[u64]) -> bool {
    let key = state.hash();
    let len = keys.len();
    let mut distance = 2;
    let mut remaining = state.half_move_clock - 2;
    while distance < len && remaining >= 0 {
        if keys[len - 1 - distance] == key {
            return true;
        }
        distance += 2;
        remaining -= 2;
    }
    false
}

pub fn is_threefold_repetition(state: &BoardState, keys: &[u64]) -> bool {
    let key = state.hash();
    let len = keys.len();
    let mut count = 1;
    let mut distance = 1;
    let mut remaining = state.half_move_clock;
    while distance < len && remaining > 0 {
        if keys[len - 1 - distance] == key {
            count += 1;
            if count >= 3 {
                return true;
            }
        }
        distance += 1;
        remaining -= 1;
    }
    false
}

/// `keys` must end with the key of `state`.
pub fn is_rule_draw(state: &BoardState, keys: &[u64]) -> bool {
    if state.is_null_branch() {
        return false;
    }
    state.half_move_clock >= 100 || is_threefold_repetition(state, keys) || state.has_insufficient_material()
}

/// `keys` must end with the key of `state`.
pub fn is_search_draw(state: &BoardState, keys: &[u64]) -> bool {
    if state.is_null_branch() {
        return false;
    }
    state.half_move_clock >= 100 || is_repetition(state, keys) || state.has_insufficient_material()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_repetitions() {
        let mut position = BoardPosition::start_pos();
        for mv in ["g1f3", "g8f6", "f3g1", "f6g8"] {
            position.do_move_uci(mv).unwrap();
        }
        assert!(is_repetition(position.state(), position.keys()));
        assert!(!is_threefold_repetition(position.state(), position.keys()));
        for mv in ["g1f3", "g8f6", "f3g1", "f6g8"] {
            position.do_move_uci(mv).unwrap();
        }
        assert!(is_rule_draw(position.state(), position.keys()));
    }

    #[test]
    fn insufficient_material() {
        assert!(BoardState::from_fen("8/8/8/4k3/8/8/2B5/4K3 w - - 0 1").unwrap().has_insufficient_material());
        assert!(!BoardState::from_fen("8/8/8/4k3/8/8/2BB4/4K3 w - - 0 1").unwrap().has_insufficient_material());
        assert!(!BoardState::from_fen("8/8/8/4k3/8/8/2P5/4K3 w - - 0 1").unwrap().has_insufficient_material());
    }
}
