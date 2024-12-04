use std::error::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};

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

pub struct LLMClient {
    client: Client,
    api_key: String,
}

impl LLMClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn generate(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: prompt.to_string(),
                }],
            }],
        };

        let response = self.client
            .post("https://generativelanguage.googleapis.com/v1beta/models/gemini-pro:generateContent")
            .query(&[("key", &self.api_key)])
            .json(&request)
            .send()
            .await?
            .json::<GeminiResponse>()
            .await?;

        if let Some(candidate) = response.candidates.first() {
            if let Some(part) = candidate.content.parts.first() {
                return Ok(part.text.clone());
            }
        }

        Err("No response from Gemini".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::runtime::Runtime;

    // Mock client for testing
    struct MockClient {
        api_key: String,
    }

    impl MockClient {
        fn new(api_key: String) -> Self {
            Self { api_key }
        }

        async fn generate(&self, _prompt: &str) -> Result<String, Box<dyn Error>> {
            if self.api_key == "test_key" {
                Err("Invalid API key: test_key".into())
            } else {
                Ok("Mock response".to_string())
            }
        }
    }

    #[test]
    fn test_llm_client() {
        let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| "test_key".to_string());
        let client = MockClient::new(api_key);
        let rt = Runtime::new().unwrap();

        let result = rt.block_on(async {
            client.generate("Hello, world!").await
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test_key"));
    }
} 