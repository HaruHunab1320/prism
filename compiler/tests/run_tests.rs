use colored::*;
use prism::Interpreter;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

mod integration_tests;

type TestFuture = Pin<Box<dyn Future<Output = Result<(), Box<dyn Error + Send + Sync>>> + Send>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("\n{}", "Running Prism Language Tests".bold().green());
    println!("{}", "=========================".green());

    let start_time = Instant::now();
    let mut passed = 0;
    let mut failed = 0;

    // Run individual feature tests
    let tests: Vec<(&str, TestFuture)> = vec![
        (
            "Confidence Flow",
            Box::pin(integration_tests::test_confidence_flow()),
        ),
        (
            "Context Operations",
            Box::pin(integration_tests::test_context_operations()),
        ),
        (
            "Pattern Matching",
            Box::pin(integration_tests::test_pattern_matching()),
        ),
        (
            "Tensor Operations",
            Box::pin(integration_tests::test_tensor_operations()),
        ),
        (
            "Semantic Matching",
            Box::pin(integration_tests::test_semantic_matching()),
        ),
        (
            "Verification System",
            Box::pin(integration_tests::test_verification()),
        ),
        (
            "Uncertain Conditionals",
            Box::pin(integration_tests::test_uncertain_conditionals()),
        ),
        (
            "Try-Confidence Blocks",
            Box::pin(integration_tests::test_try_confidence()),
        ),
        (
            "Async Operations",
            Box::pin(integration_tests::test_async_operations()),
        ),
        (
            "All Features Combined",
            Box::pin(integration_tests::test_all_features()),
        ),
    ];

    let total_tests = tests.len();

    for (name, test) in tests {
        print!("Testing {:<30}", name);
        match test.await {
            Ok(_) => {
                println!("{}", "PASSED".green());
                passed += 1;
            }
            Err(e) => {
                println!("{}", "FAILED".red());
                println!("  Error: {}", e.to_string().red());
                failed += 1;
            }
        }
    }

    // Print summary
    let duration = start_time.elapsed();
    println!("\n{}", "Test Summary".bold());
    println!("------------");
    println!("Total Tests: {}", total_tests);
    println!("Passed:      {}", passed.to_string().green());
    println!("Failed:      {}", failed.to_string().red());
    println!("Duration:    {:.2?}", duration);

    if failed > 0 {
        Err(format!("{} tests failed", failed).into())
    } else {
        println!("\n{}", "All tests passed successfully!".bold().green());
        Ok(())
    }
}

// Helper function to run a single test file
pub async fn run_test_file(source: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);
    interpreter.eval(source.to_string()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runner() -> Result<(), Box<dyn Error + Send + Sync>> {
        integration_tests::run_all_tests().await
    }
}
