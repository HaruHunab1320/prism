use std::collections::HashMap;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::fmt;

#[derive(Clone)]
pub enum Value {
    None,
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    NativeFunction(Arc<dyn Fn(&mut crate::interpreter::Interpreter, Vec<Value>) -> Result<Value, Box<dyn Error + Send + Sync>> + Send + Sync>),
    AsyncFn(Arc<dyn Fn(Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error + Send + Sync>>> + Send + Sync>> + Send + Sync>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "None"),
            Value::Float(x) => write!(f, "{}", x),
            Value::String(s) => write!(f, "{:?}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(arr) => write!(f, "{:?}", arr),
            Value::Object(obj) => write!(f, "{:?}", obj),
            Value::NativeFunction(_) => write!(f, "<native function>"),
            Value::AsyncFn(_) => write!(f, "<async function>"),
        }
    }
}

impl Value {
    pub fn with_confidence(self, confidence: f64) -> Result<Value, Box<dyn Error + Send + Sync>> {
        if confidence < 0.0 || confidence > 1.0 {
            return Err("Confidence value must be between 0 and 1".into());
        }
        Ok(Value::Object(HashMap::from([
            ("value".to_string(), self),
            ("confidence".to_string(), Value::Float(confidence)),
        ])))
    }

    pub fn get_confidence(&self) -> Option<f64> {
        match self {
            Value::Object(obj) => {
                if let Some(Value::Float(confidence)) = obj.get("confidence") {
                    Some(*confidence)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr.clone()),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<HashMap<String, Value>> {
        match self {
            Value::Object(obj) => Some(obj.clone()),
            _ => None,
        }
    }
} 