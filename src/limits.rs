//! Search time limits: a soft budget checked between iterations and a hard one checked during search.

use std::time::Instant;

pub struct Limits {
    started: Instant,
    hard_nanos: i64,
    soft_nanos: i64,
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
        Limits { started: Instant::now(), hard_nanos: millis_to_nanos(hard_millis), soft_nanos: millis_to_nanos(soft_millis) }
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
}
