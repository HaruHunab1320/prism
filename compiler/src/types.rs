use crate::ast::Stmt;
use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone)]
pub enum Value {
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Function {
        name: String,
        params: Vec<String>,
        body: Arc<Stmt>,
        closure: HashMap<String, Value>,
    },
    NativeFunction(Arc<dyn Fn(&mut Interpreter, Vec<Value>) -> Result<Value, RuntimeError> + Send + Sync>),
    AsyncFn(Arc<dyn Fn(Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + Send>> + Send + Sync>),
}

unsafe impl Send for Value {}
unsafe impl Sync for Value {}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(elements) => {
                write!(f, "[")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{:?}", element)?;
                }
                write!(f, "]")
            }
            Value::Object(fields) => {
                write!(f, "{{")?;
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {:?}", name, value)?;
                }
                write!(f, "}}")
            }
            Value::Function { name, .. } => write!(f, "<function {}>", name),
            Value::NativeFunction(_) => write!(f, "<native function>"),
            Value::AsyncFn(_) => write!(f, "<async function>"),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(elements) => {
                write!(f, "[")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, "]")
            }
            Value::Object(fields) => {
                write!(f, "{{")?;
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", name, value)?;
                }
                write!(f, "}}")
            }
            Value::Function { name, .. } => write!(f, "<function {}>", name),
            Value::NativeFunction(_) => write!(f, "<native function>"),
            Value::AsyncFn(_) => write!(f, "<async function>"),
        }
    }
}

impl Value {
    pub fn get_type(&self) -> &'static str {
        match self {
            Value::Float(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Function { .. } => "function",
            Value::NativeFunction(_) => "native function",
            Value::AsyncFn(_) => "async function",
        }
    }
} 