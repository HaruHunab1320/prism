use prism::Interpreter;
use std::error::Error;
use std::time::Instant;

mod traditional_diagnosis;

fn main() {
    println!("Run with: cargo run --example comparison");
}

pub async fn run_comparison() -> Result<(), Box<dyn Error>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());

    println!("Running Medical Diagnosis Comparison");
    println!("==================================\n");

    // Run Prism version
    println!("Prism Implementation:");
    println!("--------------------");
    let prism_start = Instant::now();

    let source = include_str!("../medical_diagnosis.prism");
    let mut interpreter = Interpreter::new(api_key.clone());
    let prism_results = interpreter.eval(source.to_string()).await?;
    let prism_time = prism_start.elapsed();

    println!("Results: {:#?}", prism_results);
    println!("Processing Time: {:?}", prism_time);
    println!("\nCode Complexity:");
    println!("- Lines of Code: ~50");
    println!("- Built-in confidence handling");
    println!("- Automatic error propagation");
    println!("- Declarative syntax");
    println!("- Type inference");

    // Run traditional version
    println!("\nTraditional Implementation:");
    println!("--------------------------");
    let trad_start = Instant::now();

    let traditional_results = traditional_diagnosis::diagnose(api_key).await?;
    let trad_time = trad_start.elapsed();

    println!("Results: {:#?}", traditional_results);
    println!("Processing Time: {:?}", trad_time);
    println!("\nCode Complexity:");
    println!("- Lines of Code: ~200");
    println!("- Manual confidence handling");
    println!("- Manual error handling");
    println!("- Imperative syntax");
    println!("- Manual type annotations");

    // Compare results
    println!("\nComparison:");
    println!("-----------");
    println!(
        "Time Difference: {:?}",
        trad_time.as_secs_f64() - prism_time.as_secs_f64()
    );
    println!("Code Size Ratio: ~4:1 (Traditional:Prism)");
    println!("Maintainability: Prism code is more declarative and focused on the domain logic");
    println!("Error Handling: Prism provides automatic error propagation and confidence tracking");
    println!("Type Safety: Both provide strong type safety, but Prism requires less annotation");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_comparison() -> Result<(), Box<dyn Error>> {
        run_comparison().await
    }
}
