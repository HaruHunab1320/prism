pub mod core;
pub mod llm;
pub mod medical;
pub mod utils;

use crate::error::Result;
use crate::value::Value;

pub async fn create_stdlib() -> Result<Value> {
    let mut modules = Vec::new();
    
    // Create core module
    modules.push(("core", core::create_core_module()?));
    
    // Create llm module
    modules.push(("llm", llm::create_llm_module()?));
    
    // Create medical module
    modules.push(("medical", medical::create_medical_module()?));
    
    // Create utils module
    modules.push(("utils", utils::create_utils_module()?));
    
    // Return all modules
    Ok(Value::new(crate::value::ValueKind::Object(std::sync::Arc::new(modules))))
}
