use crate::ast::Value;
use crate::interpreter::Interpreter;
use std::collections::HashMap;
pub mod core;
pub mod llm;
pub mod medical;
pub mod utils;
#[derive(Debug, Clone)]
pub struct Module {
    pub(crate) name: String,
    pub(crate) functions: HashMap<String, Value>,
}
impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: HashMap::new(),
        }
    }
    pub fn register_function(&mut self, name: &str, function: Value) {
        self.functions.insert(name.to_string(), function);
    }
    pub fn get_function(&self, name: &str) -> Option<&Value> {
        self.functions.get(name)
    }
}
pub fn register_all_functions(interpreter: &mut Interpreter) {
    core::register_core_functions(interpreter);
    utils::register_utils_functions(interpreter);
    llm::register_llm_functions(interpreter);
    medical::register_medical_functions(interpreter);
}
