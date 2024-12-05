use crate::types::Value;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expression(Expr),
    Let(String, Option<Expr>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    For(Option<Box<Stmt>>, Option<Expr>, Option<Expr>, Box<Stmt>),
    Function(String, Vec<String>, Box<Stmt>),
    Return(Option<Expr>),
    Context(String, Box<Stmt>),
    ContextTransition {
        from_context: String,
        to_context: String,
        confidence: Option<f64>,
        body: Box<Stmt>,
    },
    Verify(Vec<String>, Box<Stmt>),
    UncertainIf(Expr, Box<Stmt>, Option<Box<Stmt>>, Option<Box<Stmt>>),
    TryConfidence {
        body: Box<Stmt>,
        below_threshold: Box<Stmt>,
        uncertain: Box<Stmt>,
        threshold: f64,
    },
    Match {
        value: Box<Expr>,
        patterns: Vec<Expr>,
        bodies: Vec<Stmt>,
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
    Get(Box<Expr>, String),
    Logical(Box<Expr>, String, Box<Expr>),
    Grouping(Box<Expr>),
    ConfidenceFlow(Box<Expr>, Box<Expr>),
    ConfidenceAssign(Box<Expr>, f64),
    InContext(String, Box<Expr>),
    SemanticMatch(Box<Expr>, Box<Expr>),
    Tensor {
        values: Box<Vec<Expr>>,
        shape: Option<Vec<usize>>,
    },
    TensorOp {
        operation: String,
        operands: Vec<Expr>,
    },
} 