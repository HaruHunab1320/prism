use crate::ast::Value;
use crate::interpreter::Interpreter;

pub fn register_core_functions(interpreter: &Interpreter) {
    interpreter.register_native_function("len", |_: &Interpreter, args: Vec<Value>| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("len() takes exactly one argument".into());
            }

            match &args[0] {
                Value::Array(arr) => Ok(Value::Number(arr.len() as f64)),
                Value::String(s) => Ok(Value::Number(s.len() as f64)),
                Value::Object(obj) => Ok(Value::Number(obj.len() as f64)),
                _ => Err("len() argument must be array, string, or object".into()),
            }
        })
    });

    interpreter.register_native_function("map", |interpreter: &Interpreter, args: Vec<Value>| {
        let interpreter = interpreter.clone();
        Box::pin(async move {
            if args.len() != 2 {
                return Err("map() takes exactly two arguments".into());
            }

            match (&args[0], &args[1]) {
                (Value::Array(arr), Value::AsyncFn(f)) => {
                    let mut new_arr = Vec::new();
                    for item in arr {
                        let result = f(&interpreter, vec![item.clone()]).await?;
                        new_arr.push(result);
                    }
                    Ok(Value::Array(new_arr))
                }
                _ => Err("map() first argument must be array, second must be function".into()),
            }
        })
    });

    interpreter.register_native_function("keys", |_: &Interpreter, args: Vec<Value>| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("keys() takes exactly one argument".into());
            }

            match &args[0] {
                Value::Object(obj) => {
                    let keys = obj.keys().map(|k| Value::String(k.clone())).collect();
                    Ok(Value::Array(keys))
                }
                _ => Err("keys() argument must be object".into()),
            }
        })
    });

    interpreter.register_native_function("values", |_: &Interpreter, args: Vec<Value>| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("values() takes exactly one argument".into());
            }

            match &args[0] {
                Value::Object(obj) => {
                    let values = obj.values().cloned().collect();
                    Ok(Value::Array(values))
                }
                _ => Err("values() argument must be object".into()),
            }
        })
    });
}
