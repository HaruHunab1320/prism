use crate::error::Result;
use crate::module::Module;
use parking_lot::RwLock;
use std::sync::Arc;

pub fn init_llm_module() -> Result<Arc<RwLock<Module>>> {
    let module = Arc::new(RwLock::new(Module::new("llm".to_string())));
    Ok(module)
}
