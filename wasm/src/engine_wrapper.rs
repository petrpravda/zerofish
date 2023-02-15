use zerofish::engine::{Engine, EngineOptions};

use crate::web_worker_adapter::WebWorkerEnvironmentContext;

pub struct EngineWrapper {
    pub engine: Engine,
    // pub(crate) output_adapter: WebWorkerOutputAdapter,
}

impl EngineWrapper {
    pub fn new(environment_context: WebWorkerEnvironmentContext) -> Self {
        let engine = Engine::new(EngineOptions::default(),
                                 Box::new(environment_context));
        Self {
            engine,
        }
    }
}
