use prism::interpreter::Interpreter;
use prism::value::{Value, ValueKind};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::{Value, ValueKind};

    #[tokio::test]
    async fn test_number_literal() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("42;".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(42.0));
    }

    #[tokio::test]
    async fn test_string_literal() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("\"hello\";".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::String("hello".to_string()));
    }

    #[tokio::test]
    async fn test_boolean_literal() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("true;".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Boolean(true));
    }

    #[tokio::test]
    async fn test_nil_literal() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("nil;".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Nil);
    }

    #[tokio::test]
    async fn test_arithmetic() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("2 + 3 * 4;".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(14.0));
    }

    #[tokio::test]
    async fn test_comparison() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("1 < 2 and 3 >= 3;".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Boolean(true));
    }

    #[tokio::test]
    async fn test_variable() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("let x = 42; x;".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(42.0));
    }

    #[tokio::test]
    async fn test_function() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("
            fn add(a, b) {
                return a + b;
            }
            add(2, 3);
        ".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(5.0));
    }

    #[tokio::test]
    async fn test_confidence() {
        let interpreter = Interpreter::new();
        let result = interpreter.evaluate("42 ~> 0.9;".to_string()).await.unwrap();
        assert_eq!(result.kind, ValueKind::Number(42.0));
        assert_eq!(result.confidence, Some(0.9));
    }
} 