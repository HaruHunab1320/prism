use std::fmt;
use std::sync::Arc;
use parking_lot::RwLock;
use crate::module::Module;
use crate::error::Result;

#[derive(Clone)]
pub enum ValueKind {
    Nil,
    Boolean(bool),
    Number(f64),
    String(String),
    Function {
        name: String,
        params: Vec<String>,
        body: Arc<dyn Fn(Vec<Value>) -> Result<Value> + Send + Sync>,
    },
    NativeFunction {
        name: String,
        arity: usize,
        handler: Arc<dyn Fn(Vec<Value>) -> Result<Value> + Send + Sync>,
    },
    Module(Arc<RwLock<Module>>),
    List(Vec<Value>),
    Map(Vec<(Value, Value)>),
}

impl fmt::Debug for ValueKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueKind::Nil => write!(f, "Nil"),
            ValueKind::Boolean(b) => write!(f, "Boolean({})", b),
            ValueKind::Number(n) => write!(f, "Number({})", n),
            ValueKind::String(s) => write!(f, "String({})", s),
            ValueKind::Function { name, .. } => write!(f, "Function({})", name),
            ValueKind::NativeFunction { name, .. } => write!(f, "NativeFunction({})", name),
            ValueKind::Module(m) => {
                let module = m.read();
                write!(f, "Module({})", module.name)
            },
            ValueKind::List(items) => f.debug_list().entries(items).finish(),
            ValueKind::Map(entries) => {
                let mut map = f.debug_map();
                for (k, v) in entries {
                    map.entry(k, v);
                }
                map.finish()
            }
        }
    }
}

impl PartialEq for ValueKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ValueKind::Nil, ValueKind::Nil) => true,
            (ValueKind::Boolean(a), ValueKind::Boolean(b)) => a == b,
            (ValueKind::Number(a), ValueKind::Number(b)) => (a - b).abs() < f64::EPSILON,
            (ValueKind::String(a), ValueKind::String(b)) => a == b,
            (ValueKind::Function { name: n1, .. }, ValueKind::Function { name: n2, .. }) => n1 == n2,
            (ValueKind::NativeFunction { name: n1, .. }, ValueKind::NativeFunction { name: n2, .. }) => n1 == n2,
            (ValueKind::Module(m1), ValueKind::Module(m2)) => {
                Arc::ptr_eq(&m1, &m2) || {
                    let m1 = m1.read();
                    let m2 = m2.read();
                    m1.name == m2.name
                }
            }
            (ValueKind::List(a), ValueKind::List(b)) => a == b,
            (ValueKind::Map(a), ValueKind::Map(b)) => a == b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Value {
    pub kind: ValueKind,
    pub confidence: f64,
    pub context: Option<String>,
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
            confidence,
            context: None,
        }
    }

    pub fn with_context(kind: ValueKind, context: String) -> Self {
        Self {
            kind,
            confidence: 1.0,
            context: Some(context),
        }
    }

    pub fn with_confidence_and_context(kind: ValueKind, confidence: f64, context: String) -> Self {
        Self {
            kind,
            confidence,
            context: Some(context),
        }
    }

    pub fn get_confidence(&self) -> Option<f64> {
        Some(self.confidence)
    }

    pub fn set_confidence(&mut self, confidence: f64) {
        self.confidence = confidence;
    }

    pub fn get_context(&self) -> Option<&str> {
        self.context.as_deref()
    }

    pub fn set_context(&mut self, context: String) {
        self.context = Some(context);
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ValueKind::Nil => write!(f, "nil"),
            ValueKind::Boolean(b) => write!(f, "{}", b),
            ValueKind::Number(n) => write!(f, "{}", n),
            ValueKind::String(s) => write!(f, "{}", s),
            ValueKind::Function { name, .. } => write!(f, "<fn {}>", name),
            ValueKind::NativeFunction { name, .. } => write!(f, "<native fn {}>", name),
            ValueKind::Module(m) => {
                let module = m.read();
                write!(f, "<module {}>", module.name)
            },
            ValueKind::List(items) => {
                write!(f, "[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            ValueKind::Map(entries) => {
                write!(f, "{{")?;
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        if self.confidence != other.confidence {
            return false;
        }
        if self.context != other.context {
            return false;
        }
        self.kind == other.kind
    }
} 