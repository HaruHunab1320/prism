use std::error::Error;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};
use crate::interpreter::Interpreter;

#[derive(Clone)]
pub enum Value {
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    NativeFunction(Arc<dyn Fn(&mut Interpreter, Vec<Value>) -> Result<Value, Box<dyn Error>> + Send + Sync>),
    AsyncFn(Arc<dyn Fn(Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error>>> + Send>> + Send + Sync>),
    Tensor(Vec<f64>, Vec<usize>),
    None,
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "none"),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(arr) => write!(f, "{:?}", arr),
            Value::Object(obj) => write!(f, "{:?}", obj),
            Value::NativeFunction(_) => write!(f, "<native function>"),
            Value::AsyncFn(_) => write!(f, "<async function>"),
            Value::Tensor(values, shape) => write!(f, "Tensor({:?}, shape={:?})", values, shape),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (key, value) in a {
                    if let Some(other_value) = b.get(key) {
                        if value != other_value {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            }
            (Value::Tensor(a_values, a_shape), Value::Tensor(b_values, b_shape)) => {
                a_shape == b_shape && a_values == b_values
            }
            _ => false,
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
            Value::Tensor(_, _) => "tensor",
            Value::None => "none",
        }
    }

    pub fn get_confidence(&self) -> Option<f64> {
        match self {
            Value::Object(obj) => obj.get("confidence").and_then(|v| v.as_float()),
            _ => None,
        }
    }

    pub fn get_context(&self) -> Option<String> {
        match self {
            Value::Object(obj) => obj.get("context").and_then(|v| v.as_string()),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::String(s) => s.parse().ok(),
            Value::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            Value::String(s) => Some(s.clone()),
            Value::Float(f) => Some(f.to_string()),
            Value::Boolean(b) => Some(b.to_string()),
            Value::None => Some("none".to_string()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean(b) => Some(*b),
            Value::Float(f) => Some(*f != 0.0),
            Value::String(s) => Some(!s.is_empty()),
            Value::None => Some(false),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn with_confidence(&self, confidence: f64) -> Result<Value, Box<dyn Error>> {
        if confidence < 0.0 || confidence > 1.0 {
            return Err("Confidence must be between 0 and 1".into());
        }

        let mut obj = HashMap::new();
        obj.insert("value".to_string(), self.clone());
        obj.insert("confidence".to_string(), Value::Float(confidence));
        Ok(Value::Object(obj))
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Float(f) => f.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Object(obj) => {
                let items: Vec<String> = obj
                    .iter()
                    .map(|(k, v)| format!("\"{}\": {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", items.join(", "))
            }
            Value::NativeFunction(_) => "<native function>".to_string(),
            Value::AsyncFn(_) => "<async function>".to_string(),
            Value::Tensor(values, shape) => {
                format!("Tensor({:?}, shape={:?})", values, shape)
            }
            Value::None => "none".to_string(),
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::None => serializer.serialize_none(),
            Value::Float(n) => serializer.serialize_f64(*n),
            Value::String(s) => serializer.serialize_str(s),
            Value::Boolean(b) => serializer.serialize_bool(*b),
            Value::Array(arr) => arr.serialize(serializer),
            Value::Object(obj) => obj.serialize(serializer),
            Value::NativeFunction(_) => serializer.serialize_str("<native function>"),
            Value::AsyncFn(_) => serializer.serialize_str("<async function>"),
            Value::Tensor(values, shape) => {
                use serde::ser::SerializeStruct;
                let mut state = serializer.serialize_struct("Tensor", 2)?;
                state.serialize_field("values", values)?;
                state.serialize_field("shape", shape)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor};
        use std::fmt;

        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid Prism value")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Boolean(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Float(value as f64))
            }

            fn visit_u64<E>(self, value: u64) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Float(value as f64))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::Float(value))
            }

            fn visit_str<E>(self, value: &str) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(value.to_string()))
            }

            fn visit_string<E>(self, value: String) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::String(value))
            }

            fn visit_none<E>(self) -> Result<Value, E>
            where
                E: de::Error,
            {
                Ok(Value::None)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Value::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(Value::Array(values))
            }

            fn visit_map<M>(self, mut access: M) -> Result<Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut map = HashMap::new();
                while let Some((key, value)) = access.next_entry()? {
                    map.insert(key, value);
                }
                Ok(Value::Object(map))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
} 