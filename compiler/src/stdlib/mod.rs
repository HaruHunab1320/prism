pub mod core;
pub mod llm;
pub mod medical;
pub mod utils;

use std::error::Error;

pub fn register_all_functions() -> Result<(), Box<dyn Error + Send + Sync>> {
    core::register_core_functions()?;
    llm::register_llm_functions()?;
    medical::register_medical_functions()?;
    utils::register_utils_functions()?;
    Ok(())
}
