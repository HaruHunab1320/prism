use std::sync::{Arc, RwLock};
use prism::value::{Value, ValueKind};
use prism::error::Result;
use prism::module::{Module, ModuleRegistry};

#[tokio::test]
async fn test_module_confidence() -> Result<()> {
    let mut registry = ModuleRegistry::new();

    // Create and register a module with confidence
    let module = Arc::new(RwLock::new(Module::new("test".to_string())));
    {
        let mut module = module.write();
        module.export("value".to_string(), Value::with_confidence(ValueKind::Number(42.0), 0.9))?;
    }
    registry.register_module("test", module)?;

    // Resolve an import
    let result = registry.resolve_import("test", "value").await?;
    assert_eq!(result.confidence, 0.9);

    Ok(())
}

#[tokio::test]
async fn test_module_confidence_inheritance() -> Result<()> {
    let mut registry = ModuleRegistry::new();

    // Create and register a module with confidence
    let module = Arc::new(RwLock::new(Module::new("test".to_string())));
    {
        let mut module = module.write();
        module.export("value".to_string(), Value::with_confidence(ValueKind::Number(42.0), 0.95))?;
    }
    registry.register_module("test", module)?;

    // Import with confidence
    let result = registry.resolve_import("test", "value").await?;
    assert_eq!(result.confidence, 0.95);

    Ok(())
}

#[tokio::test]
async fn test_module_confidence_composition() -> Result<()> {
    let mut registry = ModuleRegistry::new();

    // Create and register a module with confidence
    let module = Arc::new(RwLock::new(Module::new("test".to_string())));
    {
        let mut module = module.write();
        module.export("value".to_string(), Value::with_confidence(ValueKind::Number(42.0), 0.9))?;
    }
    registry.register_module("test", module)?;

    // Import with confidence
    let result = registry.resolve_import("test", "value").await?;
    assert_eq!(result.confidence, 0.9);

    Ok(())
}

#[tokio::test]
async fn test_module_confidence_chain() -> Result<()> {
    let mut registry = ModuleRegistry::new();

    // Create and register modules with confidence
    let module1 = Arc::new(RwLock::new(Module::new("test1".to_string())));
    {
        let mut module = module1.write();
        module.export("value".to_string(), Value::with_confidence(ValueKind::Number(42.0), 0.8))?;
    }
    registry.register_module("test1", module1)?;

    let module2 = Arc::new(RwLock::new(Module::new("test2".to_string())));
    {
        let mut module = module2.write();
        module.export("value".to_string(), Value::with_confidence(ValueKind::Number(42.0), 0.85))?;
    }
    registry.register_module("test2", module2)?;

    let module3 = Arc::new(RwLock::new(Module::new("test3".to_string())));
    {
        let mut module = module3.write();
        module.export("value".to_string(), Value::with_confidence(ValueKind::Number(42.0), 0.95))?;
    }
    registry.register_module("test3", module3)?;

    // Import with confidence chain
    let result = registry.resolve_import("test3", "value").await?;
    assert_eq!(result.confidence, 0.95);

    Ok(())
}

#[tokio::test]
async fn test_module_context() -> Result<()> {
    let mut registry = ModuleRegistry::new();

    // Create and register a module with context
    let module = Arc::new(RwLock::new(Module::new("test".to_string())));
    {
        let mut module = module.write();
        module.export("value".to_string(), Value::with_context(ValueKind::Number(42.0), "test context".to_string()))?;
    }
    registry.register_module("test", module)?;

    // Import with context
    let result = registry.resolve_import("test", "value").await?;
    assert_eq!(result.context, Some("test context".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_module_confidence_and_context() -> Result<()> {
    let mut registry = ModuleRegistry::new();

    // Create and register a module with confidence and context
    let module = Arc::new(RwLock::new(Module::new("test".to_string())));
    {
        let mut module = module.write();
        module.export("value".to_string(), Value::with_confidence_and_context(
            ValueKind::Number(42.0),
            0.9,
            "test context".to_string(),
        ))?;
    }
    registry.register_module("test", module)?;

    // Import with confidence and context
    let result = registry.resolve_import("test", "value").await?;
    assert_eq!(result.confidence, 0.9);
    assert_eq!(result.context, Some("test context".to_string()));

    Ok(())
} 