use prism::token::{Token, TokenKind};
use prism::lexer::Lexer;
use prism::parser::Parser;
use prism::ast::{Expr, Stmt};
use prism::value::{Value, ValueKind};

// Helper functions for AST verification
fn parse_stmt(source: &str) -> Vec<Stmt> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    parser.parse().unwrap()
}

fn assert_function(stmt: &Stmt, expected_name: &str, expected_params: &[&str], is_async: bool) {
    match stmt {
        Stmt::Function { name, params, is_async: actual_async, .. } => {
            assert_eq!(name, expected_name);
            assert_eq!(params.len(), expected_params.len());
            for (param, &expected) in params.iter().zip(expected_params) {
                assert_eq!(param, expected);
            }
            assert_eq!(*actual_async, is_async);
        }
        _ => panic!("Expected function declaration, got {:?}", stmt),
    }
}

fn assert_let(stmt: &Stmt, expected_name: &str, expected_value: Option<&Expr>) {
    match stmt {
        Stmt::Let(name, value) => {
            assert_eq!(name, expected_name);
            match (value, expected_value) {
                (Some(v1), Some(v2)) => assert_expr_eq(v1, v2),
                (None, None) => (),
                _ => panic!("Let initializer mismatch"),
            }
        }
        _ => panic!("Expected let declaration, got {:?}", stmt),
    }
}

fn assert_expr_eq(actual: &Expr, expected: &Expr) {
    assert_eq!(format!("{:?}", actual), format!("{:?}", expected));
}

// Updated tests with AST verification
#[test]
pub fn test_parse_function_declaration() {
    let source = "fn test() { return 42; }".to_string();
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();
    assert!(!statements.is_empty());
}

#[test]
pub fn test_parse_async_function() {
    let source = r#"
        async fn fetch(url) {
            let response = http.get(url);
            response;
        }
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 1);
    assert_function(&stmts[0], "fetch", &["url"], true);
    
    if let Stmt::Function { body, .. } = &stmts[0] {
        if let Stmt::Block(block_stmts) = &**body {
            assert_eq!(block_stmts.len(), 2);
            // Verify let statement
            if let Stmt::Let(name, Some(init)) = &block_stmts[0] {
                assert_eq!(name, "response");
                if let Expr::Call { callee, arguments } = &**init {
                    if let Expr::Get { object, name } = &**callee {
                        assert_expr_eq(object, &Expr::Variable("http".to_string()));
                        assert_eq!(name, "get");
                    } else {
                        panic!("Expected get expression");
                    }
                    assert_eq!(arguments.len(), 1);
                    assert_expr_eq(&arguments[0], &Expr::Variable("url".to_string()));
                } else {
                    panic!("Expected call expression");
                }
            } else {
                panic!("Expected let statement");
            }
            // Verify return expression
            if let Stmt::Expression(expr) = &block_stmts[1] {
                assert_expr_eq(expr, &Expr::Variable("response".to_string()));
            } else {
                panic!("Expected expression statement");
            }
        } else {
            panic!("Expected block");
        }
    } else {
        panic!("Expected function");
    }
}

#[test]
pub fn test_parse_let_declaration() {
    let source = r#"
        let x = 42;
        x;
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 2);
    
    // Verify let statement
    assert_let(&stmts[0], "x", Some(&Expr::Literal(Value::new(ValueKind::Number(42.0)))));
    
    // Verify expression statement
    if let Stmt::Expression(expr) = &stmts[1] {
        assert_expr_eq(expr, &Expr::Variable("x".to_string()));
    } else {
        panic!("Expected expression statement");
    }
}

#[test]
pub fn test_parse_if_statement() {
    let source = r#"
        if (x > 5) {
            print("high");
        } else {
            print("low");
        }
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 1);
    
    if let Stmt::If { condition, then_branch, else_branch } = &stmts[0] {
        assert!(else_branch.is_some());
    } else {
        panic!("Expected if statement");
    }
}

