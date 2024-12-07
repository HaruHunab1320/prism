use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use std::error::Error;
use std::collections::HashMap;
use crate::error::{PrismError, Result};
use crate::value::{Value, ValueKind};

pub fn create_utils_module() -> Result<Value> {
    let mut module = Value::new(ValueKind::Object(Arc::new(HashMap::<String, Value>::new())));
    module.set_context("utils".to_string());
    
    let mut random_fn = Value::new(ValueKind::NativeFunction(Arc::new(|args: Vec<Value>| -> Pin<Box<dyn Future<Output = Result<Value>> + Send + Sync>> {
        Box::pin(async move {
            if !args.is_empty() {
                return Err(Box::new(PrismError::InvalidArgument(
                    "random() takes no arguments".to_string()
                )) as Box<dyn Error + Send + Sync>);
            }
            
            // For now, just return a fixed value since we don't have rand dependency
            Ok(Value::new(ValueKind::Number(0.5)))
        })
    })));
    random_fn.set_context("utils.random".to_string());
    
    let mut sleep_fn = Value::new(ValueKind::NativeFunction(Arc::new(|args: Vec<Value>| -> Pin<Box<dyn Future<Output = Result<Value>> + Send + Sync>> {
        Box::pin(async move {
            if args.len() != 1 {
                return Err(Box::new(PrismError::InvalidArgument(
                    "sleep() takes exactly one argument".to_string()
                )) as Box<dyn Error + Send + Sync>);
            }
            
            let duration = match &args[0].kind {
                ValueKind::Number(n) => *n,
                _ => return Err(Box::new(PrismError::InvalidArgument(
                    "sleep() argument must be a number".to_string()
                )) as Box<dyn Error + Send + Sync>),
            };
            
            if duration < 0.0 {
                return Err(Box::new(PrismError::InvalidArgument(
                    "sleep() duration must be non-negative".to_string()
                )) as Box<dyn Error + Send + Sync>);
            }
            
            tokio::time::sleep(tokio::time::Duration::from_secs_f64(duration)).await;
            Ok(Value::new(ValueKind::Nil))
        })
    })));
    sleep_fn.set_context("utils.sleep".to_string());
    
    Ok(module)
}
