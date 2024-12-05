use crate::ast::Value;
use crate::interpreter::Interpreter;

pub fn register_core_functions(interpreter: &mut Interpreter) {
    interpreter.register_native_function("len", |_, args| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("len() takes exactly one argument".into());
            }
            match &args[0] {
                Value::List(arr) => Ok(Value::Number(arr.len() as f64)),
                Value::String(s) => Ok(Value::Number(s.len() as f64)),
                Value::Object(obj) => Ok(Value::Number(obj.len() as f64)),
                _ => Err("len() requires a list, string, or object argument".into()),
            }
        })
    });

    interpreter.register_native_function("push", |_, args| {
        Box::pin(async move {
            if args.len() != 2 {
                return Err("push() takes exactly two arguments".into());
            }
            match &args[0] {
                Value::List(arr) => {
                    let mut new_arr = arr.clone();
                    new_arr.push(args[1].clone());
                    Ok(Value::List(new_arr))
                }
                _ => Err("push() requires a list as its first argument".into()),
            }
        })
    });

    interpreter.register_native_function("keys", |_, args| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("keys() takes exactly one argument".into());
            }
            match &args[0] {
                Value::Object(obj) => {
                    let keys = obj.keys().map(|k| Value::String(k.clone())).collect();
                    Ok(Value::List(keys))
                }
                _ => Err("keys() requires an object argument".into()),
            }
        })
    });

    interpreter.register_native_function("values", |_, args| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("values() takes exactly one argument".into());
            }
            match &args[0] {
                Value::Object(obj) => {
                    let values = obj.values().cloned().collect();
                    Ok(Value::List(values))
                }
                _ => Err("values() requires an object argument".into()),
            }
        })
    });
}
