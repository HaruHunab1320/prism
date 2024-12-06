use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::value::Value;
use crate::error::Error;

#[derive(Clone)]
pub struct Environment {
    values: Arc<RwLock<HashMap<String, Value>>>,
    pub enclosing: Option<Arc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: Arc::new(RwLock::new(HashMap::new())),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: Arc<Environment>) -> Self {
        Self {
            values: Arc::new(RwLock::new(HashMap::new())),
            enclosing: Some(enclosing),
        }
    }

    pub async fn define(&self, name: String, value: Value) {
        let mut values = self.values.write().await;
        values.insert(name, value);
    }

    pub async fn assign(&self, name: &str, value: Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut values = self.values.write().await;
        if values.contains_key(name) {
            values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            drop(values); // Release the write lock before recursive call
            Box::pin(enclosing.assign(name, value)).await
        } else {
            Err(Box::new(Error::new(&format!("Undefined variable '{}'.", name))))
        }
    }

    pub async fn get(&self, name: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let values = self.values.read().await;
        if let Some(value) = values.get(name) {
            Ok(value.clone())
        } else if let Some(enclosing) = &self.enclosing {
            drop(values); // Release the read lock before recursive call
            Box::pin(enclosing.get(name)).await
        } else {
            Err(Box::new(Error::new(&format!("Undefined variable '{}'.", name))))
        }
    }

    pub async fn get_at(&self, distance: usize, name: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        if distance == 0 {
            self.get(name).await
        } else if let Some(enclosing) = &self.enclosing {
            Box::pin(enclosing.get_at(distance - 1, name)).await
        } else {
            Err(Box::new(Error::new("Invalid scope distance")))
        }
    }

    pub async fn assign_at(&self, distance: usize, name: &str, value: Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if distance == 0 {
            self.assign(name, value).await
        } else if let Some(enclosing) = &self.enclosing {
            Box::pin(enclosing.assign_at(distance - 1, name, value)).await
        } else {
            Err(Box::new(Error::new("Invalid scope distance")))
        }
    }
}
