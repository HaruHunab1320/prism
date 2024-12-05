use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::fmt;
use crate::interpreter::Interpreter;

pub type AsyncResult<T> = Pin<Box<dyn Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync>>> + Send + Sync>>;
pub type AsyncFn = Arc<dyn Fn(&Interpreter, Vec<Value>) -> AsyncResult<Value> + Send + Sync>;

#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    AsyncFn(AsyncFn),
    Pattern(String),
    Wildcard,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", v)?;
                }
                write!(f, "]")
            }
            Value::Object(obj) => {
                write!(f, "{{")?;
                for (i, (k, v)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {:?}", k, v)?;
                }
                write!(f, "}}")
            }
            Value::AsyncFn(_) => write!(f, "<function>"),
            Value::Pattern(p) => write!(f, "pattern(\"{}\")", p),
            Value::Wildcard => write!(f, "_"),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::Pattern(a), Value::Pattern(b)) => a == b,
            (Value::Wildcard, Value::Wildcard) => true,
            (Value::AsyncFn(_), Value::AsyncFn(_)) => false, // Functions are never equal
            _ => false,
        }
    }
}

impl Value {
    pub fn get_confidence(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum UnaryOp {
    Not,
    Minus,
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
}

#[derive(Clone, Debug)]
pub enum Expr {
    Literal(Value),
    Variable(String),
    Unary {
        operator: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        arguments: Vec<Expr>,
    },
    Match {
        value: Box<Expr>,
        arms: Vec<(Expr, Box<Expr>)>,
    },
    TryConfidence {
        body: Box<Expr>,
        threshold: f64,
        fallback: Box<Expr>,
        error_handler: Option<Box<Expr>>,
    },
    Verify {
        value: Box<Expr>,
        pattern: Box<Expr>,
    },
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Expression(Expr),
    Let(String, Expr),
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
        is_async: bool,
    },
    Return(Box<Expr>),
}
