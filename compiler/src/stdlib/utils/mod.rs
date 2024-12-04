use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use crate::stdlib::Module;
use crate::types::Value;
use std::sync::Arc;

pub fn create_utils_module() -> Module {
    let mut module = Module::new("utils");

    module.register_function("parse_json", Value::NativeFunction(Arc::new(|_interpreter: &mut Interpreter, args: Vec<Value>| {
        if args.len() != 1 {
            return Err(RuntimeError::TypeError("parse_json takes exactly 1 argument".to_string()));
        }

        let json_str = match &args[0] {
            Value::String(s) => s,
            _ => return Err(RuntimeError::TypeError("parse_json requires a string argument".to_string())),
        };

        fn convert_json(value: serde_json::Value) -> Result<Value, RuntimeError> {
            match value {
                serde_json::Value::Null => Ok(Value::String("null".to_string())),
                serde_json::Value::Bool(b) => Ok(Value::Boolean(b)),
                serde_json::Value::Number(n) => Ok(Value::Float(n.as_f64().unwrap_or(0.0))),
                serde_json::Value::String(s) => Ok(Value::String(s)),
                serde_json::Value::Array(arr) => {
                    let mut values = Vec::new();
                    for v in arr {
                        values.push(convert_json(v)?);
                    }
                    Ok(Value::Array(values))
                }
                serde_json::Value::Object(obj) => {
                    let mut fields = Vec::new();
                    for (k, v) in obj {
                        fields.push((k, convert_json(v)?));
                    }
                    Ok(Value::Object(fields))
                }
            }
        }

        match serde_json::from_str(json_str) {
            Ok(json) => convert_json(json),
            Err(e) => Err(RuntimeError::TypeError(format!("Invalid JSON: {}", e))),
        }
    })));

    module.register_function("split", Value::NativeFunction(Arc::new(|_interpreter: &mut Interpreter, args: Vec<Value>| {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError("split takes exactly 2 arguments".to_string()));
        }

        let (text, separator) = match (&args[0], &args[1]) {
            (Value::String(t), Value::String(s)) => (t, s),
            _ => return Err(RuntimeError::TypeError("split requires two string arguments".to_string())),
        };

        let parts: Vec<Value> = text.split(separator)
            .map(|s| Value::String(s.trim().to_string()))
            .collect();

        Ok(Value::Array(parts))
    })));

    module.register_function("join", Value::NativeFunction(Arc::new(|_interpreter: &mut Interpreter, args: Vec<Value>| {
        if args.len() != 2 {
            return Err(RuntimeError::TypeError("join takes exactly 2 arguments".to_string()));
        }

        let (array, separator) = match (&args[0], &args[1]) {
            (Value::Array(arr), Value::String(sep)) => (arr, sep),
            _ => return Err(RuntimeError::TypeError("join requires an array and a string separator".to_string())),
        };

        let strings: Result<Vec<String>, RuntimeError> = array.iter()
            .map(|v| match v {
                Value::String(s) => Ok(s.clone()),
                _ => Err(RuntimeError::TypeError("join array must contain only strings".to_string())),
            })
            .collect();

        match strings {
            Ok(parts) => Ok(Value::String(parts.join(separator))),
            Err(e) => Err(e),
        }
    })));

    module
} 