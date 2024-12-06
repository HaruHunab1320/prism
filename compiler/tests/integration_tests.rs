use prism::{Interpreter, value::Value};

#[tokio::test]
async fn test_basic_arithmetic() {
    let interpreter = Interpreter::new("test_key".to_string());
    let result = interpreter.eval("2 + 3 * 4;".to_string()).await.unwrap();
    assert_eq!(result.as_float().unwrap(), 14.0);
}

#[tokio::test]
async fn test_variable_declaration() {
    let interpreter = Interpreter::new("test_key".to_string());
    let result = interpreter.eval("let x = 42; x;".to_string()).await.unwrap();
    assert_eq!(result.as_float().unwrap(), 42.0);
}

#[tokio::test]
async fn test_function_call() {
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
async fn test_if_statement() {
    let interpreter = Interpreter::new("test_key".to_string());
    let source = r#"
        let x = 10;
        if (x > 5) {
            x * 2;
        } else {
            x / 2;
        }
    "#;
    let result = interpreter.eval(source.to_string()).await.unwrap();
    assert_eq!(result.as_float().unwrap(), 20.0);
}

#[tokio::test]
async fn test_while_loop() {
    let interpreter = Interpreter::new("test_key".to_string());
    let source = r#"
        let x = 0;
        let i = 0;
        while (i < 5) {
            x = x + i;
            i = i + 1;
        }
        x;
    "#;
    let result = interpreter.eval(source.to_string()).await.unwrap();
    assert_eq!(result.as_float().unwrap(), 10.0);
}

#[tokio::test]
async fn test_string_concatenation() {
    let interpreter = Interpreter::new("test_key".to_string());
    let source = r#"
        let greeting = "Hello";
        let name = "World";
        greeting + " " + name;
    "#;
    let result = interpreter.eval(source.to_string()).await.unwrap();
    assert_eq!(result.as_string().unwrap(), "Hello World");
}

#[tokio::test]
async fn test_logical_operators() {
    let interpreter = Interpreter::new("test_key".to_string());
    let source = r#"
        let x = 5;
        let y = 10;
        x < y and y > 0;
    "#;
    let result = interpreter.eval(source.to_string()).await.unwrap();
    assert_eq!(result.as_bool().unwrap(), true);
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
async fn test_nested_scopes() {
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
async fn test_error_handling() {
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
