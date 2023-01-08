use zerofish::engine::{Engine, EngineOptions};

pub struct EngineWrapper {
    pub engine: Engine,
}

impl EngineWrapper {
    pub fn new() -> Self {
        let engine = Engine::new(EngineOptions{ log_filename: None });
        Self {
            engine,
        }
    }
}
