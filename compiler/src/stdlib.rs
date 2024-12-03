use crate::types::{Confidence, Context};
use crate::interpreter::Value;
use crate::llm::LLMClient;
use std::collections::HashMap;
use std::sync::Arc;

/// Standard library function type
pub type StdFunction = fn(Vec<Value>) -> Result<Value, String>;

/// Standard library definition
pub struct StandardLibrary {
    functions: HashMap<String, StdFunction>,
    llm_client: Option<Arc<LLMClient>>,
}

impl StandardLibrary {
    pub fn new() -> Self {
        let mut stdlib = Self {
            functions: HashMap::new(),
            llm_client: LLMClient::new().ok().map(Arc::new),
        };
        stdlib.register_core_functions();
        stdlib
    }

    fn register_core_functions(&mut self) {
        // Confidence operations
        self.register("confidence.combine", confidence_combine);
        self.register("confidence.max", confidence_max);
        self.register("confidence.min", confidence_min);
        self.register("confidence.average", confidence_average);

        // Pattern matching
        self.register("pattern.match", pattern_match);
        self.register("pattern.semantic_match", pattern_semantic_match);
        self.register("pattern.transform", pattern_transform);

        // Context operations
        self.register("context.create", context_create);
        self.register("context.validate", context_validate);

        // Verification
        self.register("verify.source", verify_source);
        self.register("verify.combine_sources", verify_combine_sources);
    }

    fn register(&mut self, name: &str, func: StdFunction) {
        self.functions.insert(name.to_string(), func);
    }

    pub fn call(&self, name: &str, args: Vec<Value>) -> Result<Value, String> {
        match self.functions.get(name) {
            Some(func) => func(args),
            None => Err(format!("Function '{}' not found in standard library", name)),
        }
    }

    pub fn llm_client(&self) -> Option<Arc<LLMClient>> {
        self.llm_client.clone()
    }
}

// Confidence Operations

fn confidence_combine(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("confidence.combine requires 2 arguments".to_string());
    }

    match (&args[0], &args[1]) {
        (Value::Confidence(c1), Value::Confidence(c2)) => {
            let combined = c1.value() * c2.value();
            Ok(Value::Confidence(Confidence::new(combined)?))
        }
        _ => Err("Invalid argument types for confidence.combine".to_string()),
    }
}

fn confidence_max(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("confidence.max requires at least 1 argument".to_string());
    }

    let mut max_conf: f64 = 0.0;
    let mut count = 0;

    for arg in args {
        match arg {
            Value::Confidence(conf) => {
                max_conf = max_conf.max(conf.value());
                count += 1;
            }
            _ => return Err("All arguments must be confidence values".to_string()),
        }
    }

    Ok(Value::Confidence(Confidence::new(max_conf)?))
}

fn confidence_min(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("confidence.min requires at least 1 argument".to_string());
    }

    let mut min_conf: f64 = 1.0;
    let mut count = 0;

    for arg in args {
        match arg {
            Value::Confidence(conf) => {
                min_conf = min_conf.min(conf.value());
                count += 1;
            }
            _ => return Err("All arguments must be confidence values".to_string()),
        }
    }

    Ok(Value::Confidence(Confidence::new(min_conf)?))
}

fn confidence_average(args: Vec<Value>) -> Result<Value, String> {
    if args.is_empty() {
        return Err("confidence.average requires at least 1 argument".to_string());
    }

    let mut sum = 0.0;
    let mut count = 0;

    for arg in args {
        match arg {
            Value::Confidence(conf) => {
                sum += conf.value();
                count += 1;
            }
            _ => return Err("All arguments must be confidence values".to_string()),
        }
    }

    let average = sum / count as f64;
    Ok(Value::Confidence(Confidence::new(average)?))
}

// Pattern Matching Operations

fn pattern_match(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("pattern.match requires 2 arguments: text and pattern".to_string());
    }

    match (&args[0], &args[1]) {
        (Value::String(text), Value::String(pattern)) => {
            let matches = text.contains(pattern);
            Ok(Value::Confidence(Confidence::new(if matches { 1.0 } else { 0.0 })?))
        }
        _ => Err("Invalid argument types for pattern.match".to_string()),
    }
}

fn pattern_semantic_match(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("pattern.semantic_match requires 2 arguments: text1 and text2".to_string());
    }

    match (&args[0], &args[1]) {
        (Value::String(text1), Value::String(text2)) => {
            let client = StandardLibrary::new().llm_client()
                .ok_or("LLM client not available")?;

            match client.blocking_semantic_similarity(text1, text2) {
                Ok(similarity) => Ok(Value::Confidence(Confidence::new(similarity as f64)?)),
                Err(e) => Err(format!("Semantic matching failed: {}", e)),
            }
        }
        _ => Err("Invalid argument types for pattern.semantic_match".to_string()),
    }
}

