use std::fmt;
use std::collections::VecDeque;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContextError {
    #[error("Invalid confidence value: {0}")]
    InvalidConfidence(f64),
    #[error("Invalid confidence range")]
    InvalidRange,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub name: String,
    pub values: HashMap<String, f64>,
    pub confidence: f64,
    pub range: (f64, f64),
}

impl Context {
    pub fn new(name: String, base_confidence: f64, confidence: f64, range: (f64, f64)) -> Result<Self, ContextError> {
        if confidence < range.0 || confidence > range.1 {
            return Err(ContextError::InvalidConfidence(confidence));
        }
        if range.0 > range.1 {
            return Err(ContextError::InvalidRange);
        }
        Ok(Self {
            name,
            values: HashMap::new(),
            confidence: confidence * base_confidence,
            range,
        })
    }

    pub fn set(&mut self, key: String, value: f64) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<f64> {
        self.values.get(key).copied()
    }

    pub fn confidence(&self) -> f64 {
        self.confidence
    }

    pub fn merge(&mut self, other: &Context) {
        for (key, value) in &other.values {
            self.values.insert(key.clone(), *value);
        }
        self.confidence = self.confidence * other.confidence;
    }
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Context({}, confidence={}, values={:?})", 
            self.name, self.confidence, self.values)
    }
}

pub struct ContextManager {
    contexts: VecDeque<Context>,
    confidence_threshold: f64,
}

impl ContextManager {
    pub fn new(confidence_threshold: f64) -> Self {
        Self {
            contexts: VecDeque::new(),
            confidence_threshold,
        }
    }

    pub fn push_context(&mut self, context: Context) -> Result<(), String> {
        if !self.contexts.is_empty() {
            let current = self.contexts.back().unwrap();
            if current.confidence() < context.confidence() {
                return Err("Child context cannot have higher confidence than parent".into());
            }
        }
        self.contexts.push_back(context);
        Ok(())
    }

    pub fn pop_context(&mut self) -> Result<Context, String> {
        self.contexts.pop_back()
            .ok_or_else(|| "No context to pop".into())
    }

    pub fn current_context(&self) -> Option<&Context> {
        self.contexts.back()
    }

    pub fn confidence_threshold(&self) -> f64 {
        self.confidence_threshold
    }

    pub fn validate_confidence(&self, confidence: f64) -> bool {
        confidence <= self.confidence_threshold()
    }

    pub fn exit_context(&mut self) -> Result<(), String> {
        self.pop_context()?;
        Ok(())
    }

    pub fn enter_context(&mut self, name: String, confidence: f64) -> Result<(), String> {
        let context = Context::new(
            name,
            1.0,
            confidence,
            (0.0, 1.0),
        ).map_err(|e| e.to_string())?;
        self.push_context(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-10;

    fn assert_float_eq(a: f64, b: f64) {
        assert!((a - b).abs() < EPSILON, "Expected {} but got {}", b, a);
    }

    #[test]
    fn test_context_creation() {
        let ctx = Context::new(
            "test".to_string(),
            0.5,
            0.8,
            (0.0, 1.0)
        ).unwrap();
        assert_float_eq(ctx.confidence(), 0.4);
    }

    #[test]
    fn test_invalid_confidence() {
        let result = Context::new(
            "test".to_string(),
            0.5,
            1.5,
            (0.0, 1.0)
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_range() {
        let result = Context::new(
            "test".to_string(),
            0.5,
            0.8,
            (1.0, 0.0)
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_context_values() {
        let mut ctx = Context::new(
            "test".to_string(),
            0.5,
            0.8,
            (0.0, 1.0)
        ).unwrap();

        ctx.set("x".to_string(), 42.0);
        assert_eq!(ctx.get("x"), Some(42.0));
        assert_eq!(ctx.get("y"), None);
    }

    #[test]
    fn test_context_merge() {
        let mut ctx1 = Context::new(
            "test1".to_string(),
            0.5,
            0.8,
            (0.0, 1.0)
        ).unwrap();

        let mut ctx2 = Context::new(
            "test2".to_string(),
            0.5,
            0.7,
            (0.0, 1.0)
        ).unwrap();

        ctx1.set("x".to_string(), 42.0);
        ctx2.set("y".to_string(), 24.0);

        ctx1.merge(&ctx2);

        assert_eq!(ctx1.get("x"), Some(42.0));
        assert_eq!(ctx1.get("y"), Some(24.0));
        assert_float_eq(ctx1.confidence(), 0.14); // 0.4 * 0.35
    }

    #[test]
    fn test_context_manager_creation() {
        let manager = ContextManager::new(0.5);
        assert_eq!(manager.confidence_threshold(), 0.5);
        assert!(manager.current_context().is_none());
    }

    #[test]
    fn test_context_manager_push_pop() {
        let mut manager = ContextManager::new(0.5);
        
        let ctx = Context::new(
            "test".to_string(),
            0.5,
            0.8,
            (0.0, 1.0)
        ).unwrap();
        
        assert!(manager.push_context(ctx.clone()).is_ok());
        assert_eq!(manager.current_context().unwrap().name, "test");
        
        let popped = manager.pop_context().unwrap();
        assert_eq!(popped.name, "test");
        assert!(manager.current_context().is_none());
    }

    #[test]
    fn test_context_manager_confidence_validation() {
        let mut manager = ContextManager::new(0.5);
        
        assert!(manager.validate_confidence(0.3));
        assert!(!manager.validate_confidence(0.7));
        
        let ctx = Context::new(
            "test".to_string(),
            0.5,
            0.8,
            (0.0, 1.0)
        ).unwrap();
        
        assert!(manager.push_context(ctx).is_ok());
        assert!(manager.validate_confidence(0.3));
        assert!(!manager.validate_confidence(0.7));
    }
} 