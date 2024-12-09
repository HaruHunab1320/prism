use std::sync::Arc;
use parking_lot::RwLock;
use crate::error::Result;
use crate::module::Module;

pub fn init_medical_module() -> Result<Arc<RwLock<Module>>> {
    let module = Arc::new(RwLock::new(Module::new("medical".to_string())));
    Ok(module)
}
