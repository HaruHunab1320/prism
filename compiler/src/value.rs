use std::fmt;
use crate::ast::Stmt;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use crate::module::Module;

#[derive(Clone)]
pub enum ValueKind {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Function {
        name: String,
        params: Vec<String>,
        body: Box<Stmt>,
        is_async: bool,
        confidence: Option<f64>,
    },
    NativeFunction(Arc<dyn Fn(Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error + Send + Sync>>> + Send + Sync>> + Send + Sync>),
    Object(Arc<dyn std::any::Any + Send + Sync>),
    Module(Arc<RwLock<Module>>),
}

impl fmt::Debug for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::Nil => write!(f, "nil"),
            ValueKind::Boolean(b) => write!(f, "{}", b),
            ValueKind::Number(n) => write!(f, "{}", n),
            ValueKind::String(s) => write!(f, "\"{}\"", s),
            ValueKind::Function { name, .. } => write!(f, "<fn {}>", name),
            ValueKind::NativeFunction(_) => write!(f, "<native fn>"),
            ValueKind::Object(_) => write!(f, "<object>"),
            ValueKind::Module(m) => write!(f, "<module {}>", m.read().unwrap().name),
        }
    }
}

impl PartialEq for ValueKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueKind::Nil, ValueKind::Nil) => true,
            (ValueKind::Boolean(a), ValueKind::Boolean(b)) => a == b,
            (ValueKind::Number(a), ValueKind::Number(b)) => a == b,
            (ValueKind::String(a), ValueKind::String(b)) => a == b,
            (ValueKind::Function { name: n1, params: p1, body: b1, is_async: a1, confidence: c1 },
             ValueKind::Function { name: n2, params: p2, body: b2, is_async: a2, confidence: c2 }) => {
                n1 == n2 && p1 == p2 && b1 == b2 && a1 == a2 && c1 == c2
            }
            (ValueKind::Module(m1), ValueKind::Module(m2)) => {
                Arc::ptr_eq(m1, m2) || m1.read().unwrap().name == m2.read().unwrap().name
            }
            // NativeFunction and Object are compared by reference equality
            (ValueKind::NativeFunction(a), ValueKind::NativeFunction(b)) => Arc::ptr_eq(a, b),
            (ValueKind::Object(a), ValueKind::Object(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Value {
    pub kind: ValueKind,
    pub confidence: Option<f64>,
    pub context: Option<String>,
}

impl Value {
    pub fn new(kind: ValueKind) -> Self {
        Self {
            kind,
            confidence: None,
            context: None,
        }
    }

    pub fn with_confidence(kind: ValueKind, confidence: f64) -> Self {
        Self {
            kind,
            confidence: Some(confidence),
            context: None,
        }
    }

    pub fn in_context(kind: ValueKind, context: String) -> Self {
        Self {
            kind,
            confidence: None,
            context: Some(context),
        }
    }

    pub fn set_confidence(&mut self, confidence: f64) {
        self.confidence = Some(confidence);
    }

    pub fn get_confidence(&self) -> Option<f64> {
        self.confidence
    }

    pub fn set_context(&mut self, context: String) {
        self.context = Some(context);
    }

    pub fn get_context(&self) -> Option<&str> {
        self.context.as_deref()
    }

    pub fn is_truthy(&self) -> bool {
        match &self.kind {
            ValueKind::Nil => false,
            ValueKind::Boolean(b) => *b,
            _ => true,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.confidence == other.confidence && self.context == other.context
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::new(ValueKind::Boolean(b))
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::new(ValueKind::Number(n))
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::new(ValueKind::String(s))
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::new(ValueKind::String(s.to_string()))
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::new(ValueKind::Nil)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::new(ValueKind::Number(n as f64))
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::new(ValueKind::Number(n as f64))
    }
}

impl From<u32> for Value {
    fn from(n: u32) -> Self {
        Value::new(ValueKind::Number(n as f64))
    }
}

impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Value::new(ValueKind::Number(n as f64))
    }
} 