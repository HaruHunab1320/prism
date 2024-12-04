use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::stdlib::Module;
use crate::types::Value;
use std::sync::Arc;

pub fn create_core_module() -> Module {
    let mut module = Module::new("core");

    module.register_function("to_string", Value::NativeFunction(Arc::new(|_interpreter: &mut Interpreter, args: Vec<Value>| {
        if args.len() != 1 {
            return Err(RuntimeError::TypeError("to_string takes exactly 1 argument".to_string()));
        }
        Ok(Value::String(args[0].to_string()))
    })));

    module.register_function("len", Value::NativeFunction(Arc::new(|_interpreter: &mut Interpreter, args: Vec<Value>| {
        if args.len() != 1 {
            return Err(RuntimeError::TypeError("len takes exactly 1 argument".to_string()));
        }
        match &args[0] {
            Value::Array(arr) => Ok(Value::Float(arr.len() as f64)),
            Value::String(s) => Ok(Value::Float(s.len() as f64)),
            _ => Err(RuntimeError::TypeError(format!("Cannot get length of {}", args[0].get_type()))),
        }
    })));

    module.register_function("keys", Value::NativeFunction(Arc::new(|_interpreter: &mut Interpreter, args: Vec<Value>| {
        if args.len() != 1 {
            return Err(RuntimeError::TypeError("keys takes exactly 1 argument".to_string()));
        }
        match &args[0] {
            Value::Object(fields) => {
                let keys: Vec<Value> = fields
                    .iter()
                    .map(|(k, _)| Value::String(k.clone()))
                    .collect();
                Ok(Value::Array(keys))
            }
            _ => Err(RuntimeError::TypeError(format!("Cannot get keys of {}", args[0].get_type()))),
        }
    })));

    module
} 