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
        let confidence = match symptom.as_str() {
            "fever" => 0.9,
            "cough" => 0.85,
            "fatigue" => 0.7,
            _ => 0.3,
        };
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
        let score = match (symptom.as_str(), disease.as_str()) {
            ("fever", "flu") => 0.8,
            ("fever", "covid") => 0.7,
            ("fever", "cold") => 0.6,
            ("cough", "flu") => 0.7,
            ("cough", "covid") => 0.9,
            ("cough", "cold") => 0.8,
            ("fatigue", "flu") => 0.6,
            ("fatigue", "covid") => 0.8,
            ("fatigue", "cold") => 0.5,
            _ => 0.2,
        };
        Ok(Value::Float(score))
    })));

    module.register_function("get_disease_pattern", Value::NativeFunction(Arc::new(|_interpreter: &mut Interpreter, args: Vec<Value>| {
        if args.len() != 1 {
            return Err(RuntimeError::TypeError("get_disease_pattern takes exactly 1 argument".to_string()));
        }
        let disease = match &args[0] {
            Value::String(d) => d.clone(),
            _ => return Err(RuntimeError::TypeError("get_disease_pattern requires a string argument".to_string())),
        };
        
        // Return predefined patterns
        let pattern = match disease.as_str() {
            "flu" => "fever, body aches, fatigue, cough",
            "covid" => "fever, cough, fatigue, loss of taste/smell",
            "cold" => "runny nose, cough, sore throat, mild fever",
            _ => "unknown pattern",
        };
        Ok(Value::String(pattern.to_string()))
    })));

    module
} 