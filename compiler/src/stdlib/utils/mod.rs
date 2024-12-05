use crate::ast::Value;
use crate::interpreter::Interpreter;
use std::collections::HashMap;

pub fn register_utils_functions(interpreter: &mut Interpreter) {
    // Register utility functions
    interpreter.register_native_function("print", |_, args| {
        Box::pin(async move {
            for arg in args {
                println!("{:?}", arg);
            }
            Ok(Value::Null)
        })
    });

    interpreter.register_native_function("type_of", |_, args| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("type_of takes exactly one argument".into());
            }
            let type_name = match &args[0] {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::List(_) => "list",
                Value::Object(_) => "object",
                Value::AsyncFn(_) => "function",
            };
            Ok(Value::String(type_name.to_string()))
        })
    });
}
