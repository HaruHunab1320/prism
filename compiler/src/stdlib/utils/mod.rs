use crate::value::Value;
use crate::interpreter::Interpreter;
use std::error::Error;

pub fn register_utils_functions(interpreter: &mut Interpreter) {
    interpreter.register_native_function("split", |_, args| {
        if args.len() != 2 {
            return Err("split takes exactly two arguments".into());
        }
        let (text, delimiter) = match (&args[0], &args[1]) {
            (Value::String(s), Value::String(d)) => (s.clone(), d.clone()),
            _ => return Err("split requires two string arguments".into()),
        };

        let parts: Vec<Value> = text.split(&delimiter)
            .map(|s| Value::String(s.to_string()))
            .collect();
        Ok(Value::Array(parts))
    });

    interpreter.register_native_function("join", |_, args| {
        if args.len() != 2 {
            return Err("join takes exactly two arguments".into());
        }
        let (array, delimiter) = match (&args[0], &args[1]) {
            (Value::Array(arr), Value::String(d)) => (arr.clone(), d.clone()),
            _ => return Err("join requires an array and a string argument".into()),
        };

        let strings: Result<Vec<String>, Box<dyn Error>> = array.iter()
            .map(|v| match v {
                Value::String(s) => Ok(s.clone()),
                _ => Err("join array must contain only strings".into()),
            })
            .collect();

        Ok(Value::String(strings?.join(&delimiter)))
    });

    interpreter.register_native_function("map", |interpreter, args| {
        if args.len() != 2 {
            return Err("map takes exactly two arguments".into());
        }
        let (array, function) = match (&args[0], &args[1]) {
            (Value::Array(arr), Value::NativeFunction(f)) => (arr.clone(), f.clone()),
            _ => return Err("map requires an array and a function argument".into()),
        };

        let mut result = Vec::new();
        for item in array {
            result.push(function(interpreter, vec![item])?);
        }
        Ok(Value::Array(result))
    });

    interpreter.register_native_function("filter", |interpreter, args| {
        if args.len() != 2 {
            return Err("filter takes exactly two arguments".into());
        }
        let (array, function) = match (&args[0], &args[1]) {
            (Value::Array(arr), Value::NativeFunction(f)) => (arr.clone(), f.clone()),
            _ => return Err("filter requires an array and a function argument".into()),
        };

        let mut result = Vec::new();
        for item in array {
            let predicate_result = function(interpreter, vec![item.clone()])?;
            if let Value::Boolean(true) = predicate_result {
                result.push(item);
            }
        }
        Ok(Value::Array(result))
    });

    interpreter.register_native_function("reduce", |interpreter, args| {
        if args.len() != 3 {
            return Err("reduce takes exactly three arguments".into());
        }
        let (array, function, initial) = match (&args[0], &args[1], &args[2]) {
            (Value::Array(arr), Value::NativeFunction(f), initial) => (arr.clone(), f.clone(), initial.clone()),
            _ => return Err("reduce requires an array, a function, and an initial value".into()),
        };

        let mut acc = initial;
        for item in array {
            acc = function(interpreter, vec![acc, item])?;
        }
        Ok(acc)
    });
} 