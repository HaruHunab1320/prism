use std::sync::Arc;
use parking_lot::RwLock;
use crate::error::Result;
use crate::value::{Value, ValueKind};
use crate::module::Module;

pub mod core;
pub mod llm;
pub mod medical;
pub mod utils;

pub fn init_stdlib() -> Result<Vec<(&'static str, Value)>> {
    let mut modules = Vec::new();
    
    // Initialize each module and convert to Value
    let core_module = core::init_core_module()?;
    let llm_module = llm::init_llm_module()?;
    let medical_module = medical::init_medical_module()?;
    let utils_module = utils::init_utils_module()?;

    // Convert each module to a Value with the correct RwLock type
    let convert_module = |m: Arc<RwLock<Module>>| -> Value {
        Value::new(ValueKind::Module(m))
    };

    modules.push(("core", convert_module(core_module)));
    modules.push(("llm", convert_module(llm_module)));
    modules.push(("medical", convert_module(medical_module)));
    modules.push(("utils", convert_module(utils_module)));
    
    Ok(modules)
}
