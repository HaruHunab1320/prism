use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Let(String, Expr),
    Block(Vec<Stmt>),
    Context(String, Box<Stmt>),
    ContextTransition {
        from_context: String,
        to_context: String,
        confidence: f64,
        body: Box<Stmt>,
    },
    Verify(Vec<String>, Box<Stmt>),
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
        is_async: bool,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Assign(String, Box<Expr>),
    Binary(Box<Expr>, String, Box<Expr>),
    Unary(String, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Confidence(Box<Expr>, f64),
    SemanticMatch(Box<Expr>, Box<Expr>),
    Match {
        value: Box<Expr>,
        arms: Vec<(Expr, (f64, f64), Expr)>,
    },
    UncertainIf {
        conditions: Vec<(Expr, f64, Expr)>,
    },
    TryConfidence {
        body: Box<Expr>,
        threshold: f64,
        fallback: Box<Expr>,
        error_handler: Box<Expr>,
    },
    Await(Box<Expr>),
} 