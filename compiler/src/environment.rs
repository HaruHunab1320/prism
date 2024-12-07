use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::error::Error;
use crate::value::Value;
use crate::error::{PrismError, Result};
use crate::module::Module;

#[derive(Debug)]
pub struct Environment {
    values: RwLock<HashMap<String, Value>>,
    enclosing: Option<Arc<Environment>>,
    current_module: Option<Arc<RwLock<Module>>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: RwLock::new(HashMap::new()),
            enclosing: None,
            current_module: None,
        }
    }

    pub fn with_enclosing(enclosing: Arc<Environment>) -> Self {
        Self {
            values: RwLock::new(HashMap::new()),
            enclosing: Some(enclosing),
            current_module: None,
        }
    }

    pub fn with_module(module: Arc<RwLock<Module>>) -> Self {
        Self {
            values: RwLock::new(HashMap::new()),
            enclosing: None,
            current_module: Some(module),
        }
    }

    pub fn get_module(&self) -> Option<Arc<RwLock<Module>>> {
        self.current_module.clone().or_else(|| {
            self.enclosing.as_ref().and_then(|env| env.get_module())
        })
    }

    pub fn set_module(&mut self, module: Arc<RwLock<Module>>) {
        self.current_module = Some(module);
    }

    pub fn define(&self, name: String, value: Value) {
        self.values.write().unwrap().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(value) = self.values.read().unwrap().get(name) {
            Some(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get(name)
        } else {
            None
        }
    }

    pub fn assign(&self, name: &str, value: Value) -> Result<()> {
        if self.values.read().unwrap().contains_key(name) {
            self.values.write().unwrap().insert(name.to_string(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.assign(name, value)
        } else {
            Err(Box::new(PrismError::UndefinedVariable(name.to_string())) as Box<dyn Error + Send + Sync>)
        }
    }

    pub fn get_at(&self, distance: usize, name: &str) -> Result<Value> {
        if distance == 0 {
            Ok(self.values
                .read()
                .unwrap()
                .get(name)
                .cloned()
                .ok_or_else(|| Box::new(PrismError::UndefinedVariable(name.to_string())) as Box<dyn Error + Send + Sync>)?)
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.get_at(distance - 1, name)
        } else {
            Err(Box::new(PrismError::UndefinedVariable(name.to_string())) as Box<dyn Error + Send + Sync>)
        }
    }

    pub fn assign_at(&self, distance: usize, name: &str, value: Value) -> Result<()> {
        if distance == 0 {
            if self.values.read().unwrap().contains_key(name) {
                self.values.write().unwrap().insert(name.to_string(), value);
                Ok(())
            } else {
                Err(Box::new(PrismError::UndefinedVariable(name.to_string())) as Box<dyn Error + Send + Sync>)
            }
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.assign_at(distance - 1, name, value)
        } else {
            Err(Box::new(PrismError::UndefinedVariable(name.to_string())) as Box<dyn Error + Send + Sync>)
        }
    }
}
