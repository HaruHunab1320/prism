use prism::{Interpreter, Parser, Lexer, Value};

const EPSILON: f64 = 1e-10;

fn assert_float_eq(a: f64, b: f64) {
    assert!((a - b).abs() < EPSILON, "Expected {} but got {}", b, a);
}

#[tokio::test]
async fn test_basic_arithmetic() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        let x = 5.0;
        let y = 3.0;
        x + y;
    "#;

    let mut interpreter = Interpreter::new();
    let tokens = Lexer::new(source).collect::<Vec<_>>();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    let mut result = Value::Void;
    for stmt in statements {
        result = interpreter.eval_stmt(&stmt)?;
    }

    assert_eq!(result, Value::Float(8.0));
    Ok(())
}

#[tokio::test]
async fn test_function_definition() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        fn add(x, y) {
            return x + y;
        }
        add(2.0, 3.0);
    "#;

    let mut interpreter = Interpreter::new();
    let tokens = Lexer::new(source).collect::<Vec<_>>();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    let mut result = Value::Void;
    for stmt in statements {
        result = interpreter.eval_stmt(&stmt)?;
    }

    assert_eq!(result, Value::Float(5.0));
    Ok(())
}

#[tokio::test]
async fn test_confidence_flow() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        let x = 0.8;
        let y = x ~> 0.9;
        y;
    "#;

    let mut interpreter = Interpreter::new();
    let tokens = Lexer::new(source).collect::<Vec<_>>();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    let mut result = Value::Void;
    for stmt in statements {
        result = interpreter.eval_stmt(&stmt)?;
    }

    if let Value::Float(n) = result {
        assert_float_eq(n, 0.72);
    } else {
        panic!("Expected float value");
    }
    Ok(())
}

#[tokio::test]
async fn test_context_block() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        context "validation" {
            let x = 0.8;
            verify x > 0.7 {
                true;
            }
        }
    "#;

    let mut interpreter = Interpreter::new();
    let tokens = Lexer::new(source).collect::<Vec<_>>();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    let mut result = Value::Void;
    for stmt in statements {
        result = interpreter.eval_stmt(&stmt)?;
    }

    assert!(matches!(result, Value::Boolean(true)));
    Ok(())
} 