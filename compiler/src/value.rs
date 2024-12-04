use crate::error::RuntimeError;
use crate::interpreter::Interpreter;
use std::sync::Arc;

#[derive(Clone)]
pub enum Value {
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
    NativeFunction(Arc<dyn Fn(&mut Interpreter, Vec<Value>) -> Result<Value, RuntimeError> + Send + Sync>),
    AsyncFn(Arc<dyn Fn(Vec<Value>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value, RuntimeError>> + Send>> + Send + Sync>),
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(arr) => f.debug_list().entries(arr).finish(),
            Value::Object(obj) => {
                let mut map = f.debug_map();
                for (k, v) in obj {
                    map.entry(&k, &v);
                }
                map.finish()
            },
            Value::NativeFunction(_) => write!(f, "[native function]"),
            Value::AsyncFn(_) => write!(f, "[async function]"),
        }
    }
}

impl Value {
    pub fn get_type(&self) -> &'static str {
        match self {
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::NativeFunction(_) => "function",
            Value::AsyncFn(_) => "async_function",
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Float(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => {
                let elements: Vec<String> = arr.iter()
                    .map(|v| v.to_string())
                    .collect();
                format!("[{}]", elements.join(", "))
            },
            Value::Object(obj) => {
                let fields: Vec<String> = obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", fields.join(", "))
            },
            Value::NativeFunction(_) => "[native function]".to_string(),
            Value::AsyncFn(_) => "[async function]".to_string(),
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(RuntimeError::TypeError(format!("Cannot add {} and {}", self.get_type(), other.get_type()))),
        }
    }

    pub fn subtract(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            _ => Err(RuntimeError::TypeError(format!("Cannot subtract {} from {}", other.get_type(), self.get_type()))),
        }
    }

    pub fn multiply(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            _ => Err(RuntimeError::TypeError(format!("Cannot multiply {} and {}", self.get_type(), other.get_type()))),
        }
    }

    pub fn divide(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => {
                if *b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            _ => Err(RuntimeError::TypeError(format!("Cannot divide {} by {}", self.get_type(), other.get_type()))),
        }
    }

    pub fn equals(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean((a - b).abs() < f64::EPSILON)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a == b)),
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
            _ => Ok(Value::Boolean(false)),
        }
    }

    pub fn not_equals(&self, other: &Value) -> Result<Value, RuntimeError> {
        self.equals(other).map(|v| match v {
            Value::Boolean(b) => Value::Boolean(!b),
            _ => unreachable!(),
        })
    }

    pub fn less_than(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
            _ => Err(RuntimeError::TypeError(format!("Cannot compare {} and {}", self.get_type(), other.get_type()))),
        }
    }

    pub fn less_than_or_equal(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
            _ => Err(RuntimeError::TypeError(format!("Cannot compare {} and {}", self.get_type(), other.get_type()))),
        }
    }

    pub fn greater_than(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
            _ => Err(RuntimeError::TypeError(format!("Cannot compare {} and {}", self.get_type(), other.get_type()))),
        }
    }

    pub fn greater_than_or_equal(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
            _ => Err(RuntimeError::TypeError(format!("Cannot compare {} and {}", self.get_type(), other.get_type()))),
        }
    }

    pub fn and(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a && *b)),
            _ => Err(RuntimeError::TypeError(format!("Cannot perform logical AND on {} and {}", self.get_type(), other.get_type()))),
        }
    }

    pub fn or(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a || *b)),
            _ => Err(RuntimeError::TypeError(format!("Cannot perform logical OR on {} and {}", self.get_type(), other.get_type()))),
        }
    }

    pub fn not(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(RuntimeError::TypeError(format!("Cannot perform logical NOT on {}", self.get_type()))),
        }
    }

    pub fn negate(&self) -> Result<Value, RuntimeError> {
        match self {
            Value::Float(n) => Ok(Value::Float(-n)),
            _ => Err(RuntimeError::TypeError(format!("Cannot negate {}", self.get_type()))),
        }
    }

    pub fn get_property(&self, property: &Value) -> Result<Value, RuntimeError> {
        match (self, property) {
            (Value::Object(fields), Value::String(prop)) => {
                for (name, value) in fields {
                    if name == prop {
                        return Ok(value.clone());
                    }
                }
                Err(RuntimeError::UndefinedField(format!("Object has no property '{}'", prop)))
            }
            _ => Err(RuntimeError::TypeError(format!("Cannot get property of {} with {}", self.get_type(), property.get_type()))),
        }
    }

    pub fn get_index(&self, index: &Value) -> Result<Value, RuntimeError> {
        match (self, index) {
            (Value::Array(arr), Value::Float(i)) => {
                let i = *i as usize;
                if i < arr.len() {
                    Ok(arr[i].clone())
                } else {
                    Err(RuntimeError::IndexOutOfBounds(i, arr.len()))
                }
            }
            _ => Err(RuntimeError::TypeError(format!("Cannot index {} with {}", self.get_type(), index.get_type()))),
        }
    }

    pub fn with_confidence(&self, confidence: &Value) -> Result<Value, RuntimeError> {
        match confidence {
            Value::Float(n) if *n >= 0.0 && *n <= 1.0 => {
                match self {
                    Value::Object(fields) => {
                        let mut new_fields = fields.clone();
                        new_fields.push(("confidence".to_string(), Value::Float(*n)));
                        Ok(Value::Object(new_fields))
                    }
                    _ => {
                        let mut fields = Vec::new();
                        fields.push(("value".to_string(), self.clone()));
                        fields.push(("confidence".to_string(), Value::Float(*n)));
                        Ok(Value::Object(fields))
                    }
                }
            }
            _ => Err(RuntimeError::TypeError(format!("Confidence must be a float between 0 and 1, got {}", confidence.get_type()))),
        }
    }

    pub async fn call(&self, interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeError> {
        match self {
            Value::NativeFunction(f) => f(interpreter, args),
            Value::AsyncFn(f) => f(args).await,
            _ => Err(RuntimeError::TypeError(format!("Cannot call {}", self.get_type()))),
        }
    }
} 