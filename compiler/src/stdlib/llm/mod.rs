use crate::ast::Value;
use crate::interpreter::Interpreter;
use reqwest::Client;
use serde_json::json;
use std::error::Error;
use std::sync::Arc;

pub struct LLMClient {
    api_key: String,
    client: Client,
}

impl LLMClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    pub async fn semantic_match(&self, text: &str, pattern: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
        // Get embeddings for both text and pattern
        let text_embedding = self.get_embedding(text).await?;
        let pattern_embedding = self.get_embedding(pattern).await?;

        // Calculate cosine similarity
        let similarity = cosine_similarity(&text_embedding, &pattern_embedding);
        Ok(similarity)
    }

    async fn get_embedding(&self, text: &str) -> Result<Vec<f32>, Box<dyn Error + Send + Sync>> {
        let url = "https://api.openai.com/v1/embeddings";
        
        let response = self.client
            .post(url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "input": text,
                "model": "text-embedding-ada-002"
            }))
            .send()
            .await?;

        let json = response.json::<serde_json::Value>().await?;
        
        // Extract embedding from response
        let embedding = json["data"][0]["embedding"]
            .as_array()
            .ok_or("Invalid response format")?
            .iter()
            .map(|v| v.as_f64().ok_or("Invalid number in embedding"))
            .collect::<Result<Vec<f64>, _>>()?
            .into_iter()
            .map(|v| v as f32)
            .collect();

        Ok(embedding)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f64 {
    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    (dot_product / (norm_a * norm_b)) as f64
}

pub fn register_llm_functions(interpreter: &Interpreter) {
    interpreter.register_native_function("semantic_match", |interpreter: &Interpreter, args: Vec<Value>| {
        let api_key = interpreter.get_api_key().to_string();
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

            let client = LLMClient::new(api_key);
            let confidence = client.semantic_match(&text, &pattern).await?;
            Ok(Value::Number(confidence))
        })
    });
}
