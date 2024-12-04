use std::error::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Clone)]
pub struct LLMClient {
    client: Arc<Client>,
    api_key: String,
}

impl LLMClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Arc::new(Client::new()),
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

    pub async fn validate_symptom(&self, symptom: &str) -> Result<f64, Box<dyn Error>> {
        let prompt = format!(
            "Validate if '{}' is a clear and valid medical symptom. \
            Return a confidence score between 0 and 1, where:\n\
            1.0 = Clear, specific medical symptom\n\
            0.7-0.9 = Valid but could be more specific\n\
            0.4-0.6 = Ambiguous or general\n\
            0.0-0.3 = Not a valid medical symptom\n\
            Return only the number.",
            symptom
        );

        let response = self.generate(&prompt).await?;
        response.parse::<f64>().map_err(|e| e.into())
    }

    pub async fn semantic_match(&self, symptoms: &str, disease_pattern: &str) -> Result<f64, Box<dyn Error>> {
        let prompt = format!(
            "Compare these symptoms: '{}'\n\
            with this disease pattern: '{}'\n\
            Return a confidence score between 0 and 1 indicating how well they match.\n\
            Consider:\n\
            - Symptom overlap\n\
            - Symptom specificity\n\
            - Pattern completeness\n\
            Return only the number.",
            symptoms, disease_pattern
        );

        let response = self.generate(&prompt).await?;
        response.parse::<f64>().map_err(|e| e.into())
    }

    pub async fn get_disease_pattern(&self, condition: &str) -> Result<String, Box<dyn Error>> {
        let prompt = format!(
            "Provide a concise, comma-separated list of the most common symptoms for {}.\n\
            Focus on specific, observable symptoms.\n\
            Return only the symptoms, no additional text.",
            condition
        );

        self.generate(&prompt).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tokio::runtime::Runtime;

    #[test]
    fn test_llm_client() {
        let api_key = env::var("GEMINI_API_KEY").unwrap_or_else(|_| "test_key".to_string());
        let client = LLMClient::new(api_key);
        let rt = Runtime::new().unwrap();

        let result = rt.block_on(async {
            client.generate("Hello, world!").await
        });

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("test_key"));
    }
} 