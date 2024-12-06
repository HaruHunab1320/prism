#[cfg(test)]
mod tests {
    use prism::{Interpreter, value::{Value, ValueKind}};

    #[tokio::test]
    async fn test_variable_scoping() {
        let interpreter = Interpreter::new("test_key".to_string());
        let source = r#"
            let x = 1;
            {
                let x = 2;
                {
                    let x = 3;
                    x;
                }
            }
        "#;
        let result = interpreter.eval(source.to_string()).await.unwrap();
        assert_eq!(result.as_float().unwrap(), 3.0);
    }

    #[tokio::test]
    async fn test_variable_shadowing() {
        let interpreter = Interpreter::new("test_key".to_string());
        let source = r#"
            let x = 1;
            let y = 2;
            {
                let x = 10;
                x + y;
            }
        "#;
        let result = interpreter.eval(source.to_string()).await.unwrap();
        assert_eq!(result.as_float().unwrap(), 12.0);
    }

    #[tokio::test]
    async fn test_function_declaration() {
        let interpreter = Interpreter::new("test_key".to_string());
        let source = r#"
            fn add(a, b) {
                return a + b;
            }
            add(5, 3);
        "#;
        let result = interpreter.eval(source.to_string()).await.unwrap();
        assert_eq!(result.as_float().unwrap(), 8.0);
    }

    #[tokio::test]
    async fn test_function_closure() {
        let interpreter = Interpreter::new("test_key".to_string());
        let source = r#"
            let x = 10;
            fn add_x(n) {
                return n + x;
            }
            add_x(5);
        "#;
        let result = interpreter.eval(source.to_string()).await.unwrap();
        assert_eq!(result.as_float().unwrap(), 15.0);
    }

    #[tokio::test]
    async fn test_async_function() {
        let interpreter = Interpreter::new("test_key".to_string());
        let source = r#"
            async fn delayed_add(a, b) {
                return a + b;
            }
            delayed_add(7, 3);
        "#;
        let result = interpreter.eval(source.to_string()).await.unwrap();
        assert_eq!(result.as_float().unwrap(), 10.0);
    }

    #[tokio::test]
    async fn test_nested_functions() {
        let interpreter = Interpreter::new("test_key".to_string());
        let source = r#"
            fn outer(x) {
                fn inner(y) {
                    return x + y;
                }
                return inner(10);
            }
            outer(5);
        "#;
        let result = interpreter.eval(source.to_string()).await.unwrap();
        assert_eq!(result.as_float().unwrap(), 15.0);
    }

    #[tokio::test]
    async fn test_function_recursion() {
        let interpreter = Interpreter::new("test_key".to_string());
        let source = r#"
            fn factorial(n) {
                if (n <= 1) {
                    return 1;
                } else {
                    return n * factorial(n - 1);
                }
            }
            factorial(5);
        "#;
        let result = interpreter.eval(source.to_string()).await.unwrap();
        assert_eq!(result.as_float().unwrap(), 120.0);
    }

    #[tokio::test]
    async fn test_complex_scoping() {
        let interpreter = Interpreter::new("test_key".to_string());
        let source = r#"
            let a = 1;
            let b = 2;
            {
                let c = 3;
                let a = 10;
                {
                    let b = 20;
                    a + b + c;
                }
            }
        "#;
        let result = interpreter.eval(source.to_string()).await.unwrap();
        assert_eq!(result.as_float().unwrap(), 33.0);
    }

    #[tokio::test]
    async fn test_function_error_handling() {
        let interpreter = Interpreter::new("test_key".to_string());
        let source = r#"
            fn safe_divide(a, b) {
                if (b == 0) {
                    return null;
                } else {
                    return a / b;
                }
            }
            safe_divide(10, 0);
        "#;
        let result = interpreter.eval(source.to_string()).await.unwrap();
        assert!(matches!(result, Value::Null));
    }

    #[tokio::test]
    async fn test_basic_confidence() {
        let interpreter = Interpreter::new("test_key".to_string());
        let result = interpreter.eval("let x = 42 ~> 0.9;".to_string()).await.unwrap();
        assert_eq!(result.get_confidence(), 0.9);
    }

    #[tokio::test]
    async fn test_confidence_combination() {
        let interpreter = Interpreter::new("test_key".to_string());
        let result = interpreter.eval("
            let x = 42 ~> 0.9;
            let y = 10 ~> 0.8;
            x + y;
        ".to_string()).await.unwrap();
        assert!((result.get_confidence() - 0.72).abs() < f64::EPSILON); // 0.9 * 0.8
    }

    #[tokio::test]
    async fn test_confidence_in_function() {
        let interpreter = Interpreter::new("test_key".to_string());
        let result = interpreter.eval("
            fn process(x) ~> 0.95 {
                return x * 2;
            }
            let input = 10 ~> 0.8;
            process(input);
        ".to_string()).await.unwrap();
        assert!((result.get_confidence() - 0.76).abs() < f64::EPSILON); // 0.8 * 0.95
    }

    #[tokio::test]
    async fn test_confidence_bounds() {
        let interpreter = Interpreter::new("test_key".to_string());
        let result = interpreter.eval("42 ~> 1.5;".to_string()).await.unwrap();
        assert_eq!(result.get_confidence(), 1.0); // Should be clamped to 1.0

        let result = interpreter.eval("42 ~> -0.5;".to_string()).await.unwrap();
        assert_eq!(result.get_confidence(), 0.0); // Should be clamped to 0.0
    }

    #[tokio::test]
    async fn test_confidence_with_context() {
        let interpreter = Interpreter::new("test_key".to_string());
        let result = interpreter.eval("
            in context Medical {
                let diagnosis = \"flu\" ~> 0.85;
                diagnosis
            }
        ".to_string()).await.unwrap();
        assert_eq!(result.get_confidence(), 0.85);
        assert_eq!(result.get_context(), Some("Medical"));
    }
} 