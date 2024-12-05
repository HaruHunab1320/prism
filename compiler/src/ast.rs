use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type AsyncFnOutput = Result<Value, Box<dyn Error + Send + Sync>>;
pub type AsyncFnFuture = Pin<Box<dyn Future<Output = AsyncFnOutput> + Send + Sync>>;
pub type AsyncFn =
    Arc<dyn Fn(&mut crate::interpreter::Interpreter, Vec<Value>) -> AsyncFnFuture + Send + Sync>;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    And,
    Or,
}

#[derive(Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<Value>),
    Object(HashMap<String, Value>),
    AsyncFn(AsyncFn),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::List(a), Value::List(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            (Value::AsyncFn(_), Value::AsyncFn(_)) => false, // Functions are never equal
            _ => false,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", item)?;
                }
                write!(f, "]")
            }
            Value::Object(map) => {
                write!(f, "{{")?;
                for (i, (key, value)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {:?}", key, value)?;
                }
                write!(f, "}}")
            }
            Value::AsyncFn(_) => write!(f, "<async function>"),
        }
    }
}

impl Value {
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn confidence(&self) -> Option<f64> {
        match self {
            Value::Object(obj) => obj.get("confidence").and_then(|v| v.as_number()),
            _ => None,
        }
    }
}
