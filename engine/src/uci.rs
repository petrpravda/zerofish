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
    println!("<0)))><  0fish Chess Engine v{}", VERSION);

    loop {
        let mut line = String::new();
        let ble = io::stdin()
            .read_line(&mut line);

        match ble {
            Ok(0) => {
                send_message(tx, UciMessage::Stop);
                println!("Bye");
                break;
            }
            Ok(_) => {
                send_message(tx, UciMessage::UciCommand(line.clone()));
                if line.starts_with("quit") {
                    break;
                }
                if line.starts_with("stop") {
                    println!("Stopping qwe");
                    stop_signal.store(true, Ordering::SeqCst);
                }
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
