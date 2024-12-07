// Core module implementation will go here

use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use std::error::Error;
use std::collections::HashMap;
use crate::error::{PrismError, Result};
use crate::value::{Value, ValueKind};

pub fn create_core_module() -> Result<Value> {
    let mut module = Value::new(ValueKind::Object(Arc::new(HashMap::<String, Value>::new())));
    module.set_context("core".to_string());
    
    let mut print_fn = Value::new(ValueKind::NativeFunction(Arc::new(|args: Vec<Value>| -> Pin<Box<dyn Future<Output = Result<Value>> + Send + Sync>> {
        Box::pin(async move {
            for arg in args {
                println!("{:?}", arg);
            }
            Ok(Value::new(ValueKind::Nil))
        })
    })));
    print_fn.set_context("core.print".to_string());
    
    let mut type_fn = Value::new(ValueKind::NativeFunction(Arc::new(|args: Vec<Value>| -> Pin<Box<dyn Future<Output = Result<Value>> + Send + Sync>> {
        Box::pin(async move {
            if args.len() != 1 {
                return Err(Box::new(PrismError::InvalidArgument(
                    "type() takes exactly one argument".to_string()
                )) as Box<dyn Error + Send + Sync>);
            }
            
            let type_name = match args[0].kind {
                ValueKind::Nil => "nil",
                ValueKind::Boolean(_) => "boolean",
                ValueKind::Number(_) => "number",
                ValueKind::String(_) => "string",
                ValueKind::Function { .. } => "function",
                ValueKind::NativeFunction(_) => "native_function",
                ValueKind::Object(_) => "object",
                ValueKind::Module(_) => "module",
            };
            
            Ok(Value::new(ValueKind::String(type_name.to_string())))
        })
    })));
    type_fn.set_context("core.type".to_string());
    
    let mut assert_fn = Value::new(ValueKind::NativeFunction(Arc::new(|args: Vec<Value>| -> Pin<Box<dyn Future<Output = Result<Value>> + Send + Sync>> {
        Box::pin(async move {
            if args.len() != 1 && args.len() != 2 {
                return Err(Box::new(PrismError::InvalidArgument(
                    "assert() takes one or two arguments".to_string()
                )) as Box<dyn Error + Send + Sync>);
            }
            
            let condition = args[0].is_truthy();
            let message = if args.len() == 2 {
                if let ValueKind::String(s) = &args[1].kind {
                    s.clone()
                } else {
                    return Err(Box::new(PrismError::InvalidArgument(
                        "assert() message must be a string".to_string()
                    )) as Box<dyn Error + Send + Sync>);
                }
            } else {
                "Assertion failed".to_string()
            };
            
            if !condition {
                return Err(Box::new(PrismError::InvalidOperation(message)) as Box<dyn Error + Send + Sync>);
            }
            
            Ok(Value::new(ValueKind::Nil))
        })
    })));
    assert_fn.set_context("core.assert".to_string());
    
    Ok(module)
}
