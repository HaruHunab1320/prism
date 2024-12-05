use std::error::Error;
use std::time::Instant;
use colored::*;
use prism::{Lexer, Parser, Interpreter};

mod integration_tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("\n{}", "Running Prism Language Tests".bold().green());
    println!("{}", "=========================".green());

    let start_time = Instant::now();
    let mut passed = 0;
    let mut failed = 0;

    // Run individual feature tests
    let tests = vec![
        ("Confidence Flow", integration_tests::test_confidence_flow()),
        ("Context Operations", integration_tests::test_context_operations()),
        ("Pattern Matching", integration_tests::test_pattern_matching()),
        ("Tensor Operations", integration_tests::test_tensor_operations()),
        ("Semantic Matching", integration_tests::test_semantic_matching()),
        ("Verification System", integration_tests::test_verification()),
        ("Uncertain Conditionals", integration_tests::test_uncertain_conditionals()),
        ("Try-Confidence Blocks", integration_tests::test_try_confidence()),
        ("Async Operations", integration_tests::test_async_operations()),
        ("All Features Combined", integration_tests::test_all_features()),
    ];

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
    println!("Total Tests: {}", tests.len());
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
pub async fn run_test_file(source: &str) -> Result<(), Box<dyn Error>> {
    let mut lexer = Lexer::new(source);
    let (tokens, starts, ends) = lexer.lex()?;
    let mut parser = Parser::new(source.to_string(), tokens, starts, ends);
    let statements = parser.parse()?;

    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    for stmt in statements {
        interpreter.eval_stmt(stmt).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runner() -> Result<(), Box<dyn Error>> {
        integration_tests::run_all_tests().await
    }
} 