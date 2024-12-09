use serde::{Deserialize, Serialize};
use crate::error::Result;
use super::{CompletionRequest, CompletionResponse, TokenUsage};

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f64,
    max_tokens: usize,
    top_p: f64,
    frequency_penalty: f64,
    presence_penalty: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
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
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: format!(
                "You are an AI assistant with the following context: {}",
                request.context.as_ref().map_or("None".to_string(), |ctx| ctx.to_string())
            ),
        },
        Message {
            role: "user".to_string(),
            content: request.prompt.clone(),
        },
    ];

    let openai_request = OpenAIRequest {
        model: model_name.to_string(),
        messages,
        temperature,
        max_tokens,
        top_p: 1.0,
        frequency_penalty: 0.0,
        presence_penalty: 0.0,
    };

    let base_url = base_url.unwrap_or_else(|| 
        "https://api.openai.com".to_string()
    );

    let response = client
        .post(format!("{}/v1/chat/completions", base_url))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&openai_request)
        .send()
        .await?
        .error_for_status()?
        .json::<OpenAIResponse>()
        .await?;

    let choice = response.choices.first().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::Other, "No completion choices returned")
    })?;

    // Calculate confidence based on finish reason
    let confidence = match choice.finish_reason.as_str() {
        "stop" => 0.95, // Natural completion
        "length" => 0.7, // Cut off by max tokens
        _ => 0.5, // Other reasons (content filter, etc.)
    };

    Ok(CompletionResponse {
        text: choice.message.content.clone(),
        confidence,
        model: model_name.to_string(),
        usage: TokenUsage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::ModelConfig;

    #[tokio::test]
    async fn test_openai_completion() -> Result<()> {
        // Skip test if no API key is provided
        let api_key = std::env::var("OPENAI_API_KEY").ok();
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
            "gpt-3.5-turbo",
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