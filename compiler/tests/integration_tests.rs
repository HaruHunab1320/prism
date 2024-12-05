use std::error::Error;
use prism::{Interpreter, Value};
use std::future::Future;
use std::pin::Pin;

type TestFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error>>> + Send>>;

async fn eval_code(source: &str) -> Result<Value, Box<dyn Error>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);
    interpreter.eval(source.to_string()).await
}

pub fn test_confidence_flow() -> TestFuture {
    Box::pin(async move {
        let source = r#"
        let x = 42 ~> 0.9;
        let y = x ~> 0.8;
        y;
        "#;

        let result = eval_code(source).await?;
        assert!(result.get_confidence().unwrap_or(0.0) > 0.7);
        Ok(())
    })
}

pub fn test_context_operations() -> TestFuture {
    Box::pin(async move {
        let source = r#"
        in context "medical" {
            let diagnosis = "flu" ~> 0.9;
            
            context transition "medical" to "treatment" with confidence 0.85 {
                let treatment = "antibiotics" ~> 0.9;
                treatment;
            }
        }
        "#;

        let result = eval_code(source).await?;
        assert!(result.get_confidence().unwrap_or(0.0) > 0.8);
        assert_eq!(result.get_context().unwrap_or_default(), "treatment".to_string());
        Ok(())
    })
}

pub fn test_pattern_matching() -> TestFuture {
    Box::pin(async move {
        let source = r#"
        let symptom = "fever" ~> 0.9;
        match symptom {
            x ~{0.8, 1.0} => "high",
            x ~{0.5, 0.79} => "medium",
            _ => "low"
        }
        "#;

        let result = eval_code(source).await?;
        assert_eq!(result.to_string(), "high");
        Ok(())
    })
}

pub fn test_tensor_operations() -> TestFuture {
    Box::pin(async move {
        let source = r#"
        let v1 = tensor([1.0, 0.0, 0.0], [3]) ~> 0.9;
        let v2 = tensor([0.0, 1.0, 0.0], [3]) ~> 0.85;
        let similarity = v1.cosine_similarity(v2);
        similarity;
        "#;

        let result = eval_code(source).await?;
        assert_eq!(result.as_float().unwrap_or(1.0), 0.0);
        Ok(())
    })
}

pub fn test_semantic_matching() -> TestFuture {
    Box::pin(async move {
        let source = r#"
        let pattern = "patient has fever and cough";
        let description = "severe fever with persistent cough";
        let match_score = pattern ~= description;
        match_score;
        "#;

        let result = eval_code(source).await?;
        assert!(result.get_confidence().unwrap_or(0.0) > 0.5);
        Ok(())
    })
}

pub fn test_verification() -> TestFuture {
    Box::pin(async move {
        let source = r#"
        verify against sources ["medical_database"] {
            let condition = "influenza" ~> 0.85;
            condition;
        }
        "#;

        let result = eval_code(source).await?;
        assert!(result.get_confidence().unwrap_or(0.0) > 0.8);
        Ok(())
    })
}

pub fn test_uncertain_conditionals() -> TestFuture {
    Box::pin(async move {
        let source = r#"
        let confidence_value = 0.75;
        uncertain if (confidence_value > 0.8) {
            "high"
        } medium (confidence_value > 0.6) {
            "medium"
        } low {
            "low"
        }
        "#;

        let result = eval_code(source).await?;
        assert_eq!(result.to_string(), "medium");
        Ok(())
    })
}

pub fn test_try_confidence() -> TestFuture {
    Box::pin(async move {
        let source = r#"
        try confidence {
            let risky_operation = "surgery" ~> 0.7;
            risky_operation;
        } below threshold 0.8 {
            "low confidence"
        } uncertain {
            "error"
        }
        "#;

        let result = eval_code(source).await?;
        assert_eq!(result.to_string(), "low confidence");
        Ok(())
    })
}

pub fn test_async_operations() -> TestFuture {
    Box::pin(async move {
        let source = r#"
        async fn analyze_data(data: string) -> string ~0.9 {
            let result = await llm.analyze(data) ~> 0.85;
            return result ~> 0.9;
        }
        
        analyze_data("test data");
        "#;

        let result = eval_code(source).await?;
        assert!(result.get_confidence().unwrap_or(0.0) > 0.8);
        Ok(())
    })
}

pub fn test_all_features() -> TestFuture {
    Box::pin(async move {
        let source = include_str!("integration/all_features.prism");
        let result = eval_code(source).await?;
        assert!(result.get_confidence().unwrap_or(0.0) > 0.0);
        Ok(())
    })
}

// Helper function to run all tests
pub async fn run_all_tests() -> Result<(), Box<dyn Error>> {
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