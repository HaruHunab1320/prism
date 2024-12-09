use prism::value::{Value, ValueKind};
use prism::interpreter::Interpreter;
use prism::environment::Environment;
use std::sync::{Arc, RwLock};

#[tokio::test]
async fn test_basic_execution() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate("42;".to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(42.0));
    Ok(())
}

#[tokio::test]
async fn test_variables() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate("let x = 42; x;".to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(42.0));
    Ok(())
}

#[tokio::test]
async fn test_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate("2 + 3 * 4;".to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(14.0));
    Ok(())
}

#[tokio::test]
async fn test_functions() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        fn add(a, b) {
            a + b;
        }
        add(2, 3);
    "#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(5.0));
    Ok(())
}

#[tokio::test]
async fn test_conditionals() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        let x = 42;
        if (x > 0) {
            true;
        } else {
            false;
        }
    "#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Boolean(true));
    Ok(())
}

#[tokio::test]
async fn test_loops() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        let x = 0;
        while (x < 5) {
            x = x + 1;
        }
        x;
    "#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(5.0));
    Ok(())
}

#[tokio::test]
async fn test_scope() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        let x = 1;
        {
            let x = 2;
            x;
        }
        x;
    "#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(1.0));
    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate("1 / 0;".to_string()).await;
    assert!(result.is_err());
    Ok(())
} 