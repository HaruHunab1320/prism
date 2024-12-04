use std::collections::HashMap;
use crate::types::Value;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.values.get(name).cloned()
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_scoping() {
        let mut env = Environment::new();

        // Global scope
        env.define("x".to_string(), Value::Float(1.0));
        assert_eq!(env.get("x"), Some(Value::Float(1.0)));

        // New scope
        env.push_scope();
        env.define("x".to_string(), Value::Float(2.0));
        assert_eq!(env.get("x"), Some(Value::Float(2.0)));

        // Pop scope
        env.pop_scope();
        assert_eq!(env.get("x"), Some(Value::Float(1.0)));
    }

    #[test]
    fn test_environment_assignment() {
        let mut env = Environment::new();

        // Define in global scope
        env.define("x".to_string(), Value::Float(1.0));

        // New scope
        env.push_scope();
        
        // Assignment should modify outer scope
        assert!(env.assign("x", Value::Float(2.0)));
        assert_eq!(env.get("x"), Some(Value::Float(2.0)));

        // Assignment to undefined variable should fail
        assert!(!env.assign("y", Value::Float(3.0)));
    }
} 