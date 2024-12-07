use std::path::PathBuf;
use prism::{
    interpreter::Interpreter,
    module::{Module, ModuleRegistry},
    value::{Value, ValueKind},
    error::PrismError,
};
use std::sync::{Arc, RwLock};

#[test]
fn test_basic_module_loading() {
    let mut interpreter = Interpreter::new();
    
    // Create a test module
    let mut test_module = Module::new("test", PathBuf::from("test.prism"));
    test_module.register_export("greeting", Value::new(ValueKind::String("Hello".to_string())));
    
    // Register the module
    interpreter.register_builtin_module("test", test_module).unwrap();
    
    // Try to load and use the module
    let result = interpreter.import_symbol("test", "greeting").unwrap();
    assert!(matches!(result.kind, ValueKind::String(ref s) if s == "Hello"));
}

#[test]
fn test_module_dependencies() {
    let mut interpreter = Interpreter::new();
    
    // Create dependent modules
    let mut core_module = Module::new("core", PathBuf::from("std/core.prism"));
    core_module.register_export("add", Value::new(ValueKind::String("add".to_string())));
    
    let mut math_module = Module::new("math", PathBuf::from("std/math.prism"));
    math_module.register_import("add", "core", "add");
    
    // Register modules
    interpreter.register_builtin_module("core", core_module).unwrap();
    interpreter.register_builtin_module("math", math_module).unwrap();
    
    // Load math module which should load core module
    let math = interpreter.load_module("math").unwrap();
    assert!(math.imports.contains_key("add"));
}

#[test]
fn test_circular_dependency_detection() {
    let mut interpreter = Interpreter::new();
    
    // Create modules with circular dependency
    let mut module_a = Module::new("a", PathBuf::from("a.prism"));
    module_a.register_import("b_func", "b", "func");
    
    let mut module_b = Module::new("b", PathBuf::from("b.prism"));
    module_b.register_import("a_func", "a", "func");
    
    // Register modules
    interpreter.register_builtin_module("a", module_a).unwrap();
    interpreter.register_builtin_module("b", module_b).unwrap();
    
    // Attempt to load module a
    let result = interpreter.load_module("a");
    let err = result.unwrap_err();
    if let Some(prism_err) = err.downcast_ref::<PrismError>() {
        assert!(matches!(prism_err, PrismError::CircularDependency(_)));
    } else {
        panic!("Expected PrismError::CircularDependency");
    }
}

#[test]
fn test_module_symbol_resolution() {
    let mut interpreter = Interpreter::new();
    
    // Create a module with multiple exports
    let mut test_module = Module::new("test", PathBuf::from("test.prism"));
    test_module.register_export("var1", Value::new(ValueKind::Number(1.0)));
    test_module.register_export("var2", Value::new(ValueKind::Number(2.0)));
    
    // Register the module
    interpreter.register_builtin_module("test", test_module).unwrap();
    
    // Test successful symbol resolution
    let var1 = interpreter.import_symbol("test", "var1").unwrap();
    assert!(matches!(var1.kind, ValueKind::Number(n) if n == 1.0));
    
    // Test symbol not found
    let result = interpreter.import_symbol("test", "nonexistent");
    let err = result.unwrap_err();
    if let Some(prism_err) = err.downcast_ref::<PrismError>() {
        assert!(matches!(prism_err, PrismError::SymbolNotFound { .. }));
    } else {
        panic!("Expected PrismError::SymbolNotFound");
    }
}

#[test]
fn test_module_registry() {
    let registry = ModuleRegistry::new();
    
    // Create and register a module
    let module = Module::new("test", PathBuf::from("test.prism"));
    registry.register_builtin_module("test", module).unwrap();
    
    // Test module resolution
    let resolved = registry.resolve_module("test").unwrap();
    assert_eq!(resolved.name, "test");
    
    // Test nonexistent module
    let result = registry.resolve_module("nonexistent");
    let err = result.unwrap_err();
    if let Some(prism_err) = err.downcast_ref::<PrismError>() {
        assert!(matches!(prism_err, PrismError::ModuleNotFound(_)));
    } else {
        panic!("Expected PrismError::ModuleNotFound");
    }
}

