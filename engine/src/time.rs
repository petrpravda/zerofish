#![allow(dead_code, unused_imports)]

use std::convert::{TryInto};
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::time::SystemTime;
use crate::search::Search;


// use wasm_bindgen::prelude::*;
// use js_sys::Date;

// https://github.com/rust-lang/rust/issues/48564

// #[cfg(not(target_arch = "wasm32"))]
// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct Instant(Instant);

// #[cfg(not(target_arch = "wasm32"))]
// impl Instant {
//     pub fn now() -> Self { Self(Instant::now()) }
//     pub fn duration_since(&self, earlier: Instant) -> Duration { self.0.duration_since(earlier.0) }
//     pub fn elapsed(&self) -> Duration { self.0.elapsed() }
//     pub fn checked_add(&self, duration: Duration) -> Option<Self> { self.0.checked_add(duration).map(|i| Self(i)) }
//     pub fn checked_sub(&self, duration: Duration) -> Option<Self> { self.0.checked_sub(duration).map(|i| Self(i)) }
// }
//
// impl Add<Duration> for Instant { type Output = Instant; fn add(self, other: Duration) -> Instant { self.checked_add(other).unwrap() } }
// impl Sub<Duration> for Instant { type Output = Instant; fn sub(self, other: Duration) -> Instant { self.checked_sub(other).unwrap() } }
// impl Sub<Instant>  for Instant { type Output = Duration; fn sub(self, other: Instant) -> Duration { self.duration_since(other) } }
// impl AddAssign<Duration> for Instant { fn add_assign(&mut self, other: Duration) { *self = *self + other; } }
// impl SubAssign<Duration> for Instant { fn sub_assign(&mut self, other: Duration) { *self = *self - other; } }


pub trait TimeCounter {
    fn current_time_millis() -> u64;
}

#[cfg(not(target_arch = "wasm32"))]
impl TimeCounter for Search {
    fn current_time_millis() -> u64 {
        let n = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis();
        (n & 0xFFFFFFFFFFFFFFFF) as u64
    }
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use js_sys::Date;

#[cfg(target_arch = "wasm32")]
impl TimeCounter for Search {
    fn current_time_millis() -> u64 {
        let date = Date::new_0();
        let time = date.get_time();
        (time * 1000.0) as u64
    }
}