#[test]
pub fn test_arithmetic_precedence() {
    let source = r#"
        let result = 1 + 2 * 3;  // Should be 7, not 9
        let grouped = (1 + 2) * 3;  // Should be 9
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 2);
    
    // Verify first let statement (1 + 2 * 3)
    if let Stmt::Let(name, Some(init)) = &stmts[0] {
        assert_eq!(name, "result");
        if let Expr::Binary { left: l1, right: r1, .. } = &**init {
            assert_expr_eq(l1, &Expr::Literal(Value::new(ValueKind::Number(1.0))));
            if let Expr::Binary { left: l2, right: r2, .. } = &**r1 {
                assert_expr_eq(l2, &Expr::Literal(Value::new(ValueKind::Number(2.0))));
                assert_expr_eq(r2, &Expr::Literal(Value::new(ValueKind::Number(3.0))));
            } else {
                panic!("Expected binary expression");
            }
        } else {
            panic!("Expected binary expression");
        }
    } else {
        panic!("Expected let statement");
    }
    
    // Verify second let statement ((1 + 2) * 3)
    if let Stmt::Let(name, Some(init)) = &stmts[1] {
        assert_eq!(name, "grouped");
        if let Expr::Binary { left: l1, right: r1, .. } = &**init {
            if let Expr::Grouping(inner) = &**l1 {
                if let Expr::Binary { left: l2, right: r2, .. } = &**inner {
                    assert_expr_eq(l2, &Expr::Literal(Value::new(ValueKind::Number(1.0))));
                    assert_expr_eq(r2, &Expr::Literal(Value::new(ValueKind::Number(2.0))));
                } else {
                    panic!("Expected binary expression");
                }
            } else {
                panic!("Expected grouping");
            }
            assert_expr_eq(r1, &Expr::Literal(Value::new(ValueKind::Number(3.0))));
        } else {
            panic!("Expected binary expression");
        }
    } else {
        panic!("Expected let statement");
    }
}

#[test]
pub fn test_logical_precedence() {
    let source = r#"
        let result = true and false or true;  // Should be true
        let grouped = true and (false or true);  // Should be true
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 2);
    
    // Helper function to create boolean literals
    let bool_lit = |b| Expr::Literal(Value::new(ValueKind::Boolean(b)));
    
    // Verify first let statement (true and false or true)
    if let Stmt::Let(name, Some(init)) = &stmts[0] {
        assert_eq!(name, "result");
        if let Expr::Logical { left: l1, right: r1, .. } = &**init {
            if let Expr::Logical { left: l2, right: r2, .. } = &**l1 {
                assert_expr_eq(l2, &bool_lit(true));
                assert_expr_eq(r2, &bool_lit(false));
            } else {
                panic!("Expected logical expression");
            }
            assert_expr_eq(r1, &bool_lit(true));
        } else {
            panic!("Expected logical expression");
        }
    } else {
        panic!("Expected let statement");
    }
    
    // Verify second let statement (true and (false or true))
    if let Stmt::Let(name, Some(init)) = &stmts[1] {
        assert_eq!(name, "grouped");
        if let Expr::Logical { left: l1, right: r1, .. } = &**init {
            assert_expr_eq(l1, &bool_lit(true));
            if let Expr::Grouping(inner) = &**r1 {
                if let Expr::Logical { left: l2, right: r2, .. } = &**inner {
                    assert_expr_eq(l2, &bool_lit(false));
                    assert_expr_eq(r2, &bool_lit(true));
                } else {
                    panic!("Expected logical expression");
                }
            } else {
                panic!("Expected grouping");
            }
        } else {
            panic!("Expected logical expression");
        }
    } else {
        panic!("Expected let statement");
    }
}

#[test]
pub fn test_unary_expressions() {
    let source = r#"
        let negated = -42;
        let not_true = !true;
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 2);
    
    // Verify negation
    if let Stmt::Let(name, Some(init)) = &stmts[0] {
        assert_eq!(name, "negated");
        if let Expr::Unary { right, .. } = &**init {
            assert_expr_eq(right, &Expr::Literal(Value::new(ValueKind::Number(42.0))));
        } else {
            panic!("Expected unary expression");
        }
    } else {
        panic!("Expected let statement");
    }
    
    // Verify logical not
    if let Stmt::Let(name, Some(init)) = &stmts[1] {
        assert_eq!(name, "not_true");
        if let Expr::Unary { right, .. } = &**init {
            assert_expr_eq(right, &Expr::Literal(Value::new(ValueKind::Boolean(true))));
        } else {
            panic!("Expected unary expression");
        }
    } else {
        panic!("Expected let statement");
    }
}

