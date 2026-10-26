//! Search time limits: a soft budget checked between iterations and a hard one checked during search.

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

// The browser host supplies a monotonic clock; std::time has no WASM host clock.
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "zerofish")]
extern "C" { fn now_ms() -> f64; }
#[cfg(target_arch = "wasm32")]
struct Instant(f64);
#[cfg(target_arch = "wasm32")]
impl Instant {
    fn now() -> Self { Self(unsafe { now_ms() }) }
    fn elapsed(&self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(((unsafe { now_ms() } - self.0) / 1000.0).max(0.0))
    }
}

use crate::moves::Move;

pub struct Limits {
    started: Instant,
    hard_nanos: i64,
    soft_nanos: i64,
    base_soft_nanos: i64,
    adaptive: bool,
    previous_move: u16,
    previous_score: i32,
    stable_iterations: i32,
    completed_iteration: bool,
}

impl Default for Limits {
    fn default() -> Self {
        Self::infinite()
    }
}

fn millis_to_nanos(millis: i64) -> i64 {
    if millis >= i64::MAX / 1_000_000 {
        i64::MAX
    } else {
        millis.max(0) * 1_000_000
    }
}

impl Limits {
    pub fn infinite() -> Self {
        Self::with_millis(i64::MAX, i64::MAX)
    }

    pub fn with_millis(soft_millis: i64, hard_millis: i64) -> Self {
        Self::build(soft_millis, hard_millis, false)
    }

    fn build(soft_millis: i64, hard_millis: i64, adaptive: bool) -> Self {
        let soft = millis_to_nanos(soft_millis);
        Limits {
            started: Instant::now(),
            hard_nanos: millis_to_nanos(hard_millis),
            soft_nanos: soft,
            base_soft_nanos: soft,
            adaptive,
            previous_move: 0,
            previous_score: 0,
            stable_iterations: 0,
            completed_iteration: false,
        }
    }

    pub fn clock(remaining_millis: i64, increment_millis: i64, moves_to_go: i64, overhead_millis: i64) -> Self {
        let available = (remaining_millis - overhead_millis).max(0);
        let soft = available.min((available / moves_to_go.max(1) + increment_millis * 3 / 4).max(1));
        Self::build(soft, available.min(soft * 4), true)
    }

    /// Stretches the soft budget when the best move changes or the score swings, shrinks it when stable.
    pub fn on_iteration(&mut self, mv: Move, score: i32, root_node_share: f64) {
        if !self.adaptive {
            return;
        }
        let same_move = self.completed_iteration && self.previous_move == mv.bits();
        self.stable_iterations = if same_move { self.stable_iterations + 1 } else { 0 };
        let change = if self.completed_iteration { (score - self.previous_score).abs() } else { 0 };
        let stability = (1.15 - self.stable_iterations as f64 * 0.12).max(0.65);
        let score_factor = 1.0 + (change as f64 / 160.0).min(0.5);
        let node_factor = 1.35 - 0.6 * root_node_share.clamp(0.0, 1.0);
        self.soft_nanos = self.hard_nanos.min((self.base_soft_nanos as f64 * stability * score_factor * node_factor) as i64);
        self.previous_move = mv.bits();
        self.previous_score = score;
        self.completed_iteration = true;
    }

    pub fn expired(&self) -> bool {
        self.elapsed_nanos() >= self.hard_nanos
    }

    pub fn soft_expired(&self) -> bool {
        self.elapsed_nanos() >= self.soft_nanos
    }

    pub fn elapsed_nanos(&self) -> i64 {
        self.started.elapsed().as_nanos() as i64
    }

    pub fn soft_budget_nanos(&self) -> i64 {
        self.soft_nanos
    }

    pub fn hard_budget_nanos(&self) -> i64 {
        self.hard_nanos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_budgets() {
        let limits = Limits::clock(60_000, 1_000, 30, 20);
        assert_eq!(limits.soft_budget_nanos(), (59_980 / 30 + 750) * 1_000_000);
        assert_eq!(limits.hard_budget_nanos(), limits.soft_budget_nanos() * 4);
        let limits = Limits::clock(10, 0, 30, 20);
        assert_eq!(limits.hard_budget_nanos(), 0);
        assert!(Limits::infinite().hard_budget_nanos() == i64::MAX);
    }
}
