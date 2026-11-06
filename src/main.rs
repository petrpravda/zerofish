use std::io::{self, BufRead};
use std::time::Instant;

use zerofish::board::BoardState;
use zerofish::fen::START_POS;
use zerofish::perft::{perft, perft_string};
use zerofish::uci::{Uci, BANNER};
use zerofish::{bitboard, nnue};

fn main() {
    let (backend, network, args) = nnue::cli_configuration(nnue::EvaluationBackend::StockfishNnue)
        .unwrap_or_else(|e| { eprintln!("{e}"); std::process::exit(2) });
    bitboard::init();
    match args.first().map(String::as_str) {
        Some("bench") => bench(),
        Some("perft") => {
            let depth = args.get(1).and_then(|d| d.parse().ok()).unwrap_or(5);
            let fen = if args.len() > 2 { args[2..].join(" ") } else { START_POS.to_string() };
            match BoardState::from_fen(&fen) {
                Ok(state) => println!("{}", perft_string(&state, depth)),
                Err(error) => {
                    eprintln!("{error}");
                    std::process::exit(2);
                }
            }
        }
        _ => {
            nnue::configure(backend, network.as_deref()).unwrap_or_else(|e| { eprintln!("{e}"); std::process::exit(2) });
            uci_loop()
        },
    }
}

fn uci_loop() {
    nnue::init();
    println!("{BANNER}");
    let mut uci = Uci::new();
    for line in io::stdin().lock().lines() {
        let Ok(line) = line else { break };
        if !uci.handle(&line) {
            return;
        }
    }
    // end of input acts as quit
    uci.stop_search();
}

/// Perft 6 from the start position, ten runs; the first is warm-up and excluded from the average.
fn bench() {
    let mut total_time = 0f64;
    let mut total_nodes = 0u64;
    for run in 0..10 {
        let state = BoardState::from_fen(START_POS).expect("start position");
        let start = Instant::now();
        let nodes = perft(&state, 6);
        let seconds = start.elapsed().as_secs_f64();
        println!("{nodes} nodes in {:.6} ms, nps: {:.6}", seconds * 1e3, nodes as f64 / seconds);
        if run > 0 {
            total_time += seconds;
            total_nodes += nodes;
        }
    }
    println!("AVG:\n{total_nodes} nodes in {:.6}, nps: {:.6}", total_time * 1e3, total_nodes as f64 / total_time);
}
