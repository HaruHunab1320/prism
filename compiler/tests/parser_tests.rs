use std::sync::Arc;
use prism::ast::{Expr, Stmt};
use prism::error::Result;
use prism::parser::Parser;
use prism::value::{Value, ValueKind};
use prism::token::{Token, TokenKind};

pub async fn test_parse_let_statement() -> Result<()> {
    let source = "let x = 42;";
    let mut parser = Parser::new(vec![
        Token::new(TokenKind::Let, "let".to_string(), 1),
        Token::new(TokenKind::Identifier("x".to_string()), "x".to_string(), 1),
        Token::new(TokenKind::Equal, "=".to_string(), 1),
        Token::new(TokenKind::Number(42.0), "42".to_string(), 1),
        Token::new(TokenKind::Semicolon, ";".to_string(), 1),
    ]);

    let stmt = parser.parse()?.pop().unwrap();
    match stmt {
        Stmt::Let(name, Some(expr)) => {
            assert_eq!(name, "x");
            match *expr {
                Expr::Literal(Value { kind: ValueKind::Number(n), .. }) => {
                    assert_eq!(n, 42.0);
                }
                _ => panic!("Expected number literal"),
            }
        }
        _ => panic!("Expected let statement"),
    }

    Ok(())
}

pub async fn test_parse_function_declaration() -> Result<()> {
    let source = "fn add(a, b) { return a + b; }";
    let mut parser = Parser::new(vec![
        Token::new(TokenKind::Fun, "fn".to_string(), 1),
        Token::new(TokenKind::Identifier("add".to_string()), "add".to_string(), 1),
        Token::new(TokenKind::LeftParen, "(".to_string(), 1),
        Token::new(TokenKind::Identifier("a".to_string()), "a".to_string(), 1),
        Token::new(TokenKind::Comma, ",".to_string(), 1),
        Token::new(TokenKind::Identifier("b".to_string()), "b".to_string(), 1),
        Token::new(TokenKind::RightParen, ")".to_string(), 1),
        Token::new(TokenKind::LeftBrace, "{".to_string(), 1),
        Token::new(TokenKind::Return, "return".to_string(), 1),
        Token::new(TokenKind::Identifier("a".to_string()), "a".to_string(), 1),
        Token::new(TokenKind::Plus, "+".to_string(), 1),
        Token::new(TokenKind::Identifier("b".to_string()), "b".to_string(), 1),
        Token::new(TokenKind::Semicolon, ";".to_string(), 1),
        Token::new(TokenKind::RightBrace, "}".to_string(), 1),
    ]);

    let stmt = parser.parse()?.pop().unwrap();
    match stmt {
        Stmt::Function { name, params, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params, vec!["a", "b"]);
        }
        _ => panic!("Expected function declaration"),
    }

    Ok(())
}

pub async fn test_parse_if_statement() -> Result<()> {
    let source = "if (x > 0) { return true; } else { return false; }";
    let mut parser = Parser::new(vec![
        Token::new(TokenKind::If, "if".to_string(), 1),
        Token::new(TokenKind::LeftParen, "(".to_string(), 1),
        Token::new(TokenKind::Identifier("x".to_string()), "x".to_string(), 1),
        Token::new(TokenKind::Greater, ">".to_string(), 1),
        Token::new(TokenKind::Number(0.0), "0".to_string(), 1),
        Token::new(TokenKind::RightParen, ")".to_string(), 1),
        Token::new(TokenKind::LeftBrace, "{".to_string(), 1),
        Token::new(TokenKind::Return, "return".to_string(), 1),
        Token::new(TokenKind::True, "true".to_string(), 1),
        Token::new(TokenKind::Semicolon, ";".to_string(), 1),
        Token::new(TokenKind::RightBrace, "}".to_string(), 1),
        Token::new(TokenKind::Else, "else".to_string(), 1),
        Token::new(TokenKind::LeftBrace, "{".to_string(), 1),
        Token::new(TokenKind::Return, "return".to_string(), 1),
        Token::new(TokenKind::False, "false".to_string(), 1),
        Token::new(TokenKind::Semicolon, ";".to_string(), 1),
        Token::new(TokenKind::RightBrace, "}".to_string(), 1),
    ]);

    let stmt = parser.parse()?.pop().unwrap();
    match stmt {
        Stmt::If { condition, then_branch, else_branch } => {
            match *condition {
                Expr::Binary { operator, .. } => {
                    assert_eq!(operator.kind, TokenKind::Greater);
                }
                _ => panic!("Expected binary expression"),
            }
            assert!(else_branch.is_some());
        }
        _ => panic!("Expected if statement"),
    }

    Ok(())
}

pub async fn test_parse_while_statement() -> Result<()> {
    let source = "while (x < 10) { x = x + 1; }";
    let mut parser = Parser::new(vec![
        Token::new(TokenKind::While, "while".to_string(), 1),
        Token::new(TokenKind::LeftParen, "(".to_string(), 1),
        Token::new(TokenKind::Identifier("x".to_string()), "x".to_string(), 1),
        Token::new(TokenKind::Less, "<".to_string(), 1),
        Token::new(TokenKind::Number(10.0), "10".to_string(), 1),
        Token::new(TokenKind::RightParen, ")".to_string(), 1),
        Token::new(TokenKind::LeftBrace, "{".to_string(), 1),
        Token::new(TokenKind::Identifier("x".to_string()), "x".to_string(), 1),
        Token::new(TokenKind::Equal, "=".to_string(), 1),
        Token::new(TokenKind::Identifier("x".to_string()), "x".to_string(), 1),
        Token::new(TokenKind::Plus, "+".to_string(), 1),
        Token::new(TokenKind::Number(1.0), "1".to_string(), 1),
        Token::new(TokenKind::Semicolon, ";".to_string(), 1),
        Token::new(TokenKind::RightBrace, "}".to_string(), 1),
    ]);

    let stmt = parser.parse()?.pop().unwrap();
    match stmt {
        Stmt::While { condition, .. } => {
            match *condition {
                Expr::Binary { operator, .. } => {
                    assert_eq!(operator.kind, TokenKind::Less);
                }
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected while statement"),
    }

    Ok(())
}

pub async fn test_parse_expression() -> Result<()> {
    let source = "2 + 3 * 4";
    let mut parser = Parser::new(vec![
        Token::new(TokenKind::Number(2.0), "2".to_string(), 1),
        Token::new(TokenKind::Plus, "+".to_string(), 1),
        Token::new(TokenKind::Number(3.0), "3".to_string(), 1),
        Token::new(TokenKind::Star, "*".to_string(), 1),
        Token::new(TokenKind::Number(4.0), "4".to_string(), 1),
    ]);

    let stmt = parser.parse()?.pop().unwrap();
    match stmt {
        Stmt::Expression(expr) => {
            match *expr {
                Expr::Binary { operator, .. } => {
                    assert_eq!(operator.kind, TokenKind::Plus);
                }
                _ => panic!("Expected binary expression"),
            }
        }
        _ => panic!("Expected expression statement"),
    }

    Ok(())
} 