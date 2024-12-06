use prism::lexer::Lexer;
use prism::parser::Parser;
use prism::ast::{Expr, Stmt};
use prism::value::Value;

// Helper functions for AST verification
fn parse(source: &str) -> Vec<Stmt> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.lex().unwrap();
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
    match (actual, expected) {
        (Expr::Literal(v1), Expr::Literal(v2)) => assert_eq!(v1, v2),
        (Expr::Variable(n1), Expr::Variable(n2)) => assert_eq!(n1, n2),
        (Expr::Binary { left: l1, operator: op1, right: r1 },
         Expr::Binary { left: l2, operator: op2, right: r2 }) => {
            assert_eq!(op1.token_type, op2.token_type);
            assert_expr_eq(l1, l2);
            assert_expr_eq(r1, r2);
        }
        (Expr::Unary { operator: op1, right: r1 },
         Expr::Unary { operator: op2, right: r2 }) => {
            assert_eq!(op1.token_type, op2.token_type);
            assert_expr_eq(r1, r2);
        }
        _ => panic!("Expression mismatch: {:?} != {:?}", actual, expected),
    }
}

// Updated tests with AST verification
#[test]
pub fn test_parse_function_declaration() {
    let source = r#"
        fn add(x, y) {
            let result = x + y;
            result;
        }
    "#;
    
    let stmts = parse(source);
    assert_eq!(stmts.len(), 1);
    assert_function(&stmts[0], "add", &["x", "y"], false);
    
    if let Stmt::Function { body, .. } = &stmts[0] {
        if let Stmt::Block(block_stmts) = &**body {
            assert_eq!(block_stmts.len(), 2);
            // Verify let statement
            if let Stmt::Let(name, Some(init)) = &block_stmts[0] {
                assert_eq!(name, "result");
                if let Expr::Binary { left, right, .. } = &**init {
                    assert_expr_eq(left, &Expr::Variable("x".to_string()));
                    assert_expr_eq(right, &Expr::Variable("y".to_string()));
                } else {
                    panic!("Expected binary expression");
                }
            } else {
                panic!("Expected let statement");
            }
            // Verify return expression
            if let Stmt::Expression(expr) = &block_stmts[1] {
                assert_expr_eq(expr, &Expr::Variable("result".to_string()));
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
pub fn test_parse_async_function() {
    let source = r#"
        async fn fetch(url) {
            let response = http.get(url);
            response;
        }
    "#;
    
    let stmts = parse(source);
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
    
    let stmts = parse(source);
    assert_eq!(stmts.len(), 2);
    
    // Verify let statement
    assert_let(&stmts[0], "x", Some(&Expr::Literal(Value::Number(42.0))));
    
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
        if (x > 0) {
            let y = x + 1;
            y;
        }
    "#;
    
    let stmts = parse(source);
    assert_eq!(stmts.len(), 1);
    
    if let Stmt::If { condition, then_branch, else_branch } = &stmts[0] {
        // Verify condition
        if let Expr::Binary { left, right, .. } = &**condition {
            assert_expr_eq(left, &Expr::Variable("x".to_string()));
            assert_expr_eq(right, &Expr::Literal(Value::Number(0.0)));
        } else {
            panic!("Expected binary expression");
        }
        
        // Verify then branch
        if let Stmt::Block(block_stmts) = &**then_branch {
            assert_eq!(block_stmts.len(), 2);
            
            // Verify let statement
            if let Stmt::Let(name, Some(init)) = &block_stmts[0] {
                assert_eq!(name, "y");
                if let Expr::Binary { left, right, .. } = &**init {
                    assert_expr_eq(left, &Expr::Variable("x".to_string()));
                    assert_expr_eq(right, &Expr::Literal(Value::Number(1.0)));
                } else {
                    panic!("Expected binary expression");
                }
            } else {
                panic!("Expected let statement");
            }
            
            // Verify expression statement
            if let Stmt::Expression(expr) = &block_stmts[1] {
                assert_expr_eq(expr, &Expr::Variable("y".to_string()));
            } else {
                panic!("Expected expression statement");
            }
        } else {
            panic!("Expected block");
        }
        
        // Verify no else branch
        assert!(else_branch.is_none());
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
    
    let stmts = parse(source);
    assert_eq!(stmts.len(), 2);
    
    // Verify first let statement (1 + 2 * 3)
    if let Stmt::Let(name, Some(init)) = &stmts[0] {
        assert_eq!(name, "result");
        if let Expr::Binary { left: l1, right: r1, .. } = &**init {
            assert_expr_eq(l1, &Expr::Literal(Value::Number(1.0)));
            if let Expr::Binary { left: l2, right: r2, .. } = &**r1 {
                assert_expr_eq(l2, &Expr::Literal(Value::Number(2.0)));
                assert_expr_eq(r2, &Expr::Literal(Value::Number(3.0)));
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
                    assert_expr_eq(l2, &Expr::Literal(Value::Number(1.0)));
                    assert_expr_eq(r2, &Expr::Literal(Value::Number(2.0)));
                } else {
                    panic!("Expected binary expression");
                }
            } else {
                panic!("Expected grouping");
            }
            assert_expr_eq(r1, &Expr::Literal(Value::Number(3.0)));
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
    
    let stmts = parse(source);
    assert_eq!(stmts.len(), 2);
    
    // Helper function to create boolean literals
    let bool_lit = |b| Expr::Literal(Value::Bool(b));
    
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
    
    let stmts = parse(source);
    assert_eq!(stmts.len(), 2);
    
    // Verify negation
    if let Stmt::Let(name, Some(init)) = &stmts[0] {
        assert_eq!(name, "negated");
        if let Expr::Unary { right, .. } = &**init {
            assert_expr_eq(right, &Expr::Literal(Value::Number(42.0)));
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
            assert_expr_eq(right, &Expr::Literal(Value::Bool(true)));
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
        let result = add(1, 2);
        let chained = obj.method().field;
    "#;
    
    let stmts = parse(source);
    assert_eq!(stmts.len(), 2);
    
    // Verify function call
    if let Stmt::Let(name, Some(init)) = &stmts[0] {
        assert_eq!(name, "result");
        if let Expr::Call { callee, arguments } = &**init {
            assert_expr_eq(callee, &Expr::Variable("add".to_string()));
            assert_eq!(arguments.len(), 2);
            assert_expr_eq(&arguments[0], &Expr::Literal(Value::Number(1.0)));
            assert_expr_eq(&arguments[1], &Expr::Literal(Value::Number(2.0)));
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
    let source = "42 ~> 0.9";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let expr = parser.parse().unwrap();
    
    match expr.first().unwrap() {
        Stmt::Expression(expr) => {
            match &**expr {
                Expr::Confidence { confidence, .. } => {
                    assert!((confidence - 0.9).abs() < f64::EPSILON);
                },
                _ => panic!("Expected Confidence expression"),
            }
        },
        _ => panic!("Expected Expression statement"),
    }
}

#[test]
fn test_uncertain_if() {
    let source = r#"
        uncertain if (confidence > 0.8) {
            print("high");
        } medium {
            print("medium");
        } low {
            print("low");
        }
    "#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let stmt = parser.parse().unwrap();
    
    match stmt.first().unwrap() {
        Stmt::UncertainIf { medium_branch, low_branch, .. } => {
            assert!(medium_branch.is_some());
            assert!(low_branch.is_some());
        },
        _ => panic!("Expected UncertainIf statement"),
    }
}

#[test]
fn test_context_statement() {
    let source = r#"
        in context Medical {
            let diagnosis = "flu" ~> 0.85;
        }
    "#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let stmt = parser.parse().unwrap();
    
    match stmt.first().unwrap() {
        Stmt::Context { name, .. } => {
            assert_eq!(name, "Medical");
        },
        _ => panic!("Expected Context statement"),
    }
}

#[test]
fn test_function_with_confidence() {
    let source = r#"
        fn process(data) ~> 0.95 {
            return data * 2;
        }
    "#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.scan_tokens().unwrap();
    let mut parser = Parser::new(tokens);
    let stmt = parser.parse().unwrap();
    
    match stmt.first().unwrap() {
        Stmt::Function { confidence, .. } => {
            assert!(confidence.is_some());
            assert!((confidence.unwrap() - 0.95).abs() < f64::EPSILON);
        },
        _ => panic!("Expected Function statement"),
    }
} 