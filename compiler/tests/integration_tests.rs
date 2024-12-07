use prism::interpreter::Interpreter;
use prism::value::{Value, ValueKind};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_arithmetic() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("2 + 3 * 4;".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(14.0));
    }

    #[tokio::test]
    async fn test_variables() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("let x = 42; x;".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(42.0));
    }

    #[tokio::test]
    async fn test_functions() {
        let interpreter = Interpreter::new();
        let source = r#"
            fn add(a, b) {
                return a + b;
            }
            add(2, 3);
        "#;
        let result = interpreter.evaluate(source.to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(5.0));
    }

    #[tokio::test]
    async fn test_modules() {
        let interpreter = Interpreter::new();
        let source = r#"
            module math ~> 0.9 {
                export fn add(a, b) {
                    return a + b;
                }
            }
            import { add } from "math";
            add(2, 3);
        "#;
        let result = interpreter.evaluate(source.to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(5.0));
    }

    #[tokio::test]
    async fn test_confidence() {
        let interpreter = Interpreter::new();
        let source = r#"
            let x = 42 ~> 0.9;
            x;
        "#;
        let result = interpreter.evaluate(source.to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(42.0));
        assert_eq!(result.confidence, Some(0.9));
    }
}
