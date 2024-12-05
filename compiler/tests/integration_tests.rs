use prism::{Interpreter, Value};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

type TestFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send>>;

async fn eval_code(source: &str) -> Result<Value, Box<dyn Error>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);
    interpreter.eval(source.to_string()).await
}

pub async fn test_confidence_flow() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
        let x = 42 ~> 0.9;
        x + 10
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    assert!(result.get_confidence().unwrap_or(0.0) > 0.7);
    Ok(())
}

pub async fn test_context_operations() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
        in context "test" {
            let x = "value" ~> 0.9;
            x
        }
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    assert!(result.get_confidence().unwrap_or(0.0) > 0.8);
    assert_eq!(
        result.get_context().unwrap_or_default(),
        "test"
    );
    Ok(())
}

pub async fn test_pattern_matching() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
        let value = 42;
        match value {
            x if x > 40 => "high",
            x if x > 20 => "medium",
            _ => "low"
        }
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    assert_eq!(result.as_string().unwrap_or_default(), "high");
    Ok(())
}

pub async fn test_tensor_operations() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
        let t1 = tensor([1.0, 2.0, 3.0]);
        let t2 = tensor([4.0, 5.0, 6.0]);
        t1 + t2
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    assert_eq!(result.as_float().unwrap_or(1.0), 0.0);
    Ok(())
}

pub async fn test_semantic_matching() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
        let text1 = "The weather is nice";
        let text2 = "It's a beautiful day";
        semantic_match(text1, text2)
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    assert!(result.get_confidence().unwrap_or(0.0) > 0.5);
    Ok(())
}

pub async fn test_verification() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
        verify against sources ["test"] {
            let x = 42 ~> 0.9;
            x > 40
        }
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    assert!(result.get_confidence().unwrap_or(0.0) > 0.8);
    Ok(())
}

pub async fn test_uncertain_conditionals() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
        let x = 0.85;
        uncertain if (x > 0.8) {
            "high"
        } medium (x > 0.5) {
            "medium"
        } low {
            "low"
        }
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    assert_eq!(result.as_string().unwrap_or_default(), "medium");
    Ok(())
}

pub async fn test_try_confidence() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
        try confidence {
            let x = 0.3 ~> 0.4;
            x
        } below 0.5 {
            "low confidence"
        }
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    assert_eq!(result.as_string().unwrap_or_default(), "low confidence");
    Ok(())
}

pub async fn test_async_operations() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
        async fn fetch() -> string ~0.9 {
            promise ~0.9 "data"
        }
        await fetch()
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    assert!(result.get_confidence().unwrap_or(0.0) > 0.8);
    Ok(())
}

pub async fn test_all_features() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = include_str!("integration/all_features.prism");
    let result = interpreter.eval(source.to_string()).await?;
    assert!(result.get_confidence().unwrap_or(0.0) > 0.0);
    Ok(())
}

pub async fn run_all_tests() -> Result<(), Box<dyn Error + Send + Sync>> {
    test_confidence_flow().await?;
    test_context_operations().await?;
    test_pattern_matching().await?;
    test_tensor_operations().await?;
    test_semantic_matching().await?;
    test_verification().await?;
    test_uncertain_conditionals().await?;
    test_try_confidence().await?;
    test_async_operations().await?;
    test_all_features().await?;
    Ok(())
}
