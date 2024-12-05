use super::Module;
use crate::types::Value;
use std::sync::Arc;
use std::error::Error;

pub fn create_llm_module() -> Module {
    let mut module = Module::new("llm");

    module.register_function("complete", Value::AsyncFn(Arc::new(|args: Vec<Value>| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "complete takes exactly 1 argument")) as Box<dyn Error>);
            }
            let prompt = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidInput, "complete requires a string argument")) as Box<dyn Error>),
            };
            
            // TODO: Implement actual LLM completion
            Ok(Value::String(format!("Completion for: {}", prompt)))
        })
    })));

    module
} 