use prism::Interpreter;
use prism::llm::{LLMClient, Provider, CompletionRequest};
use std::error::Error;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize both interpreters
    let mut interpreter = Interpreter::new();
    let mut llm_client = LLMClient::new(Provider::Google("YOUR_API_KEY".to_string()));

    let source = r#"
        let text1 = "The weather is nice";
        let text2 = "It's a beautiful day";
        let match_score = semantic_match(text1, text2);
        match_score
    "#;

    // Run comparison
    let prism_results = interpreter.evaluate(source.to_string()).await?;
    let llm_results = llm_client.complete(CompletionRequest {
        prompt: "Compare these two sentences: 'The weather is nice' and 'It's a beautiful day'".to_string(),
        context: None,
        config: None,
    }).await?;

    println!("Prism Results: {:?}", prism_results);
    println!("LLM Results: {:?}", llm_results);

    Ok(())
}
