use std::error::Error;

pub struct LLMClient {
    api_key: String,
}

impl LLMClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn semantic_similarity(&self, text1: &str, text2: &str) -> Result<f64, Box<dyn Error + Send + Sync>> {
        // For now, return a simple similarity score based on length difference
        let len_diff = (text1.len() as f64 - text2.len() as f64).abs();
        let max_len = text1.len().max(text2.len()) as f64;
        Ok(1.0 - len_diff / max_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_similarity() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = LLMClient::new("test_key".to_string());
        let similarity = client.semantic_similarity("hello", "hello world").await?;
        assert!(similarity > 0.0 && similarity < 1.0);
        Ok(())
    }
}
