use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use crate::engine::{Engine, EngineOptions, UciMessage};
// use crate::fen::{configure_command_line_options, START_POS};
// use crate::transposition_table::DEFAULT_SIZE_MB;

pub struct EngineThread {
    pub rx: Receiver<UciMessage>,
    pub engine: Engine,
}

impl EngineThread {
    // pub fn new_from_fen(rx: Receiver<UciMessage>, engine_options: &EngineOptions) -> Self {
    //     let engine = Engine::new_from_fen(engine_options);
    //     EngineThread {
    //         rx,
    //         engine,
    //     }
    // }

    pub fn new(rx: Receiver<UciMessage>, engine_options: EngineOptions) -> Self {
        let engine = Engine::new(engine_options);
        EngineThread {
            rx,
            engine,
        }
    }

    fn start_loop(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(msg) => {
                    if !self.handle_message(msg) {
                        break;
                    }
                }
                Err(err) => {
                    println!("Engine communication error: {:?}", err);
                    return;
                }
            }
        }
        println!("Exiting loop");
    }

    fn handle_message(&mut self, msg: UciMessage) -> bool {
        match msg {
            UciMessage::Stop => {
                println!("Stopping");
                self.engine.search.stopped = true;
            },
            UciMessage::UciCommand(uci_command) => {
                // println!("UciCommand: {}", uci_command);
                let _result = self.engine.process_uci_command(uci_command);
                //println!("UciCommand execution result:\n{}", result);

                if _result.eq(&"quitting") {
                    println!("Quitting");
                    return false;
                }
            }
        }
        true
    }
}

pub fn spawn_engine_thread(engine_options: &EngineOptions) -> (Sender<UciMessage>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<UciMessage>();
    let surviving_engine_options = engine_options.clone();

    let handle = thread::spawn(move || {
        let mut engine = EngineThread::new(rx, surviving_engine_options.clone());
        // configure_command_line_options(&mut engine.engine.board);
        engine.start_loop();
    });

    (tx, handle)
}

