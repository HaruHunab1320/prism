use prism::lexer::Lexer;
use prism::parser::Parser;

// Basic Declaration Tests
#[test]
pub fn test_parse_function_declaration() {
    let source = r#"
        fn add(x, y) {
            let result = x + y;
            result;
        }
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
pub fn test_parse_async_function() {
    let source = r#"
        async fn fetch(url) {
            let response = http.get(url);
            response;
        }
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
pub fn test_parse_let_declaration() {
    let source = r#"
        let x = 42;
        x;
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
pub fn test_parse_if_statement() {
    let source = r#"
        if (x > 0) {
            let y = x + 1;
            y;
        }
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

// Operator Precedence Tests
#[test]
pub fn test_arithmetic_precedence() {
    let source = r#"
        let result = 1 + 2 * 3;  // Should be 7, not 9
        let grouped = (1 + 2) * 3;  // Should be 9
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
pub fn test_logical_precedence() {
    let source = r#"
        let result = true and false or true;  // Should be true
        let grouped = true and (false or true);  // Should be true
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

// Expression Tests
#[test]
pub fn test_unary_expressions() {
    let source = r#"
        let negated = -42;
        let not_true = !true;
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
pub fn test_binary_expressions() {
    let source = r#"
        let sum = 1 + 2;
        let product = 3 * 4;
        let comparison = 5 > 6;
        let equality = 7 == 8;
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
pub fn test_call_expressions() {
    let source = r#"
        let result = add(1, 2);
        let chained = obj.method().field;
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

// Error Recovery Tests
#[test]
pub fn test_missing_semicolon_recovery() {
    let source = r#"
        let x = 42
        let y = 43;  // Should continue parsing after error
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
pub fn test_missing_closing_brace_recovery() {
    let source = r#"
        fn incomplete() {
            let x = 42;
        // Missing closing brace
        fn next() {
            let y = 43;
        }
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
}

#[test]
pub fn test_invalid_expression_recovery() {
    let source = r#"
        let x = @invalid@;
        let y = 42;  // Should continue parsing after error
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_ok());
} 