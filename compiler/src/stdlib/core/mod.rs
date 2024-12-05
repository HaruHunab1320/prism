use crate::types::Value;
use crate::interpreter::Interpreter;
use std::error::Error;

pub fn register_core_functions(interpreter: &mut Interpreter) {
    interpreter.register_native_function("len", |_, args| {
        if args.len() != 1 {
            return Err("len() takes exactly one argument".into());
        }

        match &args[0] {
            Value::Array(arr) => Ok(Value::Float(arr.len() as f64)),
            Value::String(s) => Ok(Value::Float(s.len() as f64)),
            Value::Object(obj) => Ok(Value::Float(obj.len() as f64)),
            _ => Err("len() requires array, string, or object argument".into()),
        }
    });

    interpreter.register_native_function("push", |_, args| {
        if args.len() != 2 {
            return Err("push() takes exactly two arguments".into());
        }

        match &args[0] {
            Value::Array(arr) => {
                let mut new_arr = arr.clone();
                new_arr.push(args[1].clone());
                Ok(Value::Array(new_arr))
            },
            _ => Err("push() requires array as first argument".into()),
        }
    });

    interpreter.register_native_function("keys", |_, args| {
        if args.len() != 1 {
            return Err("keys() takes exactly one argument".into());
        }

        match &args[0] {
            Value::Object(obj) => {
                let keys: Vec<Value> = obj.iter()
                    .map(|(k, _)| Value::String(k.clone()))
                    .collect();
                Ok(Value::Array(keys))
            },
            _ => Err("keys() requires object argument".into()),
        }
    });

    interpreter.register_native_function("values", |_, args| {
        if args.len() != 1 {
            return Err("values() takes exactly one argument".into());
        }

        match &args[0] {
            Value::Object(obj) => {
                let values: Vec<Value> = obj.iter()
                    .map(|(_, v)| v.clone())
                    .collect();
                Ok(Value::Array(values))
            },
            _ => Err("values() requires object argument".into()),
        }
    });
} 