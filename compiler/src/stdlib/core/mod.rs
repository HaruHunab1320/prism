// Core module implementation will go here

use std::sync::Arc;
use parking_lot::RwLock;
use crate::error::Result;
use crate::module::Module;
use crate::value::{Value, ValueKind};

pub fn init_core_module() -> Result<Arc<RwLock<Module>>> {
    let module = Arc::new(RwLock::new(Module::new("core".to_string())));

    // print function
    let print_fn = Value::new(ValueKind::NativeFunction {
        name: "print".to_string(),
        arity: 1,
        handler: Arc::new(|args| {
            if let Some(arg) = args.first() {
                println!("{:?}", arg);
            }
            Ok(Value::new(ValueKind::Nil))
        }),
    });

    // type function
    let type_fn = Value::new(ValueKind::NativeFunction {
        name: "type".to_string(),
        arity: 1,
        handler: Arc::new(|args| {
            if let Some(arg) = args.first() {
                let type_str = match &arg.kind {
                    ValueKind::Nil => "nil",
                    ValueKind::Boolean(_) => "boolean",
                    ValueKind::Number(_) => "number",
                    ValueKind::String(_) => "string",
                    ValueKind::Function { .. } => "function",
                    ValueKind::NativeFunction { .. } => "native_function",
                    ValueKind::Module(_) => "module",
                    ValueKind::List(_) => "list",
                    ValueKind::Map(_) => "map",
                };
                Ok(Value::new(ValueKind::String(type_str.to_string())))
            } else {
                Ok(Value::new(ValueKind::Nil))
            }
        }),
    });

    // assert function
    let assert_fn = Value::new(ValueKind::NativeFunction {
        name: "assert".to_string(),
        arity: 2,
        handler: Arc::new(|args| {
            if args.len() != 2 {
                return Ok(Value::new(ValueKind::Nil));
            }

            let condition = &args[0];
            let message = &args[1];

            match &condition.kind {
                ValueKind::Boolean(true) => Ok(Value::new(ValueKind::Nil)),
                _ => {
                    let message = match &message.kind {
                        ValueKind::String(s) => s.clone(),
                        _ => "Assertion failed".to_string(),
                    };
                    Err(crate::error::PrismError::RuntimeError(message))
                }
            }
        }),
    });

    {
        let mut module_guard = module.write();
        module_guard.export("print".to_string(), print_fn)?;
        module_guard.export("type".to_string(), type_fn)?;
        module_guard.export("assert".to_string(), assert_fn)?;
    }

    Ok(module)
}
