use std::rc::Rc;
use wasm_bindgen::JsValue;
use zerofish::engine::OutputAdapter;
// use wasm_bindgen::prelude::*;
use web_sys::Worker;

pub struct WebWorkerOutputAdapter {
    string_buffer: String,
    pub(crate) worker: Rc<Worker>,
}

impl WebWorkerOutputAdapter {
    pub fn new(worker: &Worker) -> Self {
        Self {
            string_buffer: String::new(),
            worker: Rc::new(worker.clone()),
        }
    }
}

impl OutputAdapter for WebWorkerOutputAdapter {
    fn writeln(&mut self, output: &str) {
        self.string_buffer.push_str(output);
        self.string_buffer.push('\n');

        let message = JsValue::from(output.to_string());
        match self.worker.post_message(&message) {
            Ok(_) => {
                // message was successfully posted
            }
            Err(error) => {
                // an error occurred, handle it here
                //console_error!("Error posting message: {:?}", error);
            }
        }
    }
}

impl ToString for WebWorkerOutputAdapter {
    fn to_string(&self) -> String {
        self.string_buffer.clone()
    }
}
