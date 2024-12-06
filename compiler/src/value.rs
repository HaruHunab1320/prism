use std::fmt;
use std::sync::Arc;
use crate::ast::Stmt;
use crate::environment::Environment;

#[derive(Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub confidence: f64,
    pub context: Option<String>,
}

#[derive(Clone)]
pub enum ValueKind {
    Number(f64),
    String(String),
    Bool(bool),
    Function {
        name: String,
        params: Vec<String>,
        body: Stmt,
        closure: Arc<Environment>,
        is_async: bool,
    },
    Null,
}

impl Value {
    pub fn new(kind: ValueKind) -> Self {
        Self {
            kind,
            confidence: 1.0,
            context: None,
        }
    }

    pub fn with_confidence(kind: ValueKind, confidence: f64) -> Self {
        Self {
            kind,
            confidence: confidence.max(0.0).min(1.0),
            context: None,
        }
    }

    pub fn in_context(kind: ValueKind, context: String) -> Self {
        Self {
            kind,
            confidence: 1.0,
            context: Some(context),
        }
    }

    pub fn get_confidence(&self) -> f64 {
        self.confidence
    }

    pub fn get_context(&self) -> Option<&str> {
        self.context.as_deref()
    }

    pub fn combine_confidence(&self, other: f64) -> Self {
        Self {
            kind: self.kind.clone(),
            confidence: (self.confidence * other).max(0.0).min(1.0),
            context: self.context.clone(),
        }
    }

    pub fn as_number(&self) -> Option<f64> {
        match &self.kind {
            ValueKind::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match &self.kind {
            ValueKind::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match &self.kind {
            ValueKind::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        self.as_number()
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ValueKind::Number(n) => write!(f, "{} ~> {}", n, self.confidence),
            ValueKind::String(s) => write!(f, "\"{}\" ~> {}", s, self.confidence),
            ValueKind::Bool(b) => write!(f, "{} ~> {}", b, self.confidence),
            ValueKind::Function { name, .. } => write!(f, "<fn {}> ~> {}", name, self.confidence),
            ValueKind::Null => write!(f, "null ~> {}", self.confidence),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (&self.kind, &other.kind) {
            (ValueKind::Number(a), ValueKind::Number(b)) => (a - b).abs() < f64::EPSILON,
            (ValueKind::String(a), ValueKind::String(b)) => a == b,
            (ValueKind::Bool(a), ValueKind::Bool(b)) => a == b,
            (ValueKind::Null, ValueKind::Null) => true,
            _ => false,
        }
    }
} 