#[cfg(test)]
mod tests {
    use prism::interpreter::Interpreter;
    use prism::value::{Value, ValueKind};
    use prism::module::{Module, ModuleRegistry};
    use prism::error::PrismError;
    use std::sync::{Arc, RwLock};
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_module_definition() {
        let interpreter = Interpreter::new();
        let source = r#"
            module math ~> 0.9 {
                export let PI = 3.14159;
                export fn add(a, b) {
                    return a + b;
                }
            }
            math.PI;
        "#;

        let result = interpreter.evaluate(source.to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(3.14159));
        assert_eq!(result.confidence, Some(0.9));
    }

    #[tokio::test]
    async fn test_module_import() {
        let interpreter = Interpreter::new();
        let source = r#"
            module helper ~> 0.9 {
                export fn help() {
                    return "helped";
                }
            }

            import { help } from "helper" ~> 0.8;
            help();
        "#;

        let result = interpreter.evaluate(source.to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::String("helped".to_string()));
        assert_eq!(result.confidence, Some(0.9 * 0.8));
    }

    #[tokio::test]
    async fn test_module_confidence_propagation() {
        let interpreter = Interpreter::new();
        let source = r#"
            module calculator ~> 0.9 {
                export fn add(a, b) ~> 0.95 {
                    return a + b;
                }
            }

            import { add } from "calculator" ~> 0.8;
            let result = add(2, 3);
        "#;

        let result = interpreter.evaluate(source.to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(5.0));
        assert_eq!(result.confidence, Some(0.9 * 0.95 * 0.8));
    }

    #[tokio::test]
    async fn test_module_loading() {
        let registry = ModuleRegistry::new();
        
        // Create a test module file
        std::fs::create_dir_all("lib").unwrap();
        std::fs::write(
            "lib/test_module.prism",
            r#"
                export fn add(a, b) {
                    return a + b;
                }

                export let greeting = "Hello, World!" ~> 0.9;
            "#,
        ).unwrap();

        // Load the module
        let module = registry.load_module("test_module").unwrap();
        
        // Verify exports
        let module_guard = module.read().unwrap();
        assert!(module_guard.get_export("add").is_some());
        assert!(module_guard.get_export("greeting").is_some());
        
        // Clean up
        std::fs::remove_file("lib/test_module.prism").unwrap();
    }

    #[tokio::test]
    async fn test_module_imports() {
        let registry = ModuleRegistry::new();
        
        // Create utility module
        std::fs::write(
            "lib/utils.prism",
            r#"
                export fn multiply(a, b) {
                    return a * b;
                }
            "#,
        ).unwrap();

        // Create main module that imports from utils
        std::fs::write(
            "lib/main.prism",
            r#"
                import { multiply } from "utils" ~> 0.9;
                
                export fn calculate(x) {
                    return multiply(x, 2);
                }
            "#,
        ).unwrap();

        // Load and verify main module
        let module = registry.load_module("main").unwrap();
        
        // Verify imports
        let module_guard = module.read().unwrap();
        assert!(module_guard.get_import("multiply").is_some());
        let (module_name, symbol_name) = module_guard.get_import("multiply").unwrap();
        assert_eq!(module_name, "utils");
        assert_eq!(symbol_name, "multiply");
        
        // Clean up
        std::fs::remove_file("lib/utils.prism").unwrap();
        std::fs::remove_file("lib/main.prism").unwrap();
    }

    #[tokio::test]
    async fn test_module_error_handling() {
        let registry = ModuleRegistry::new();
        
        // Test loading non-existent module
        let result = registry.load_module("nonexistent");
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err().downcast_ref::<PrismError>(),
            Some(PrismError::ModuleNotFound(_))
        ));

        // Test invalid exports
        std::fs::write(
            "lib/invalid.prism",
            r#"
                export 123;  // Invalid export
            "#,
        ).unwrap();

        let result = registry.load_module("invalid");
        assert!(result.is_err());
        
        // Clean up
        std::fs::remove_file("lib/invalid.prism").unwrap();
    }

    #[tokio::test]
    async fn test_module_caching() {
        let registry = ModuleRegistry::new();
        
        // Create test module
        std::fs::write(
            "lib/cached.prism",
            r#"
                export let value = 42;
            "#,
        ).unwrap();

        // Load module twice
        let first_load = registry.load_module("cached").unwrap();
        let second_load = registry.load_module("cached").unwrap();
        
        // Verify it's the same instance
        assert!(Arc::ptr_eq(&first_load, &second_load));
        
        // Clean up
        std::fs::remove_file("lib/cached.prism").unwrap();
    }
} 