use prism::llm::{LLMClient, Provider, CompletionRequest, ModelConfig};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key from environment
    let api_key = env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY environment variable must be set");

    // Create a client with default configuration
    let client = LLMClient::new(Provider::Google(api_key));

    // Or create a client with custom configuration
    let config = ModelConfig {
        model: "gemini-pro".to_string(),
        temperature: 0.7,
        max_tokens: 1024,
        timeout_secs: 30,
        max_retries: 3,
    };
    let client_with_config = LLMClient::with_config(Provider::Google(api_key), config);

    // Create a completion request
    let request = CompletionRequest {
        prompt: "What are three interesting facts about quantum computing?".to_string(),
        context: None,
        config: None,
    };

    // Get a completion
    let response = client.complete(request).await?;
    println!("Response: {}", response.text);
    println!("Confidence: {}", response.confidence);
    println!("Model: {}", response.model);
    println!("Token usage: {} total ({} prompt, {} completion)",
        response.usage.total_tokens,
        response.usage.prompt_tokens,
        response.usage.completion_tokens
    );

    Ok(())
} 