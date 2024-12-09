use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::error::{PrismError, Result};
use crate::value::Value;

#[derive(Debug)]
pub struct Environment {
    values: HashMap<String, Value>,
    enclosing: Option<Arc<RwLock<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Arc<RwLock<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(enclosing),
        }
    }

    pub fn get_enclosing(&self) -> Option<Arc<RwLock<Environment>>> {
        self.enclosing.clone()
    }

    pub fn define(&mut self, name: String, value: Value) -> Result<()> {
        self.values.insert(name, value);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(value) = self.values.get(name) {
            Ok(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.read().get(name)
        } else {
            Err(PrismError::UndefinedVariable(name.to_string()))
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<()> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.write().assign(name, value)
        } else {
            Err(PrismError::UndefinedVariable(name.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ValueKind;

    #[test]
    fn test_environment() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::new(ValueKind::Number(42.0)))
            .unwrap();
        assert_eq!(
            env.get("x").unwrap().kind,
            ValueKind::Number(42.0)
        );
    }

    #[test]
    fn test_environment_with_enclosing() {
        let mut global = Environment::new();
        global
            .define("x".to_string(), Value::new(ValueKind::Number(42.0)))
            .unwrap();
        let global = Arc::new(RwLock::new(global));

        let mut local = Environment::with_enclosing(global);
        local
            .define("y".to_string(), Value::new(ValueKind::Number(24.0)))
            .unwrap();

        assert_eq!(
            local.get("x").unwrap().kind,
            ValueKind::Number(42.0)
        );
        assert_eq!(
            local.get("y").unwrap().kind,
            ValueKind::Number(24.0)
        );
    }

    #[test]
    fn test_environment_assign() {
        let mut env = Environment::new();
        env.define("x".to_string(), Value::new(ValueKind::Number(42.0)))
            .unwrap();
        env.assign("x", Value::new(ValueKind::Number(24.0)))
            .unwrap();
        assert_eq!(
            env.get("x").unwrap().kind,
            ValueKind::Number(24.0)
        );
    }

    #[test]
    fn test_environment_assign_enclosing() {
        let mut global = Environment::new();
        global
            .define("x".to_string(), Value::new(ValueKind::Number(42.0)))
            .unwrap();
        let global = Arc::new(RwLock::new(global));

        let mut local = Environment::with_enclosing(global.clone());
        local
            .assign("x", Value::new(ValueKind::Number(24.0)))
            .unwrap();

        assert_eq!(
            global.read().get("x").unwrap().kind,
            ValueKind::Number(24.0)
        );
    }
}
