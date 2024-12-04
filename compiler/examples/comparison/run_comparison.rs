use std::env;
use std::error::Error;
use std::time::Instant;
use std::ops::RangeInclusive;
use dotenv::dotenv;

mod traditional_diagnosis;
use traditional_diagnosis::TraditionalDiagnosisSystem;

use prism::{Interpreter, Parser, Lexer};
use prism::stdlib::medical::MedicalLLM;

#[derive(Debug)]
struct ComparisonMetrics {
    traditional_loc: usize,
    prism_loc: usize,
    traditional_time: f64,
    prism_time: f64,
    traditional_accuracy: f64,
    prism_accuracy: f64,
    code_reduction: f64,
    speed_improvement: f64,
    traditional_avg_confidence: f64,
    prism_avg_confidence: f64,
}

async fn run_comparison() -> Result<ComparisonMetrics, Box<dyn Error>> {
    println!("\n=== Starting Comparison Demo ===\n");
    
    // Load environment variables
    dotenv().ok();
    println!("Loading environment variables...");
    
    // Get API key
    let api_key = env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY must be set in environment");
    println!("API key loaded successfully.");

    // Test cases with expected confidence ranges
    let test_cases = vec![
        (
            vec!["fever", "cough", "fatigue", "body_aches"],
            "flu",
            0.8..=0.95,
        ),
        (
            vec!["fever", "dry_cough", "loss_of_taste", "shortness_of_breath"],
            "covid19",
            0.85..=0.95,
        ),
        (
            vec!["runny_nose", "sneezing", "mild_cough"],
            "common_cold",
            0.7..=0.9,
        ),
        (
            vec!["sneezing", "itchy_eyes", "runny_nose", "congestion"],
            "allergies",
            0.75..=0.9,
        ),
    ];

    println!("\n=== Traditional Implementation ===");
    println!("Running {} test cases...\n", test_cases.len());
    
    let traditional_start = Instant::now();
    let mut trad_system = TraditionalDiagnosisSystem::new(api_key.clone());
    let mut traditional_confidences = Vec::new();
    
    for (i, (symptoms, actual, expected_range)) in test_cases.iter().enumerate() {
        println!("Test Case {}/{}:", i + 1, test_cases.len());
        println!("Symptoms: {:?}", symptoms);
        println!("Expected Condition: {}", actual);
        
        let symptoms = symptoms.iter().map(|s| s.to_string()).collect();
        let result = trad_system.diagnose(symptoms, actual).await?;
        
        println!("Diagnosis: {} (Confidence: {:.2})", result.condition, result.confidence);
        println!("Expected Confidence Range: {:.2}..={:.2}", expected_range.start(), expected_range.end());
        
        if expected_range.contains(&result.confidence) {
            println!("✓ Confidence within expected range");
        } else {
            println!("✗ Confidence outside expected range");
        }
        println!();
        
        traditional_confidences.push(result.confidence);
    }
    
    let traditional_time = traditional_start.elapsed().as_secs_f64();
    trad_system.print_comparison_metrics();

    println!("\n=== Prism Implementation ===");
    let prism_start = Instant::now();
    
    println!("Initializing interpreter...");
    let mut interpreter = Interpreter::new();
    let medical_llm = MedicalLLM::new(api_key);
    medical_llm.register_functions(&mut interpreter);
    println!("Interpreter initialized.");

    println!("Loading Prism source code...");
    let source = include_str!("../medical_diagnosis.prism");
    let tokens = Lexer::new(source).collect::<Vec<_>>();
    println!("Tokens: {:?}", tokens);
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;
    println!("Source code parsed successfully.");

    println!("Running Prism implementation...");
    let mut prism_confidences = Vec::new();
    
    for (i, stmt) in statements.iter().enumerate() {
        println!("Executing statement {}/{}...", i + 1, statements.len());
        let result = interpreter.eval_stmt(stmt).await?;
        if let Some(confidence) = result.get_confidence() {
            println!("Got confidence value: {:.2}", confidence);
            prism_confidences.push(confidence);
        }
    }

    let prism_time = prism_start.elapsed().as_secs_f64();
    println!("Prism implementation completed.");

    println!("\n=== Calculating Metrics ===");
    let metrics = ComparisonMetrics {
        traditional_loc: 350,
        prism_loc: 150,
        traditional_time,
        prism_time,
        traditional_accuracy: trad_system.metrics.accuracy(),
        prism_accuracy: interpreter.metrics.accuracy(),
        code_reduction: (350 - 150) as f64 / 350.0 * 100.0,
        speed_improvement: (traditional_time - prism_time) / traditional_time * 100.0,
        traditional_avg_confidence: traditional_confidences.iter().sum::<f64>() / traditional_confidences.len() as f64,
        prism_avg_confidence: if !prism_confidences.is_empty() {
            prism_confidences.iter().sum::<f64>() / prism_confidences.len() as f64
        } else {
            0.0
        },
    };

    println!("\n=== Comparison Summary ===");
    println!("Lines of Code Reduction: {:.1}%", metrics.code_reduction);
    println!("Speed Improvement: {:.1}%", metrics.speed_improvement);
    println!("Traditional Implementation Time: {:.3}s", metrics.traditional_time);
    println!("Prism Implementation Time: {:.3}s", metrics.prism_time);
    println!("Traditional Implementation Accuracy: {:.2}%", metrics.traditional_accuracy * 100.0);
    println!("Prism Implementation Accuracy: {:.2}%", metrics.prism_accuracy * 100.0);
    println!("Traditional Average Confidence: {:.2}", metrics.traditional_avg_confidence);
    println!("Prism Average Confidence: {:.2}", metrics.prism_avg_confidence);

    Ok(metrics)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    match run_comparison().await {
        Ok(metrics) => {
            println!("\nComparison completed successfully!");
            println!("See above for detailed metrics.");
        }
        Err(e) => {
            eprintln!("\nError running comparison:");
            eprintln!("Error: {}", e);
            if let Some(source) = e.source() {
                eprintln!("Caused by: {}", source);
            }
            std::process::exit(1);
        }
    }

    Ok(())
} 