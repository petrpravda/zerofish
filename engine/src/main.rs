extern crate core;

use env::args;
use std::{env};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use zerofish::{engine_thread, uci};
use zerofish::engine::{EngineOptions};
use zerofish::util::extract_parameter;

fn main() {
    let args_strings = args().collect::<Vec<String>>();
    let args: Vec<&str> = args_strings.iter().map(|s| s.as_ref()).collect();
    let log_filename = extract_parameter(&args, "--log");
    let engine_options = EngineOptions::for_filename(log_filename);
    env::set_var("RUST_BACKTRACE", "full");
    let stop_signal = Arc::new(AtomicBool::new(false));
    uci::start_uci_loop(&engine_thread::spawn_engine_thread(&engine_options, stop_signal.clone()).0, stop_signal.clone());
}
