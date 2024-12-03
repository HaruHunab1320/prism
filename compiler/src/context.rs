use crate::types::{Context, Confidence};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct ContextManager {
    contexts: HashMap<String, Context>,
    active_context: Option<String>,
    context_stack: Vec<String>,
    shared_state: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            active_context: None,
            context_stack: Vec::new(),
            shared_state: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_context(&mut self, context: Context) -> Result<(), String> {
        let name = context.name().to_string();
        if self.contexts.contains_key(&name) {
            return Err(format!("Context '{}' already exists", name));
        }
        self.contexts.insert(name, context);
        Ok(())
    }

    pub fn enter_context(&mut self, name: &str) -> Result<(), String> {
        if !self.contexts.contains_key(name) {
            return Err(format!("Context '{}' does not exist", name));
        }

        if let Some(current) = &self.active_context {
            self.context_stack.push(current.clone());
        }

        self.active_context = Some(name.to_string());
        Ok(())
    }

    pub fn exit_context(&mut self) -> Result<(), String> {
        match self.context_stack.pop() {
            Some(previous) => {
                self.active_context = Some(previous);
                Ok(())
            }
            None => {
                self.active_context = None;
                Ok(())
            }
        }
    }

    pub fn shift_context(&mut self, from: &str, to: &str) -> Result<(), String> {
        if !self.contexts.contains_key(from) {
            return Err(format!("Source context '{}' does not exist", from));
        }
        if !self.contexts.contains_key(to) {
            return Err(format!("Target context '{}' does not exist", to));
        }

        // Transfer shared state
        let state_to_transfer = {
            let shared = self.shared_state.lock().unwrap();
            shared.get(from).cloned()
        };

        if let Some(state) = state_to_transfer {
            let mut shared = self.shared_state.lock().unwrap();
            shared.insert(to.to_string(), state);
        }

        self.active_context = Some(to.to_string());
        Ok(())
    }

    pub fn validate_in_context(&self, confidence: &Confidence) -> bool {
        match &self.active_context {
            Some(name) => {
                if let Some(context) = self.contexts.get(name) {
                    context.validate_confidence(confidence)
                } else {
                    false
                }
            }
            None => false,
        }
    }

    pub fn share_data(&self, from: &str, to: &str, data: Vec<String>) -> Result<(), String> {
        if !self.contexts.contains_key(from) {
            return Err(format!("Source context '{}' does not exist", from));
        }
        if !self.contexts.contains_key(to) {
            return Err(format!("Target context '{}' does not exist", to));
        }

        let mut shared = self.shared_state.lock().unwrap();
        shared.insert(to.to_string(), data);
        Ok(())
    }

    pub fn get_shared_data(&self, context: &str) -> Option<Vec<String>> {
        let shared = self.shared_state.lock().unwrap();
        shared.get(context).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context(name: &str) -> Context {
        Context::new(
            name.to_string(),
            vec!["test".to_string()],
            0.7,
            vec!["source".to_string()],
        )
        .unwrap()
    }

    #[test]
    fn test_context_registration() {
        let mut manager = ContextManager::new();
        let context = create_test_context("test");
        
        assert!(manager.register_context(context).is_ok());
        assert!(manager.register_context(create_test_context("test")).is_err());
    }

    #[test]
    fn test_context_navigation() {
        let mut manager = ContextManager::new();
        manager.register_context(create_test_context("ctx1")).unwrap();
        manager.register_context(create_test_context("ctx2")).unwrap();

        assert!(manager.enter_context("ctx1").is_ok());
        assert!(manager.enter_context("ctx2").is_ok());
        assert!(manager.exit_context().is_ok());
        assert_eq!(manager.active_context, Some("ctx1".to_string()));
    }

    #[test]
    fn test_context_validation() {
        let mut manager = ContextManager::new();
        manager.register_context(create_test_context("test")).unwrap();
        manager.enter_context("test").unwrap();

        let high_conf = Confidence::new(0.8).unwrap();
        let low_conf = Confidence::new(0.6).unwrap();

        assert!(manager.validate_in_context(&high_conf));
        assert!(!manager.validate_in_context(&low_conf));
    }

    #[test]
    fn test_context_sharing() {
        let mut manager = ContextManager::new();
        manager.register_context(create_test_context("ctx1")).unwrap();
        manager.register_context(create_test_context("ctx2")).unwrap();

        let data = vec!["shared_data".to_string()];
        assert!(manager.share_data("ctx1", "ctx2", data.clone()).is_ok());
        assert_eq!(manager.get_shared_data("ctx2"), Some(data));
    }
} 