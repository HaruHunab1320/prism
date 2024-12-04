use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Float(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    Binary {
        left: Box<Expr>,
        operator: String,
        right: Box<Expr>,
    },
    Unary {
        operator: String,
        operand: Box<Expr>,
    },
    Call {
        function: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Index {
        array: Box<Expr>,
        index: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Let {
        name: String,
        initializer: Expr,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
    },
    Return(Expr),
    Break,
    Continue,
    TryCatch {
        try_block: Box<Stmt>,
        catch_variable: String,
        catch_block: Box<Stmt>,
    },
    Throw(Expr),
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Float(n) => write!(f, "{}", n),
            Expr::String(s) => write!(f, "\"{}\"", s),
            Expr::Boolean(b) => write!(f, "{}", b),
            Expr::Identifier(name) => write!(f, "{}", name),
            Expr::Array(elements) => {
                write!(f, "[")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, "]")
            }
            Expr::Object(fields) => {
                write!(f, "{{")?;
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, value)?;
                }
                write!(f, "}}")
            }
            Expr::Binary { left, operator, right } => {
                write!(f, "({} {} {})", left, operator, right)
            }
            Expr::Unary { operator, operand } => {
                write!(f, "({}{})", operator, operand)
            }
            Expr::Call { function, arguments } => {
                write!(f, "{}(", function)?;
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            Expr::Index { array, index } => {
                write!(f, "{}[{}]", array, index)
            }
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{};", expr),
            Stmt::Let { name, initializer } => write!(f, "let {} = {};", name, initializer),
            Stmt::Block(statements) => {
                writeln!(f, "{{")?;
                for stmt in statements {
                    writeln!(f, "    {}", stmt)?;
                }
                write!(f, "}}")
            }
            Stmt::If { condition, then_branch, else_branch } => {
                write!(f, "if {} {}", condition, then_branch)?;
                if let Some(else_branch) = else_branch {
                    write!(f, " else {}", else_branch)?;
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                write!(f, "while {} {}", condition, body)
            }
            Stmt::Function { name, params, body } => {
                write!(f, "fn {}(", name)?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") {}", body)
            }
            Stmt::Return(expr) => write!(f, "return {};", expr),
            Stmt::Break => write!(f, "break;"),
            Stmt::Continue => write!(f, "continue;"),
            Stmt::TryCatch { try_block, catch_variable, catch_block } => {
                write!(f, "try {} catch {} {}", try_block, catch_variable, catch_block)
            }
            Stmt::Throw(expr) => write!(f, "throw {};", expr),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expr_display() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Float(1.0)),
            operator: "+".to_string(),
            right: Box::new(Expr::Float(2.0)),
        };
        assert_eq!(expr.to_string(), "(1 + 2)");
    }

    #[test]
    fn test_stmt_display() {
        let stmt = Stmt::Function {
            name: "test".to_string(),
            params: vec!["x".to_string(), "y".to_string()],
            body: Box::new(Stmt::Return(Expr::Float(1.0))),
        };
        assert_eq!(
            stmt.to_string(),
            "fn test(x, y) {\n    return 1;\n}"
        );
    }
} 