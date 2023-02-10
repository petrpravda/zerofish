use zerofish::engine::{Engine, EngineOptions};

use crate::web_worker_adapter::WebWorkerOutputAdapter;

pub struct EngineWrapper {
    pub engine: Engine,
    // pub(crate) output_adapter: WebWorkerOutputAdapter,
}

impl EngineWrapper {
    pub fn new(output_adapter: WebWorkerOutputAdapter) -> Self {
        let engine = Engine::new(EngineOptions{ log_filename: None },
                                 Box::new(output_adapter));
        Self {
            engine,
        }
    }
}
