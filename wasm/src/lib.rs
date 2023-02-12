mod engine_wrapper;
mod web_worker_adapter;

use std::rc::Rc;
use std::sync::Mutex;
use zerofish::engine::{OutputAdapter};
use wasm_bindgen::prelude::*;
use web_sys::{Worker};
use crate::web_worker_adapter::WebWorkerOutputAdapter;
use crate::engine_wrapper::{EngineWrapper};

#[wasm_bindgen]
pub struct EngineContext {
    pub(crate) wrapper: Rc<Mutex<EngineWrapper>>,
}

#[wasm_bindgen]
impl EngineContext {
    pub fn new(worker: &Worker, stop_signalling: &js_sys::Int32Array) -> Self {
        let mut output_adapter = WebWorkerOutputAdapter::new(worker, stop_signalling);
        output_adapter.writeln("Zerofish 0.1 64 WASM Singlethreaded");

        EngineContext{
            wrapper: Rc::new(Mutex::new(EngineWrapper::new(output_adapter))),
//            output_adapter,
        }
    }
}

#[wasm_bindgen(js_name = newEngineContext)]
pub fn new_engine_context(worker: &Worker, stop_signalling: &js_sys::Int32Array) -> EngineContext {
    EngineContext::new(worker, stop_signalling)
}

#[wasm_bindgen(js_name = executeUciCommand)]
pub fn execute_uci_command(uci_command: String, engine_context: &mut EngineContext) -> String {
    // let zzz = stop_signalling.ge
    //let int32_array = js_sys::Int32Array::new(&array_buffer);
    // let value = stop_signalling.get_index(0);

    // let mut output_adapter = WebWorkerOutputAdapter::new(&engine_context.worker);
    engine_context.wrapper.lock().unwrap().engine.process_uci_command(uci_command);

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

    //output_adapter.to_string()
    "TBD".to_string()
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
