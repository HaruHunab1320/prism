use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::interpreter::Interpreter;
use crate::token::Token;
use crate::value::{Value, ValueKind};

pub type AsyncResult<T> = Pin<Box<dyn Future<Output = Result<T, Box<dyn Error + Send + Sync>>> + Send + Sync>>;
pub type AsyncFn = Arc<dyn Fn(&Interpreter, Vec<Value>) -> AsyncResult<Value> + Send + Sync>;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Get {
        object: Box<Expr>,
        name: String,
    },
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Confidence {
        expr: Box<Expr>,
        confidence: f64,
    },
    ConfidenceCombine {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    InContext {
        context: String,
        body: Box<Expr>,
    },
    Grouping(Box<Expr>),
    ModuleAccess {
        module: String,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Expression(Box<Expr>),
    Let(String, Option<Box<Expr>>),
    Block(Vec<Stmt>),
    If {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    UncertainIf {
        condition: Box<Expr>,
        then_branch: Box<Stmt>,
        medium_branch: Option<Box<Stmt>>,
        low_branch: Option<Box<Stmt>>,
    },
    While {
        condition: Box<Expr>,
        body: Box<Stmt>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
        is_async: bool,
        confidence: Option<f64>,
    },
    Return(Option<Box<Expr>>),
    Context {
        name: String,
        body: Box<Stmt>,
    },
    Import {
        module: String,
        imports: Vec<(String, Option<String>)>, // (name, alias)
        confidence: Option<f64>,
    },
    Export(String, Box<Stmt>), // name and the statement being exported
    Module {
        name: String,
        body: Vec<Stmt>,
        confidence: Option<f64>,
    },
    ModuleAccess {
        module_name: String,
        name: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOp {
    Not,
    Minus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}

impl From<&Stmt> for Expr {
    fn from(stmt: &Stmt) -> Self {
        match stmt {
            Stmt::Expression(expr) => *expr.clone(),
            Stmt::Let(name, Some(expr)) => Expr::Assign {
                name: name.clone(),
                value: expr.clone(),
            },
            Stmt::Let(name, None) => Expr::Variable(name.clone()),
            Stmt::Block(stmts) => Expr::Grouping(Box::new(
                stmts.last()
                    .map(|s| Self::from(s))
                    .unwrap_or(Expr::Literal(Value::new(ValueKind::Nil)))
            )),
            _ => Expr::Literal(Value::new(ValueKind::Nil)),
        }
    }
}
