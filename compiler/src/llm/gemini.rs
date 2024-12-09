use serde::{Deserialize, Serialize};
use crate::error::Result;
use super::{CompletionRequest, CompletionResponse, TokenUsage};

#[derive(Debug, Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Debug, Serialize)]
struct GenerationConfig {
    temperature: f64,
    max_output_tokens: usize,
    top_p: f64,
}

#[derive(Debug, Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
    prompt_feedback: PromptFeedback,
}

#[derive(Debug, Deserialize)]
struct Candidate {
    content: Content,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct PromptFeedback {
    token_count: TokenCount,
}

#[derive(Debug, Deserialize)]
struct TokenCount {
    total_tokens: usize,
    prompt_tokens: usize,
}

pub(crate) async fn complete(
    client: &reqwest::Client,
    api_key: &str,
    request: CompletionRequest,
    model_name: &str,
    temperature: f64,
    max_tokens: usize,
    base_url: Option<String>,
) -> Result<CompletionResponse> {
    let contents = vec![
        Content {
            role: "user".to_string(),
            parts: vec![Part {
                text: format!(
                    "Context: {}\n\nPrompt: {}",
                    request.context.as_ref().map_or("None".to_string(), |ctx| ctx.to_string()),
                    request.prompt.clone()
                ),
            }],
        },
    ];

    let gemini_request = GeminiRequest {
        contents,
        generation_config: GenerationConfig {
            temperature,
            max_output_tokens: max_tokens,
            top_p: 1.0,
        },
    };

    let base_url = base_url.unwrap_or_else(|| 
        "https://generativelanguage.googleapis.com".to_string()
    );

    let response = client
        .post(format!(
            "{}/v1/models/{model_name}:generateContent?key={api_key}",
            base_url
        ))
        .json(&gemini_request)
        .send()
        .await?
        .error_for_status()?
        .json::<GeminiResponse>()
        .await?;

    let candidate = response.candidates.first().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "No completion candidates returned")
    })?;

    // Calculate confidence based on finish reason
    let confidence = match candidate.finish_reason.as_str() {
        "STOP" => 0.95, // Natural completion
        "MAX_TOKENS" => 0.7, // Cut off by max tokens
        _ => 0.5, // Other reasons
    };

    let completion_tokens = response.prompt_feedback.token_count.total_tokens
        - response.prompt_feedback.token_count.prompt_tokens;

    Ok(CompletionResponse {
        text: candidate.content.parts.first()
            .map(|part| part.text.clone())
            .unwrap_or_default(),
        confidence,
        model: model_name.to_string(),
        usage: TokenUsage {
            prompt_tokens: response.prompt_feedback.token_count.prompt_tokens,
            completion_tokens,
            total_tokens: response.prompt_feedback.token_count.total_tokens,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::ModelConfig;

    #[tokio::test]
    async fn test_gemini_completion() -> Result<()> {
        // Skip test if no API key is provided
        let api_key = std::env::var("GOOGLE_API_KEY").ok();
        if api_key.is_none() {
            return Ok(());
        }

        let client = reqwest::Client::new();
        let request = CompletionRequest {
            prompt: "What is 2+2?".to_string(),
            context: None,
            config: Some(ModelConfig::default()),
        };

        let response = complete(
            &client,
            &api_key.unwrap(),
            request,
            "gemini-pro",
            0.7,
            100,
            None,
        )
        .await?;

        assert!(!response.text.is_empty());
        assert!(response.confidence > 0.0 && response.confidence <= 1.0);
        assert!(response.usage.total_tokens > 0);

        Ok(())
    }
} 