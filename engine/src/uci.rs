//use crate::engine::UciMessage;
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use crate::engine::UciMessage;
// use crate::options::parse_set_option;

const VERSION: &str = env!("CARGO_PKG_VERSION");
//const AUTHOR: &str = "";

pub fn start_uci_loop(tx: &Sender<UciMessage>, stop_signal: Arc<AtomicBool>) {
    println!("<0)))><  zerofish Chess Engine v{}", VERSION);

    loop {
        let mut line = String::new();
        let reading_outcome = io::stdin()
            .read_line(&mut line);

        match reading_outcome {
            Ok(_) => {
                let quitting = line.starts_with("quit");
                let stopping = line.starts_with("stop");
                if stopping || quitting {
                    stop_signal.store(true, Ordering::SeqCst);
                }
                if quitting  {
                    break;
                }
                send_message(tx, UciMessage::UciCommand(line.clone()));
            }
            Err(error) => {
                eprintln!("Failed to read line: {}", error);
                break;
            }
        }
            //.expect("Failed to read line");
    }
}

// Sends a message to the engine
fn send_message(tx: &Sender<UciMessage>, msg: UciMessage) {
    match tx.send(msg) {
        Ok(_) => {},
        Err(err) => {
            eprintln!("could not send message to engine thread: {}", err);
        }
    }
}
