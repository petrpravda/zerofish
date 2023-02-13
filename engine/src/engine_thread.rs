use std::sync::atomic::AtomicBool;
use std::sync::{Arc, mpsc};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

use crate::engine::{Engine, EngineOptions, StdOutEnvironmentContext, UciMessage};
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

    pub fn new(rx: Receiver<UciMessage>, engine_options: EngineOptions, stop_signal: Arc<AtomicBool>) -> Self {
        let engine = Engine::new(engine_options, Box::new(StdOutEnvironmentContext::new_w_signal(stop_signal)));
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
        self.engine.search.environment_context.set_stop_signal(false);
        match msg {
            // UciMessage:: => {
            // },
            UciMessage::UciCommand(uci_command) => {
                // println!("UciCommand: {}", uci_command);
                let quit = uci_command.starts_with("quit");
                //let mut output_adapter = StdOutOutputAdapter::new();
                self.engine.process_uci_command(uci_command);
                //println!("UciCommand execution result:\n{}", result);

                if quit {
                    println!("Quitting");
                    return false;
                }
            }
        }
        true
    }
}

pub fn spawn_engine_thread(engine_options: &EngineOptions, stop_signal: Arc<AtomicBool>) -> (Sender<UciMessage>, JoinHandle<()>) {
    let (tx, rx) = mpsc::channel::<UciMessage>();
    let surviving_engine_options = engine_options.clone();

    let handle = thread::spawn(move || {
        let mut engine = EngineThread::new(rx, surviving_engine_options.clone(), stop_signal);
        // configure_command_line_options(&mut engine.engine.board);
        engine.start_loop();
    });

    (tx, handle)
}

