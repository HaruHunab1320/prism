use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::error::{PrismError, Result};
use crate::value::Value;

#[derive(Debug)]
pub struct Module {
    pub name: String,
    exports: HashMap<String, Value>,
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            exports: HashMap::new(),
        }
    }

    pub fn export(&mut self, name: String, value: Value) -> Result<()> {
        self.exports.insert(name, value);
        Ok(())
    }

    pub fn get_export(&self, name: &str) -> Result<Value> {
        self.exports
            .get(name)
            .cloned()
            .ok_or_else(|| PrismError::UndefinedVariable(name.to_string()))
    }
}

#[derive(Debug)]
pub struct ModuleRegistry {
    modules: HashMap<String, Arc<RwLock<Module>>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    pub fn register_module(&mut self, name: &str, module: Arc<RwLock<Module>>) -> Result<()> {
        if self.modules.contains_key(name) {
            return Err(PrismError::ModuleAlreadyExists(name.to_string()));
        }
        self.modules.insert(name.to_string(), module);
        Ok(())
    }

    pub async fn load_module(&self, name: &str) -> Result<Arc<RwLock<Module>>> {
        self.modules
            .get(name)
            .cloned()
            .ok_or_else(|| PrismError::ModuleNotFound(name.to_string()))
    }

    pub async fn resolve_import(&self, module_name: &str, import_name: &str) -> Result<Value> {
        let module = self.load_module(module_name).await?;
        let module_guard = module.read();
        module_guard.get_export(import_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ValueKind;

    #[tokio::test]
    async fn test_module_registry() -> Result<()> {
        let mut registry = ModuleRegistry::new();

        // Create and register a module
        let module = Arc::new(RwLock::new(Module::new("test".to_string())));
        {
            let mut module = module.write();
            module.export("value".to_string(), Value::new(ValueKind::Number(42.0)))?;
        }
        registry.register_module("test", module)?;

        // Load the module
        let loaded = registry.load_module("test").await?;
        assert_eq!(loaded.read().name, "test");
        
        // Resolve an import
        let value = registry.resolve_import("test", "value").await?;
        assert!(matches!(value.kind, ValueKind::Number(42.0)));

        Ok(())
    }
} 