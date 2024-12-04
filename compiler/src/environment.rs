use std::collections::HashMap;
use crate::types::Value;

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        false
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
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