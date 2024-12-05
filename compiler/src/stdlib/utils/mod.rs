use crate::types::Value;
use crate::interpreter::Interpreter;
use std::error::Error;

pub fn register_utils_functions(interpreter: &mut Interpreter) {
    interpreter.register_native_function("split", |_, args| {
        if args.len() != 2 {
            return Err("split() takes exactly two arguments".into());
        }

        match (&args[0], &args[1]) {
            (Value::String(text), Value::String(sep)) => {
                let parts: Vec<Value> = text.split(sep.as_str())
                    .map(|s| Value::String(s.to_string()))
                    .collect();
                Ok(Value::Array(parts))
            },
            _ => Err("split() requires string arguments".into()),
        }
    });

    interpreter.register_native_function("join", |_, args| {
        if args.len() != 2 {
            return Err("join() takes exactly two arguments".into());
        }

        match (&args[0], &args[1]) {
            (Value::Array(arr), Value::String(sep)) => {
                let strings: Result<Vec<String>, Box<dyn Error>> = arr.iter()
                    .map(|v| match v {
                        Value::String(s) => Ok(s.clone()),
                        _ => Err("join() array elements must be strings".into()),
                    })
                    .collect();
                Ok(Value::String(strings?.join(sep.as_str())))
            },
            _ => Err("join() requires array and string arguments".into()),
        }
    });

    interpreter.register_native_function("map", |interpreter, args| {
        if args.len() != 2 {
            return Err("map() takes exactly two arguments".into());
        }

        match (&args[0], &args[1]) {
            (Value::Array(arr), Value::NativeFunction(f)) => {
                let mut results = Vec::new();
                for value in arr {
                    results.push(f(interpreter, vec![value.clone()])?);
                }
                Ok(Value::Array(results))
            },
            _ => Err("map() requires array and function arguments".into()),
        }
    });

    interpreter.register_native_function("filter", |interpreter, args| {
        if args.len() != 2 {
            return Err("filter() takes exactly two arguments".into());
        }

        match (&args[0], &args[1]) {
            (Value::Array(arr), Value::NativeFunction(f)) => {
                let mut results = Vec::new();
                for value in arr {
                    let predicate = f(interpreter, vec![value.clone()])?;
                    if let Value::Boolean(true) = predicate {
                        results.push(value.clone());
                    }
                }
                Ok(Value::Array(results))
            },
            _ => Err("filter() requires array and function arguments".into()),
        }
    });

    interpreter.register_native_function("reduce", |interpreter, args| {
        if args.len() != 3 {
            return Err("reduce() takes exactly three arguments".into());
        }

        match (&args[0], &args[1], &args[2]) {
            (Value::Array(arr), Value::NativeFunction(f), initial) => {
                let mut acc = initial.clone();
                for value in arr {
                    acc = f(interpreter, vec![acc, value.clone()])?;
                }
                Ok(acc)
            },
            _ => Err("reduce() requires array, function, and initial value arguments".into()),
        }
    });
} 