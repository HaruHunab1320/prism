use prism::parser;
use prism::error::Result;
use prism::ast::{Stmt, Expr};

pub fn test_parse_let_statement() -> Result<()> {
    let source = "let x = 42;";
    let statements = parser::parse(source)?;
    assert_eq!(statements.len(), 1);
    match &statements[0] {
        Stmt::Let(name, Some(expr)) => {
            assert_eq!(name, "x");
            match expr {
                Expr::Literal(value) => {
                    assert_eq!(value.kind, prism::value::ValueKind::Number(42.0));
                }
                _ => panic!("Expected literal expression"),
            }
        }
        _ => panic!("Expected let statement"),
    }
    Ok(())
}

pub fn test_parse_function() -> Result<()> {
    let source = "fn add(a, b) { return a + b; }";
    let statements = parser::parse(source)?;
    assert_eq!(statements.len(), 1);
    match &statements[0] {
        Stmt::Function { name, params, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert_eq!(params[0], "a");
            assert_eq!(params[1], "b");
        }
        _ => panic!("Expected function statement"),
    }
    Ok(())
}

pub fn test_parse_if_statement() -> Result<()> {
    let source = "if (x > 5) { x = 10; } else { x = 0; }";
    let statements = parser::parse(source)?;
    assert_eq!(statements.len(), 1);
    match &statements[0] {
        Stmt::If { condition, then_branch, else_branch } => {
            assert!(matches!(condition, Expr::Binary { .. }));
            assert!(matches!(then_branch, Box::new(Stmt::Block(_))));
            assert!(matches!(else_branch, Some(Box::new(Stmt::Block(_)))));
        }
        _ => panic!("Expected if statement"),
    }
    Ok(())
}

pub fn test_parse_while_statement() -> Result<()> {
    let source = "while (x < 10) { x = x + 1; }";
    let statements = parser::parse(source)?;
    assert_eq!(statements.len(), 1);
    match &statements[0] {
        Stmt::While { condition, body } => {
            assert!(matches!(condition, Expr::Binary { .. }));
            assert!(matches!(body, Box::new(Stmt::Block(_))));
        }
        _ => panic!("Expected while statement"),
    }
    Ok(())
}

pub fn test_parse_block_statement() -> Result<()> {
    let source = "{ let x = 1; x = x + 1; }";
    let statements = parser::parse(source)?;
    assert_eq!(statements.len(), 1);
    match &statements[0] {
        Stmt::Block(stmts) => {
            assert_eq!(stmts.len(), 2);
            assert!(matches!(&stmts[0], Stmt::Let(..)));
            assert!(matches!(&stmts[1], Stmt::Expression(..)));
        }
        _ => panic!("Expected block statement"),
    }
    Ok(())
} 