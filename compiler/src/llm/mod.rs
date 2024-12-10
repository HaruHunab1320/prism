use std::time::Duration;
use crate::error::{Result, PrismError};

pub enum LLMProvider {
    OpenAI(String),
    Google(String),
}

#[derive(Clone)]
pub struct ModelConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: usize,
    pub timeout: Duration,
    pub max_retries: usize,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: 1000,
            timeout: Duration::from_secs(30),
            max_retries: 3,
        }
    }
}

pub struct CompletionRequest {
    pub prompt: String,
    pub context: Option<String>,
    pub config: Option<ModelConfig>,
}

pub struct CompletionResponse {
    pub text: String,
    pub confidence: f32,
    pub model: String,
}

pub struct LLMClient {
    provider: LLMProvider,
    config: ModelConfig,
}

impl LLMClient {
    pub fn new(provider: LLMProvider) -> Self {
        Self {
            provider,
            config: ModelConfig::default(),
        }
    }

    pub fn with_config(provider: LLMProvider, config: ModelConfig) -> Self {
        Self { provider, config }
    }

    pub fn get_provider(&self) -> &LLMProvider {
        &self.provider
    }

    pub fn get_config(&self) -> &ModelConfig {
        &self.config
    }

    pub async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse> {
        // For now, just return an error since we haven't implemented the actual API calls
        Err(PrismError::RuntimeError("LLM API not implemented yet".to_string()))
    }
} 