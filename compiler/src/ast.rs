use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    // Literals
    Float(f64),
    Integer(i64),
    String(String),
    Identifier(String),
    
    // Confidence operations
    ConfidenceValue {
        value: Box<Expr>,
    },
    ConfidenceFlow {
        source: Box<Expr>,
        target: Box<Expr>,
    },
    ReverseConfidenceFlow {
        target: Box<Expr>,
        source: Box<Expr>,
    },
    
    // Control flow
    UncertainIf {
        condition: Box<Expr>,
        high_confidence: Vec<Stmt>,
        medium_confidence: Option<Vec<Stmt>>,
        low_confidence: Option<Vec<Stmt>>,
    },
    
    // Context operations
    ContextBlock {
        context_name: String,
        body: Vec<Stmt>,
    },
    ContextShift {
        from: String,
        to: String,
        body: Vec<Stmt>,
    },
    
    // Verification
    Verify {
        sources: Vec<String>,
        threshold: f64,
        body: Vec<Stmt>,
    },
    
    // Binary operations
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Expr),
    Declaration {
        name: String,
        value: Expr,
    },
    Assignment {
        target: String,
        value: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    ConfidenceAnd,
    ConfidenceOr,
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Float(val) => write!(f, "{}", val),
            Expr::Integer(val) => write!(f, "{}", val),
            Expr::String(val) => write!(f, "\"{}\"", val),
            Expr::Identifier(name) => write!(f, "{}", name),
            Expr::ConfidenceValue { value } => write!(f, "conf {}", value),
            Expr::ConfidenceFlow { source, target } => write!(f, "{} ~> {}", source, target),
            Expr::ReverseConfidenceFlow { target, source } => write!(f, "{} <~ {}", target, source),
            _ => write!(f, "{:?}", self),
        }
    }
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stmt::Expression(expr) => write!(f, "{}", expr),
            Stmt::Declaration { name, value } => write!(f, "conf {} = {}", name, value),
            Stmt::Assignment { target, value } => write!(f, "{} = {}", target, value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_flow_expr() {
        let expr = Expr::ConfidenceFlow {
            source: Box::new(Expr::Identifier("x".to_string())),
            target: Box::new(Expr::Float(0.7)),
        };
        assert_eq!(expr.to_string(), "x ~> 0.7");
    }

    #[test]
    fn test_declaration_stmt() {
        let stmt = Stmt::Declaration {
            name: "x".to_string(),
            value: Expr::Float(0.8),
        };
        assert_eq!(stmt.to_string(), "conf x = 0.8");
    }
} 