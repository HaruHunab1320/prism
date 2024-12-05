use std::error::Error;
use std::fmt;
use crate::interpreter::Interpreter;
use crate::ast::Value;

#[derive(Debug)]
pub struct LLMError(String);

impl fmt::Display for LLMError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LLM error: {}", self.0)
    }
}

impl Error for LLMError {}

pub struct LLMClient {
    api_key: String,
}

impl LLMClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn semantic_match(
        &self,
        text: &str,
        pattern: &str,
    ) -> Result<f64, Box<dyn Error + Send + Sync>> {
        // TODO: Implement actual LLM semantic matching
        // For now, return a simple string matching score
        let score = if text.contains(pattern) { 1.0 } else { 0.0 };
        Ok(score)
    }
}

pub fn register_llm_functions(interpreter: &mut Interpreter) {
    interpreter.register_native_function("semantic_match", |interpreter, args| {
        Box::pin(async move {
            if args.len() != 2 {
                return Err("semantic_match() takes exactly two arguments".into());
            }
            
            let text = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("First argument must be a string".into()),
            };
            
            let pattern = match &args[1] {
                Value::String(s) => s.clone(),
                _ => return Err("Second argument must be a string".into()),
            };

            let client = LLMClient::new(interpreter.get_api_key().to_string());
            let score = client.semantic_match(&text, &pattern).await?;
            Ok(Value::Number(score))
        })
    });
}
