use std::error::Error;
use std::fmt;

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
        // For now, return a simple string matching score
        let score = if text.contains(pattern) { 1.0 } else { 0.0 };
        Ok(score)
    }
}

pub async fn semantic_similarity(text: &str, pattern: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
    // Initialize the LLM client with the OpenAI API
    let client = reqwest::Client::new();
    
    // Construct the prompt for semantic similarity comparison
    let prompt = format!(
        "Compare the semantic similarity between these two texts on a scale of 0.0 to 1.0:\nText 1: {}\nText 2: {}\nProvide only the numerical score.",
        text, pattern
    );
    
    // Make API request to OpenAI
    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", std::env::var("OPENAI_API_KEY")?))
        .json(&serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a semantic similarity scorer. Respond only with a number between 0.0 and 1.0 representing the semantic similarity between two texts."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3
        }))
        .send()
        .await?;
    
    // Parse the response
    let response_data: serde_json::Value = response.json().await?;
    let score_text = response_data["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("Invalid response format")?
        .trim();
    
    // Convert the score text to a float
    let score: f64 = score_text.parse()?;
    
    // Ensure the score is between 0 and 1
    Ok(score.max(0.0).min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_semantic_similarity() {
        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "A fast auburn canine leaps above a sleepy hound";
        
        let similarity = semantic_similarity(text1, text2).await.unwrap();
        assert!(similarity >= 0.0 && similarity <= 1.0);
        assert!(similarity > 0.7); // These sentences are semantically very similar
    }
}
