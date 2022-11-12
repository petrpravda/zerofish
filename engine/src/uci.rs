//use crate::engine::UciMessage;
use std::io;
use std::sync::mpsc::Sender;
use crate::engine::UciMessage;
// use crate::options::parse_set_option;

const VERSION: &str = env!("CARGO_PKG_VERSION");
//const AUTHOR: &str = "";

pub fn start_uci_loop(tx: &Sender<UciMessage>) {
    println!("<0)))><  0fish Chess Engine v{}", VERSION);

    loop {
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        // TODO TBD stop
        // TODO TBD quit
        send_message(tx, UciMessage::UciCommand(line));
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
