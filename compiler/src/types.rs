use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::error::RuntimeError;
use crate::interpreter::Interpreter;

#[derive(Clone)]
pub enum Value {
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    Function(Arc<dyn Fn(Vec<Value>) -> Result<Value, RuntimeError> + Send + Sync>),
    AsyncFn(Arc<dyn Fn(Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + Send>> + Send + Sync>),
    NativeFunction(Arc<dyn Fn(&mut Interpreter, Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + Send>> + Send + Sync>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Float(n) => write!(f, "Float({})", n),
            Value::String(s) => write!(f, "String({})", s),
            Value::Boolean(b) => write!(f, "Boolean({})", b),
            Value::Array(elements) => f.debug_list().entries(elements).finish(),
            Value::Object(fields) => {
                f.debug_map().entries(fields.iter().map(|(k, v)| (k, v))).finish()
            }
            Value::Function(_) => write!(f, "Function"),
            Value::AsyncFn(_) => write!(f, "AsyncFn"),
            Value::NativeFunction(_) => write!(f, "NativeFunction"),
        }
    }
}

impl fmt::Display for Value {
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
                    write!(f, "{}: {}", name, value)?;
                }
                write!(f, "}}")
            }
            Value::Function(_) => write!(f, "<function>"),
            Value::AsyncFn(_) => write!(f, "<async function>"),
            Value::NativeFunction(_) => write!(f, "<native function>"),
        }
    }
} 