use crate::ast::Value;
use crate::interpreter::Interpreter;
use std::sync::Arc;

pub fn register_utils_functions(interpreter: &Interpreter) {
    interpreter.register_native_function("print", |_: &Interpreter, args: Vec<Value>| {
        Box::pin(async move {
            for arg in args {
                println!("{:?}", arg);
            }
            Ok(Value::Null)
        })
    });

    interpreter.register_native_function("type_of", |_: &Interpreter, args: Vec<Value>| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("type_of() takes exactly one argument".into());
            }

            let type_name = match &args[0] {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
                Value::AsyncFn(_) => "function",
                Value::Pattern(_) => "pattern",
                Value::Wildcard => "wildcard",
            };

            Ok(Value::String(type_name.to_string()))
        })
    });
}
