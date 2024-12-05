use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use serde::{Serialize, Deserialize};

use crate::interpreter::Interpreter;

#[derive(Clone)]
pub enum Value {
    Void,
    Float(f64),
    String(String),
    Boolean(bool),
    Object(Vec<(String, Value)>),
    Array(Vec<Value>),
    NativeFunction(Arc<dyn Fn(&mut Interpreter, Vec<Value>) -> Result<Value, Box<dyn Error>> + Send + Sync>),
    AsyncFn(Arc<dyn Fn(Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error>>> + Send>> + Send + Sync>),
    Tensor(Vec<f64>, Vec<usize>),
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Object(fields) => {
                let mut debug_struct = f.debug_struct("Object");
                for (name, value) in fields {
                    debug_struct.field(name, value);
                }
                debug_struct.finish()
            },
            Value::Array(values) => {
                f.debug_list().entries(values).finish()
            },
            Value::NativeFunction(_) => write!(f, "[native function]"),
            Value::AsyncFn(_) => write!(f, "[async function]"),
            Value::Tensor(values, shape) => write!(f, "Tensor({:?}, {:?})", values, shape),
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Void, Value::Void) => true,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a == b,
            (Value::Object(a), Value::Object(b)) => {
                if a.len() != b.len() {
                    return false;
                }
                for (key, value) in a {
                    if let Some(other_value) = b.iter().find(|(k, _)| k == key) {
                        if value != &other_value.1 {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                true
            },
            (Value::Tensor(a_values, a_shape), Value::Tensor(b_values, b_shape)) => {
                a_values == b_values && a_shape == b_shape
            },
            _ => false,
        }
    }
}

impl Value {
    pub fn get_type(&self) -> &'static str {
        match self {
            Value::Void => "void",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Object(_) => "object",
            Value::Array(_) => "array",
            Value::NativeFunction(_) => "function",
            Value::AsyncFn(_) => "async_function",
            Value::Tensor(_, _) => "tensor",
        }
    }

    pub fn get_confidence(&self) -> Option<f64> {
        match self {
            Value::Object(fields) => {
                fields.iter()
                    .find(|(name, _)| name == "confidence")
                    .and_then(|(_, value)| {
                        if let Value::Float(n) = value {
                            Some(*n)
                        } else {
                            None
                        }
                    })
            }
            _ => None,
        }
    }

    pub fn with_confidence(&self, confidence: f64) -> Result<Value, Box<dyn Error>> {
        if confidence < 0.0 || confidence > 1.0 {
            return Err("Confidence must be between 0 and 1".into());
        }

        match self {
            Value::Object(fields) => {
                let mut new_fields = fields.clone();
                if let Some(pos) = new_fields.iter().position(|(name, _)| name == "confidence") {
                    new_fields[pos] = ("confidence".to_string(), Value::Float(confidence));
                } else {
                    new_fields.push(("confidence".to_string(), Value::Float(confidence)));
                }
                Ok(Value::Object(new_fields))
            }
            _ => {
                let mut fields = Vec::new();
                fields.push(("value".to_string(), self.clone()));
                fields.push(("confidence".to_string(), Value::Float(confidence)));
                Ok(Value::Object(fields))
            }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::Void => "void".to_string(),
            Value::Float(n) => n.to_string(),
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Object(fields) => {
                let mut result = String::from("{");
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&format!("{}: {}", name, value.to_string()));
                }
                result.push('}');
                result
            }
            Value::Array(values) => {
                let mut result = String::from("[");
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&value.to_string());
                }
                result.push(']');
                result
            }
            Value::NativeFunction(_) => "[native function]".to_string(),
            Value::AsyncFn(_) => "[async function]".to_string(),
            Value::Tensor(values, shape) => format!("Tensor({:?}, {:?})", values, shape),
        }
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Void => serializer.serialize_none(),
            Value::Float(n) => serializer.serialize_f64(*n),
            Value::String(s) => serializer.serialize_str(s),
            Value::Boolean(b) => serializer.serialize_bool(*b),
            Value::Object(fields) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(fields.len()))?;
                for (key, value) in fields {
                    map.serialize_entry(key, value)?;
                }
                map.end()
            }
            Value::Array(values) => values.serialize(serializer),
            Value::NativeFunction(_) => serializer.serialize_str("[native function]"),
            Value::AsyncFn(_) => serializer.serialize_str("[async function]"),
            Value::Tensor(values, shape) => {
                use serde::ser::SerializeStruct;
                let mut s = serializer.serialize_struct("Tensor", 2)?;
                s.serialize_field("values", values)?;
                s.serialize_field("shape", shape)?;
                s.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, Visitor, MapAccess, SeqAccess};
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
                Ok(Value::Void)
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Value, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Value::deserialize(deserializer)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut values = Vec::new();
                while let Some(value) = seq.next_element()? {
                    values.push(value);
                }
                Ok(Value::Array(values))
            }

            fn visit_map<M>(self, mut access: M) -> Result<Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut fields = Vec::new();
                while let Some((key, value)) = access.next_entry()? {
                    fields.push((key, value));
                }
                Ok(Value::Object(fields))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
} 