#[test]
pub fn test_call_expressions() {
    let source = r#"
        let result = add(2, 3);
        let chained = obj.method().field;
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 2);
    
    // Verify function call
    if let Stmt::Let(name, Some(init)) = &stmts[0] {
        assert_eq!(name, "result");
        if let Expr::Call { callee, arguments } = &**init {
            assert_expr_eq(callee, &Expr::Variable("add".to_string()));
            assert_eq!(arguments.len(), 2);
            assert_expr_eq(&arguments[0], &Expr::Literal(Value::new(ValueKind::Number(2.0))));
            assert_expr_eq(&arguments[1], &Expr::Literal(Value::new(ValueKind::Number(3.0))));
        } else {
            panic!("Expected call expression");
        }
    } else {
        panic!("Expected let statement");
    }
    
    // Verify method call with property access
    if let Stmt::Let(name, Some(init)) = &stmts[1] {
        assert_eq!(name, "chained");
        if let Expr::Get { object, name: field_name } = &**init {
            assert_eq!(field_name, "field");
            if let Expr::Call { callee, arguments } = &**object {
                if let Expr::Get { object: obj, name: method_name } = &**callee {
                    assert_expr_eq(obj, &Expr::Variable("obj".to_string()));
                    assert_eq!(method_name, "method");
                } else {
                    panic!("Expected get expression");
                }
                assert_eq!(arguments.len(), 0);
            } else {
                panic!("Expected call expression");
            }
        } else {
            panic!("Expected get expression");
        }
    } else {
        panic!("Expected let statement");
    }
}

#[test]
fn test_confidence_expression() {
    let source = r#"
        let x = 42 ~> 0.9;
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 1);
    
    if let Stmt::Let(name, Some(init)) = &stmts[0] {
        assert_eq!(name, "x");
        if let Expr::Confidence { expr, confidence } = &**init {
            assert_expr_eq(expr, &Expr::Literal(Value::new(ValueKind::Number(42.0))));
            assert!((confidence - 0.9).abs() < f64::EPSILON);
        } else {
            panic!("Expected confidence expression");
        }
    } else {
        panic!("Expected let statement");
    }
}

#[test]
fn test_uncertain_if() {
    let source = r#"
        if confidence > 0.8 {
            print("high");
        } medium {
            print("medium");
        } low {
            print("low");
        }
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 1);
    
    if let Stmt::UncertainIf { condition, then_branch, medium_branch, low_branch } = &stmts[0] {
        assert!(medium_branch.is_some());
        assert!(low_branch.is_some());
    } else {
        panic!("Expected uncertain if statement");
    }
}

#[test]
fn test_context() {
    let source = r#"
        context "medical" {
            let diagnosis = "flu";
        }
    "#;
    
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let stmt = parser.parse().unwrap();
    
    match stmt.first().unwrap() {
        Stmt::Context { name, .. } => {
            assert_eq!(name, "medical");
        },
        _ => panic!("Expected Context statement"),
    }
}

#[test]
fn test_function_confidence() {
    let source = r#"
        fn diagnose(symptoms) ~> 0.8 {
            return "flu";
        }
    "#;
    
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let stmt = parser.parse().unwrap();
    
    match stmt.first().unwrap() {
        Stmt::Function { confidence, .. } => {
            assert!(confidence.is_some());
            assert!((confidence.unwrap() - 0.8).abs() < f64::EPSILON);
        },
        _ => panic!("Expected Function statement"),
    }
}

fn parse_expr(source: &str) -> Expr {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    parser.parse_expression().unwrap()
}

#[test]
fn test_number_literal() {
    let expr = parse_expr("42");
    assert_expr_eq(&expr, &Expr::Literal(Value::new(ValueKind::Number(42.0))));
}

#[test]
fn test_boolean_literal() {
    let expr = parse_expr("true");
    assert_expr_eq(&expr, &Expr::Literal(Value::new(ValueKind::Boolean(true))));
}

#[test]
fn test_string_literal() {
    let expr = parse_expr("\"hello\"");
    assert_expr_eq(&expr, &Expr::Literal(Value::new(ValueKind::String("hello".to_string()))));
}

#[test]
fn test_null_literal() {
    let expr = parse_expr("nil");
    assert_expr_eq(&expr, &Expr::Literal(Value::new(ValueKind::Nil)));
}

#[test]
fn test_parse_while_loop() {
    let source = r#"
        while (x < 10) {
            x = x + 1;
        }
    "#;
    
    let stmts = parse_stmt(source);
    assert_eq!(stmts.len(), 1);
    
    if let Stmt::While { condition, body } = &stmts[0] {
        // Verify condition and body
    } else {
        panic!("Expected while statement");
    }
}

#[test]
fn test_parse_block() {
    let source = "{ let x = 1; let y = 2; }".to_string();
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse().unwrap();

    assert_eq!(statements.len(), 1);
    match &statements[0] {
        Stmt::Block(statements) => {
            assert_eq!(statements.len(), 2);
            assert!(matches!(&statements[0], Stmt::Let(..)));
            assert!(matches!(&statements[1], Stmt::Let(..)));
        }
        _ => panic!("Expected block"),
    }
} 