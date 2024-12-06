use prism::{Interpreter, Value};
use std::error::Error;

pub async fn test_confidence_flow() -> Result<(), Box<dyn Error + Send + Sync>> {
    let source = r#"
        let x = 42;
        x + 10
    "#;

    let result = eval_code(source).await?;
    assert_eq!(result.as_float().unwrap_or(0.0), 52.0);
    Ok(())
}

pub async fn test_context_operations() -> Result<(), Box<dyn Error + Send + Sync>> {
    let source = r#"
        let x = "test";
        x
    "#;

    let result = eval_code(source).await?;
    assert_eq!(result.as_string().unwrap_or_default(), "test");
    Ok(())
}

pub async fn test_pattern_matching() -> Result<(), Box<dyn Error + Send + Sync>> {
    let source = r#"
        let value = 42;
        if value > 40 {
            "high"
        } else if value > 20 {
            "medium"
        } else {
            "low"
        }
    "#;

    let result = eval_code(source).await?;
    assert_eq!(result.as_string().unwrap_or_default(), "high");
    Ok(())
}

pub async fn test_basic_arithmetic() -> Result<(), Box<dyn Error + Send + Sync>> {
    let source = r#"
        let x = 10;
        let y = 5;
        x + y
    "#;

    let result = eval_code(source).await?;
    assert_eq!(result.as_float().unwrap_or(0.0), 15.0);
    Ok(())
}

pub async fn test_string_operations() -> Result<(), Box<dyn Error + Send + Sync>> {
    let source = r#"
        let greeting = "Hello";
        let name = "World";
        greeting + " " + name
    "#;

    let result = eval_code(source).await?;
    assert_eq!(result.as_string().unwrap_or_default(), "Hello World");
    Ok(())
}

pub async fn test_boolean_operations() -> Result<(), Box<dyn Error + Send + Sync>> {
    let source = r#"
        let x = true;
        let y = false;
        x && y
    "#;

    let result = eval_code(source).await?;
    assert_eq!(result.as_bool().unwrap_or(true), false);
    Ok(())
}

pub async fn eval_code(source: &str) -> Result<Value, Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let interpreter = Interpreter::new(api_key);
    interpreter.eval(source.to_string()).await
}
