mod engine_wrapper;
mod web_worker_adapter;

use std::rc::Rc;
use std::sync::Mutex;
use zerofish::engine::{StringOutputAdapter};
use wasm_bindgen::prelude::*;
use web_sys::Worker;
use crate::web_worker_adapter::WebWorkerOutputAdapter;
use crate::engine_wrapper::{EngineWrapper};

#[wasm_bindgen]
pub struct EngineContext {
    pub(crate) wrapper: Rc<Mutex<EngineWrapper>>,
    pub(crate) worker: Rc<Worker>,
}

#[wasm_bindgen]
impl EngineContext {
    pub fn new(worker: &Worker) -> Self {
//        let engine = Engine::new(EngineOptions{ log_filename: None });
        EngineContext{
            wrapper: Rc::new(Mutex::new(EngineWrapper::new())),
            worker: Rc::new(worker.clone()),
        }
    }
}

#[wasm_bindgen(js_name = newEngineContext)]
pub fn new_engine_context(worker: &Worker) -> EngineContext {
    EngineContext::new(worker)
}

#[wasm_bindgen(js_name = executeUciCommand)]
pub fn execute_uci_command(uci_command: String, engine_context: &EngineContext) -> String {
    let mut output_adapter = WebWorkerOutputAdapter::new(&engine_context.worker);
    engine_context.wrapper.lock().unwrap().engine.process_uci_command(uci_command, &mut output_adapter);

    // let message = JsValue::from(output_adapter.to_string());
    // // let result = engine_context.worker.post_message(&message);
    //
    // match engine_context.worker.post_message(&message) {
    //     Ok(_) => {
    //         // message was successfully posted
    //     }
    //     Err(error) => {
    //         // an error occurred, handle it here
    //         //console_error!("Error posting message: {:?}", error);
    //     }
    // }

    output_adapter.to_string()
}

// #[wasm_bindgen(js_name = sendMessage)]
// pub fn send_message() -> Result<(), JsValue> {
//     let msg = JsValue::from("Hello, abc123");
//     let window = web_sys::window().unwrap();
//     // let msg2: &str = "Hello, abc1234";
//     //window.post_message(&msg, msg2)?;
//     Ok(())
// }

// #[wasm_bindgen]
// pub fn post_message_to_worker(worker: &Worker) -> Result<(), JsValue> {
//     let message = JsValue::from("Hello from Rust!");
//     worker.post_message(&message)?;
//     Ok(())
// }

// #[wasm_bindgen]
// pub fn call_js_function(f: &Function) {
//     let this = JsValue::null();
//     let args = &[];
//     let _ = f.call0(&this, args);
// }
