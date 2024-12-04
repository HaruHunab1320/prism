use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::stdlib::Module;
use crate::types::Value;
use std::sync::Arc;

pub fn create_medical_module() -> Module {
    let mut module = Module::new("medical");

    module.register_function("validate_symptom", Value::NativeFunction(Arc::new(|_interpreter: &mut Interpreter, args: Vec<Value>| {
        if args.len() != 1 {
            return Err(RuntimeError::TypeError("validate_symptom takes exactly 1 argument".to_string()));
        }
        let symptom = match &args[0] {
            Value::String(s) => s.clone(),
            _ => return Err(RuntimeError::TypeError("validate_symptom requires a string argument".to_string())),
        };
        
        // For now, just return a confidence score based on symptom length
        let confidence = (symptom.len() as f64).min(10.0) / 10.0;
        Ok(Value::Float(confidence))
    })));

    module.register_function("semantic_match", Value::NativeFunction(Arc::new(|_interpreter: &mut Interpreter, args: Vec<Value>| {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError("semantic_match takes exactly 2 arguments".to_string()));
        }
        let (symptom, disease) = match (&args[0], &args[1]) {
            (Value::String(s), Value::String(d)) => (s.clone(), d.clone()),
            _ => return Err(RuntimeError::TypeError("semantic_match requires two string arguments".to_string())),
        };
        
        // For now, just return a simple match score
        let score = if symptom.contains(&disease) || disease.contains(&symptom) {
            0.8
        } else {
            0.2
        };
        Ok(Value::Float(score))
    })));

    module
} 