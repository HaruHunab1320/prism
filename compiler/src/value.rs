use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::interpreter::Interpreter;
use crate::error::RuntimeError;
use crate::types::Value;

impl Value {
    pub fn get_type(&self) -> &'static str {
        match self {
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
            Value::Function(_) => "function",
            Value::AsyncFn(_) => "async_function",
            Value::NativeFunction(_) => "native_function",
        }
    }

    pub fn get_confidence(&self) -> Option<f64> {
        match self {
            Value::Float(n) => Some(*n),
            Value::Object(fields) => {
                for (name, value) in fields {
                    if name == "confidence" {
                        if let Value::Float(n) = value {
                            return Some(*n);
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }

    pub fn add(&self, other: &Value) -> Result<Value, RuntimeError> {
        match (self, other) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            (Value::Array(a), Value::Array(b)) => {
                let mut result = a.clone();
                result.extend(b.clone());
                Ok(Value::Array(result))
            }
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
            (Value::Array(arr), Value::String(prop)) => {
                match prop.as_str() {
                    "length" => Ok(Value::Float(arr.len() as f64)),
                    "join" => {
                        let arr = arr.clone();
                        Ok(Value::Function(Arc::new(move |args: Vec<Value>| {
                            if args.len() != 1 {
                                return Err(RuntimeError::TypeError(format!("join() takes exactly 1 argument, got {}", args.len())));
                            }
                            let separator = match &args[0] {
                                Value::String(s) => s,
                                _ => return Err(RuntimeError::TypeError(format!("join() argument must be a string, got {}", args[0].get_type()))),
                            };
                            let strings: Result<Vec<String>, RuntimeError> = arr.iter().map(|v| match v {
                                Value::String(s) => Ok(s.clone()),
                                _ => Err(RuntimeError::TypeError(format!("Cannot join array containing {}", v.get_type()))),
                            }).collect();
                            match strings {
                                Ok(strings) => Ok(Value::String(strings.join(separator))),
                                Err(e) => Err(e),
                            }
                        })))
                    }
                    _ => Err(RuntimeError::UndefinedField(format!("Array has no property '{}'", prop))),
                }
            }
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
            Value::Function(f) => f(args),
            Value::AsyncFn(f) => f(args).await,
            Value::NativeFunction(f) => f(interpreter, args).await,
            _ => Err(RuntimeError::TypeError(format!("Cannot call {}", self.get_type()))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Float(42.0).to_string(), "42");
        assert_eq!(Value::String("hello".to_string()).to_string(), "\"hello\"");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::Array(vec![Value::Float(1.0), Value::Float(2.0)]).to_string(), "[1, 2]");
        assert_eq!(Value::Object(vec![
            ("name".to_string(), Value::String("test".to_string())),
            ("value".to_string(), Value::Float(42.0)),
        ]).to_string(), "{name: \"test\", value: 42}");
    }

    #[test]
    fn test_value_confidence() {
        assert_eq!(Value::Float(0.5).get_confidence(), Some(0.5));
        assert_eq!(Value::String("test".to_string()).get_confidence(), None);
        assert_eq!(Value::Object(vec![
            ("confidence".to_string(), Value::Float(0.8)),
            ("value".to_string(), Value::String("test".to_string())),
        ]).get_confidence(), Some(0.8));
    }

    #[test]
    fn test_value_operations() {
        // Addition
        assert_eq!(
            Value::Float(1.0).add(&Value::Float(2.0)).unwrap(),
            Value::Float(3.0)
        );
        assert_eq!(
            Value::String("hello ".to_string()).add(&Value::String("world".to_string())).unwrap(),
            Value::String("hello world".to_string())
        );

        // Subtraction
        assert_eq!(
            Value::Float(3.0).subtract(&Value::Float(2.0)).unwrap(),
            Value::Float(1.0)
        );

        // Multiplication
        assert_eq!(
            Value::Float(2.0).multiply(&Value::Float(3.0)).unwrap(),
            Value::Float(6.0)
        );

        // Division
        assert_eq!(
            Value::Float(6.0).divide(&Value::Float(2.0)).unwrap(),
            Value::Float(3.0)
        );

        // Comparison
        assert_eq!(
            Value::Float(1.0).less_than(&Value::Float(2.0)).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            Value::Float(2.0).greater_than(&Value::Float(1.0)).unwrap(),
            Value::Boolean(true)
        );
        assert_eq!(
            Value::Float(1.0).equals(&Value::Float(1.0)).unwrap(),
            Value::Boolean(true)
        );
    }

    #[test]
    fn test_value_property_access() {
        let arr = Value::Array(vec![Value::String("a".to_string()), Value::String("b".to_string())]);
        assert_eq!(
            arr.get_property(&Value::String("length".to_string())).unwrap(),
            Value::Float(2.0)
        );

        let obj = Value::Object(vec![
            ("name".to_string(), Value::String("test".to_string())),
            ("value".to_string(), Value::Float(42.0)),
        ]);
        assert_eq!(
            obj.get_property(&Value::String("name".to_string())).unwrap(),
            Value::String("test".to_string())
        );
    }

    #[test]
    fn test_value_confidence_flow() {
        let value = Value::String("test".to_string());
        let with_confidence = value.with_confidence(&Value::Float(0.8)).unwrap();
        assert_eq!(with_confidence.get_confidence(), Some(0.8));
    }
} 