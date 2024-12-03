use crate::types::Context;
use std::collections::VecDeque;

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
            if current.confidence_threshold() < context.confidence_threshold() {
                return Err("Child context cannot have higher confidence threshold than parent".into());
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
        self.contexts.back()
            .map(|ctx| ctx.confidence_threshold())
            .unwrap_or(self.confidence_threshold)
    }

    pub fn validate_confidence(&self, confidence: f64) -> bool {
        confidence >= self.confidence_threshold()
    }

    pub fn exit_context(&mut self) -> Result<(), String> {
        self.pop_context()?;
        Ok(())
    }

    pub fn enter_context(&mut self, name: String, confidence_threshold: f64) -> Result<(), String> {
        let context = Context::new(
            name,
            Vec::new(),
            confidence_threshold,
            Vec::new(),
        )?;
        self.push_context(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_stack() {
        let mut manager = ContextManager::new(0.5);
        
        // Push first context
        let ctx1 = Context::new(
            "test1".into(),
            vec![],
            0.7,
            vec![],
        ).unwrap();
        assert!(manager.push_context(ctx1).is_ok());
        
        // Push second context with lower threshold
        let ctx2 = Context::new(
            "test2".into(),
            vec![],
            0.6,
            vec![],
        ).unwrap();
        assert!(manager.push_context(ctx2).is_ok());
        
        // Try to push context with higher threshold
        let ctx3 = Context::new(
            "test3".into(),
            vec![],
            0.8,
            vec![],
        ).unwrap();
        assert!(manager.push_context(ctx3).is_err());
        
        // Pop contexts
        assert!(manager.pop_context().is_ok());
        assert!(manager.pop_context().is_ok());
        assert!(manager.pop_context().is_err());
    }

    #[test]
    fn test_confidence_validation() {
        let mut manager = ContextManager::new(0.5);
        
        // Test base threshold
        assert!(manager.validate_confidence(0.6));
        assert!(!manager.validate_confidence(0.4));
        
        // Test with context
        let ctx = Context::new(
            "test".into(),
            vec![],
            0.7,
            vec![],
        ).unwrap();
        manager.push_context(ctx).unwrap();
        
        assert!(manager.validate_confidence(0.8));
        assert!(!manager.validate_confidence(0.6));
    }
} 