fn pattern_transform(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("pattern.transform requires 2 arguments: text and transformation".to_string());
    }

    match (&args[0], &args[1]) {
        (Value::String(text), Value::String(transform)) => {
            let client = StandardLibrary::new().llm_client()
                .ok_or("LLM client not available")?;

            match client.blocking_verify_statement(text, transform) {
                Ok(result) => Ok(Value::String(result.to_string())),
                Err(e) => Err(format!("Pattern transformation failed: {}", e)),
            }
        }
        _ => Err("Invalid argument types for pattern.transform".to_string()),
    }
}

// Context Operations

fn context_create(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 4 {
        return Err("context.create requires 4 arguments: name, vocabulary, confidence_threshold, validation_sources".to_string());
    }

    match (&args[0], &args[1], &args[2], &args[3]) {
        (Value::String(name), Value::Array(vocab), Value::Confidence(threshold), Value::Array(sources)) => {
            let vocabulary = vocab.iter().map(|v| {
                if let Value::String(s) = v {
                    Ok(s.clone())
                } else {
                    Err("Vocabulary must be array of strings".to_string())
                }
            }).collect::<Result<Vec<_>, _>>()?;

            let validation_sources = sources.iter().map(|s| {
                if let Value::String(src) = s {
                    Ok(src.clone())
                } else {
                    Err("Validation sources must be array of strings".to_string())
                }
            }).collect::<Result<Vec<_>, _>>()?;

            let context = Context::new(
                name.clone(),
                vocabulary,
                threshold.value(),
                validation_sources,
            )?;

            Ok(Value::Context(context))
        }
        _ => Err("Invalid argument types for context.create".to_string()),
    }
}

fn context_validate(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("context.validate requires 2 arguments: context and confidence".to_string());
    }

    match (&args[0], &args[1]) {
        (Value::Context(ctx), Value::Confidence(conf)) => {
            Ok(Value::Confidence(Confidence::new(
                if ctx.validate_confidence(conf) { 1.0 } else { 0.0 }
            )?))
        }
        _ => Err("Invalid argument types for context.validate".to_string()),
    }
}

// Verification Operations

fn verify_source(args: Vec<Value>) -> Result<Value, String> {
    if args.len() != 2 {
        return Err("verify.source requires 2 arguments: source and statement".to_string());
    }

    match (&args[0], &args[1]) {
        (Value::String(source), Value::String(statement)) => {
            let client = StandardLibrary::new().llm_client()
                .ok_or("LLM client not available")?;

            match client.blocking_verify_statement(statement, source) {
                Ok(confidence) => Ok(Value::Confidence(Confidence::new(confidence as f64)?)),
                Err(e) => Err(format!("Source verification failed: {}", e)),
            }
        }
        _ => Err("Invalid argument types for verify.source".to_string()),
    }
}

fn verify_combine_sources(args: Vec<Value>) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("verify.combine_sources requires at least 2 verification results".to_string());
    }

    let mut combined_confidence = 1.0;
    let mut count = 0;

    for arg in args {
        match arg {
            Value::Confidence(conf) => {
                combined_confidence *= conf.value();
                count += 1;
            }
            _ => return Err("All arguments must be confidence values".to_string()),
        }
    }

    let mean_confidence = combined_confidence.powf(1.0 / count as f64);
    Ok(Value::Confidence(Confidence::new(mean_confidence)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_operations() {
        let stdlib = StandardLibrary::new();
        
        // Test combine
        let args = vec![
            Value::Confidence(Confidence::new(0.8).unwrap()),
            Value::Confidence(Confidence::new(0.5).unwrap()),
        ];
        match stdlib.call("confidence.combine", args) {
            Ok(Value::Confidence(conf)) => assert_eq!(conf.value(), 0.4),
            _ => panic!("Expected confidence value"),
        }
        
        // Test max
        let args = vec![
            Value::Confidence(Confidence::new(0.8).unwrap()),
            Value::Confidence(Confidence::new(0.5).unwrap()),
        ];
        match stdlib.call("confidence.max", args) {
            Ok(Value::Confidence(conf)) => assert_eq!(conf.value(), 0.8),
            _ => panic!("Expected confidence value"),
        }
    }

    #[test]
    fn test_pattern_matching() {
        let stdlib = StandardLibrary::new();
        
        let args = vec![
            Value::String("Hello, world!".to_string()),
            Value::String("world".to_string()),
        ];
        match stdlib.call("pattern.match", args) {
            Ok(Value::Confidence(conf)) => assert_eq!(conf.value(), 1.0),
            _ => panic!("Expected confidence value"),
        }
    }

    #[test]
    fn test_context_operations() {
        let stdlib = StandardLibrary::new();
        
        let args = vec![
            Value::String("test".to_string()),
            Value::Array(vec![Value::String("vocab".to_string())]),
            Value::Confidence(Confidence::new(0.7).unwrap()),
            Value::Array(vec![Value::String("source".to_string())]),
        ];
        match stdlib.call("context.create", args) {
            Ok(Value::Context(_)) => assert!(true),
            _ => panic!("Expected context value"),
        }
    }
} 