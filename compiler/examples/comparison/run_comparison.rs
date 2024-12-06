use prism::{Interpreter, Value};
use std::error::Error;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let interpreter = Interpreter::new(api_key);

    let source = r#"
        // Medical diagnosis example
        let symptoms = ["fever", "cough", "fatigue"];
        let severity = "moderate";
        let duration = "5 days";

        // Perform diagnosis
        let diagnosis = match symptoms {
            ["fever", "cough", _] if severity == "moderate" => "Common cold",
            ["fever", "cough", "fatigue"] if duration > "7 days" => "Flu",
            ["fever", _, _] if severity == "severe" => "Seek immediate medical attention",
            _ => "Unknown condition"
        };

        // Return diagnosis
        diagnosis
    "#;

    // Measure Prism execution time
    let start = Instant::now();
    let prism_results = interpreter.eval(source.to_string()).await?;
    let prism_duration = start.elapsed();

    println!("Prism Results:");
    println!("Diagnosis: {}", prism_results.as_string().unwrap_or_default());
    println!("Time taken: {:?}", prism_duration);

    Ok(())
}
