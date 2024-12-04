use crate::types::{Value, RuntimeError};
use crate::context::Context;
use std::collections::HashMap;

pub struct Stdlib {
    functions: HashMap<String, Value>,
    context_stack: Vec<Context>,
}

impl Stdlib {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        
        // Confidence operations
        functions.insert("confidence".to_string(), Value::Function(
            vec!["value".to_string()],
            vec![],
        ));
        
        functions.insert("combine_confidence".to_string(), Value::Function(
            vec!["values".to_string()],
            vec![],
        ));
        
        functions.insert("decay_confidence".to_string(), Value::Function(
            vec!["value".to_string(), "rate".to_string()],
            vec![],
        ));

        // Context operations
        functions.insert("push_context".to_string(), Value::Function(
            vec!["name".to_string(), "confidence".to_string()],
            vec![],
        ));
        
        functions.insert("pop_context".to_string(), Value::Function(
            vec![],
            vec![],
        ));
        
        functions.insert("get_context".to_string(), Value::Function(
            vec![],
            vec![],
        ));

        // Pattern matching
        functions.insert("match_pattern".to_string(), Value::Function(
            vec!["pattern".to_string(), "value".to_string()],
            vec![],
        ));
        
        functions.insert("verify_pattern".to_string(), Value::Function(
            vec!["pattern".to_string(), "value".to_string(), "confidence".to_string()],
            vec![],
        ));

        Self {
            functions,
            context_stack: Vec::new(),
        }
    }

    pub fn get_function(&self, name: &str) -> Option<&Value> {
        self.functions.get(name)
    }

    pub fn call_builtin(&mut self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match name {
            "confidence" => {
                if args.len() != 1 {
                    return Err(RuntimeError::TypeError(
                        "confidence() requires exactly one argument".to_string()
                    ));
                }
                match &args[0] {
                    Value::Float(n) => Ok(Value::Float(*n)),
                    _ => Err(RuntimeError::TypeError(
                        "confidence() argument must be a number".to_string()
                    )),
                }
            }
            "combine_confidence" => {
                if args.len() != 1 {
                    return Err(RuntimeError::TypeError(
                        "combine_confidence() requires exactly one argument".to_string()
                    ));
                }
                match &args[0] {
                    Value::Array(values) => {
                        let confidences: Vec<f64> = values.iter()
                            .filter_map(|v| match v {
                                Value::Float(n) => Some(*n),
                                _ => None,
                            })
                            .collect();
                        if confidences.len() != values.len() {
                            return Err(RuntimeError::TypeError(
                                "combine_confidence() array must contain only numbers".to_string()
                            ));
                        }
                        let combined = confidences.iter().product::<f64>();
                        Ok(Value::Float(combined))
                    }
                    _ => Err(RuntimeError::TypeError(
                        "combine_confidence() argument must be an array".to_string()
                    )),
                }
            }
            "decay_confidence" => {
                if args.len() != 2 {
                    return Err(RuntimeError::TypeError(
                        "decay_confidence() requires exactly two arguments".to_string()
                    ));
                }
                match (&args[0], &args[1]) {
                    (Value::Float(value), Value::Float(rate)) => {
                        Ok(Value::Float(value * (1.0 - rate)))
                    }
                    _ => Err(RuntimeError::TypeError(
                        "decay_confidence() arguments must be numbers".to_string()
                    )),
                }
            }
            "push_context" => {
                if args.len() != 2 {
                    return Err(RuntimeError::TypeError(
                        "push_context() requires exactly two arguments".to_string()
                    ));
                }
                match (&args[0], &args[1]) {
                    (Value::String(name), Value::Float(confidence)) => {
                        let context = Context::new(
                            name.clone(),
                            1.0,
                            *confidence,
                            (0.0, 1.0)
                        ).map_err(|e| RuntimeError::TypeError(e.to_string()))?;
                        self.context_stack.push(context);
                        Ok(Value::Void)
                    }
                    _ => Err(RuntimeError::TypeError(
                        "push_context() arguments must be a string and a number".to_string()
                    )),
                }
            }
            "pop_context" => {
                if args.len() != 0 {
                    return Err(RuntimeError::TypeError(
                        "pop_context() takes no arguments".to_string()
                    ));
                }
                self.context_stack.pop()
                    .ok_or_else(|| RuntimeError::TypeError(
                        "No context to pop".to_string()
                    ))?;
                Ok(Value::Void)
            }
            "get_context" => {
                if args.len() != 0 {
                    return Err(RuntimeError::TypeError(
                        "get_context() takes no arguments".to_string()
                    ));
                }
                self.context_stack.last()
                    .map(|ctx| Value::String(ctx.name.clone()))
                    .ok_or_else(|| RuntimeError::TypeError(
                        "No active context".to_string()
                    ))
            }
            "match_pattern" => {
                if args.len() != 2 {
                    return Err(RuntimeError::TypeError(
                        "match_pattern() requires exactly two arguments".to_string()
                    ));
                }
                match (&args[0], &args[1]) {
                    (Value::String(pattern), value) => {
                        // Simple pattern matching for now
                        Ok(Value::Boolean(pattern == &value.to_string()))
                    }
                    _ => Err(RuntimeError::TypeError(
                        "match_pattern() first argument must be a string".to_string()
                    )),
                }
            }
            "verify_pattern" => {
                if args.len() != 3 {
                    return Err(RuntimeError::TypeError(
                        "verify_pattern() requires exactly three arguments".to_string()
                    ));
                }
                match (&args[0], &args[1], &args[2]) {
                    (Value::String(pattern), value, Value::Float(confidence)) => {
                        // Simple pattern verification for now
                        let matches = pattern == &value.to_string();
                        Ok(Value::Float(if matches { *confidence } else { 0.0 }))
                    }
                    _ => Err(RuntimeError::TypeError(
                        "verify_pattern() arguments must be a string, any value, and a number".to_string()
                    )),
                }
            }
            _ => Err(RuntimeError::TypeError(
                format!("Unknown builtin function: {}", name)
            )),
        }
    }
} 