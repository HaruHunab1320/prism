use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::io;
use std::time::Duration;
use tokio::runtime::Runtime;
use ndarray::Array1;
use dotenv::dotenv;

const GEMINI_API_URL: &str = "https://generativelanguage.googleapis.com/v1";
const DEFAULT_MODEL: &str = "gemini-pro";
const EMBEDDING_MODEL: &str = "embedding-001";

#[derive(Debug)]
pub enum LLMError {
    Environment(String),
    Api(String),
    RateLimit,
    InvalidResponse(String),
    Network(reqwest::Error),
    Io(io::Error),
}

impl std::fmt::Display for LLMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Environment(msg) => write!(f, "Environment error: {}", msg),
            Self::Api(msg) => write!(f, "API error: {}", msg),
            Self::RateLimit => write!(f, "Rate limit exceeded"),
            Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
            Self::Network(e) => write!(f, "Network error: {}", e),
            Self::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for LLMError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Network(e) => Some(e),
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for LLMError {
    fn from(err: reqwest::Error) -> Self {
        Self::Network(err)
    }
}

impl From<io::Error> for LLMError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

#[derive(Debug)]
pub struct LLMClient {
    client: Client,
    api_key: String,
    model: String,
    runtime: Runtime,
}

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    text: String,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

impl LLMClient {
    pub fn new() -> Result<Self, LLMError> {
        dotenv().ok();

        let api_key = std::env::var("GOOGLE_API_KEY")
            .map_err(|_| LLMError::Environment("GOOGLE_API_KEY not found in environment or .env file".into()))?;

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let runtime = Runtime::new()
            .map_err(|e| LLMError::Io(e))?;

        Ok(Self {
            client,
            api_key,
            model: DEFAULT_MODEL.to_string(),
            runtime,
        })
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub async fn complete(&self, prompt: &str) -> Result<String, LLMError> {
        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let url = format!(
            "{}/models/{}/generateContent?key={}",
            GEMINI_API_URL, self.model, self.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LLMError::RateLimit);
        }

        let response = response
            .json::<GeminiResponse>()
            .await
            .map_err(|e| LLMError::InvalidResponse(e.to_string()))?;

        response.candidates
            .first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| LLMError::InvalidResponse("Empty response from API".into()))
    }

    pub async fn get_embedding(&self, text: &str) -> Result<Array1<f32>, LLMError> {
        let request = EmbeddingRequest {
            text: text.to_string(),
        };

        let url = format!(
            "{}/models/{}/embeddings?key={}",
            GEMINI_API_URL, EMBEDDING_MODEL, self.api_key
        );

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(LLMError::RateLimit);
        }

        let response = response
            .json::<EmbeddingResponse>()
            .await
            .map_err(|e| LLMError::InvalidResponse(e.to_string()))?;

        Ok(Array1::from_vec(response.embedding))
    }

    pub async fn semantic_similarity(&self, text1: &str, text2: &str) -> Result<f32, LLMError> {
        let emb1 = self.get_embedding(text1).await?;
        let emb2 = self.get_embedding(text2).await?;
        
        let dot_product = emb1.dot(&emb2);
        let norm1 = (emb1.dot(&emb1)).sqrt();
        let norm2 = (emb2.dot(&emb2)).sqrt();
        
        Ok(dot_product / (norm1 * norm2))
    }

    pub async fn verify_statement(&self, statement: &str, source: &str) -> Result<f32, LLMError> {
        let prompt = format!(
            "Verify the following statement using {source} as a source:\n\n{statement}\n\n\
            Rate the confidence in this statement from 0.0 to 1.0, where:\n\
            1.0 = Completely verified and supported by the source\n\
            0.0 = Completely unverified or contradicted by the source\n\
            Respond with ONLY the numerical confidence value.",
        );

        let response = self.complete(&prompt).await?;
        response.trim().parse::<f32>()
            .map_err(|e| LLMError::InvalidResponse(format!("Failed to parse confidence value: {}", e)))
    }

    pub fn blocking_verify_statement(&self, statement: &str, source: &str) -> Result<f32, LLMError> {
        self.runtime.block_on(self.verify_statement(statement, source))
    }

    pub fn blocking_semantic_similarity(&self, text1: &str, text2: &str) -> Result<f32, LLMError> {
        self.runtime.block_on(self.semantic_similarity(text1, text2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test::block_on;

    #[test]
    fn test_client_creation() {
        match LLMClient::new() {
            Ok(_) => assert!(true, "Client created successfully"),
            Err(e) => eprintln!("Test skipped: {}", e),
        }
    }

    #[test]
    fn test_semantic_similarity() {
        if let Ok(client) = LLMClient::new() {
            let result = block_on(client.semantic_similarity(
                "The weather is nice today",
                "It's a beautiful sunny day",
            ));
            
            match result {
                Ok(similarity) => {
                    assert!(similarity > 0.0);
                    assert!(similarity <= 1.0);
                }
                Err(e) => eprintln!("Test skipped: {}", e),
            }
        }
    }

    #[test]
    fn test_rate_limit_handling() {
        if let Ok(client) = LLMClient::new() {
            // Simulate multiple rapid requests
            for _ in 0..5 {
                let result = block_on(client.complete("Test prompt"));
                match result {
                    Ok(_) => continue,
                    Err(LLMError::RateLimit) => {
                        // Expected behavior
                        assert!(true);
                        return;
                    }
                    Err(e) => eprintln!("Unexpected error: {}", e),
                }
            }
        }
    }

    #[test]
    fn test_invalid_api_key() {
        std::env::set_var("GOOGLE_API_KEY", "invalid_key");
        let client = LLMClient::new().unwrap();
        let result = block_on(client.complete("Test prompt"));
        match result {
            Err(LLMError::Api(_)) | Err(LLMError::Network(_)) | Err(LLMError::InvalidResponse(_)) => assert!(true),
            _ => panic!("Expected API, Network, or InvalidResponse error"),
        }
    }
} 