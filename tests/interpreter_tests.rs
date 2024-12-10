use prism::value::ValueKind;
use prism::interpreter::Interpreter;
use prism::error::Result;

#[tokio::test]
pub async fn test_basic_execution() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate("42;".to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(42.0));
    Ok(())
}

#[tokio::test]
pub async fn test_variables() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate("let x = 42; x;".to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(42.0));
    Ok(())
}

#[tokio::test]
pub async fn test_arithmetic() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate("2 + 3 * 4;".to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(14.0));
    Ok(())
}

#[tokio::test]
pub async fn test_functions() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        fn add(a, b) {
            return a + b;
        }
        add(2, 3);"#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(5.0));
    Ok(())
}

#[tokio::test]
pub async fn test_conditionals() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        let x = 10;
        if (x > 5) {
            x = 20;
        } else {
            x = 0;
        }
        x;"#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(20.0));
    Ok(())
}

#[tokio::test]
pub async fn test_loops() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        let x = 0;
        while (x < 5) {
            x = x + 1;
        }
        x;"#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(5.0));
    Ok(())
}

#[tokio::test]
pub async fn test_scope() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let source = r#"
        let x = 1;
        {
            let x = 2;
        }
        x;"#;
    let result = interpreter.evaluate(source.to_string()).await?;
    assert_eq!(result.kind, ValueKind::Number(1.0));
    Ok(())
}

#[tokio::test]
pub async fn test_error_handling() -> Result<()> {
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate("undefined_variable;".to_string()).await;
    assert!(result.is_err());
    Ok(())
} 