use std::collections::HashMap;
use crate::types::Value;

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Environment) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.get(name) {
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            None
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            true
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.assign(name, value)
        } else {
            false
        }
    }

    pub fn get_environment_mut(&mut self, name: &str) -> Option<&mut Environment> {
        if let Some(Value::Object(_)) = self.values.get(name) {
            Some(self)
        } else if let Some(enclosing) = &mut self.enclosing {
            enclosing.get_environment_mut(name)
        } else {
            None
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