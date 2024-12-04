use std::error::Error;
use std::time::Instant;
use dotenv::dotenv;
use std::env;

mod traditional_diagnosis;
use traditional_diagnosis::TraditionalDiagnosisSystem;

use prism::{Interpreter, Parser, Lexer};
use prism::stdlib::medical::MedicalLLM;

#[derive(Debug)]
struct ComparisonMetrics {
    traditional_time: f64,
    prism_time: f64,
    traditional_confidence: f64,
    prism_confidence: f64,
    speed_improvement: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let api_key = env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY must be set");
    
    println!("\n=== Traditional Implementation ===");
    let traditional_start = Instant::now();
    let traditional_system = TraditionalDiagnosisSystem::new(api_key.clone());

    // Test symptom validation
    let symptoms = vec![
        "fever",
        "cough",
        "fatigue",
        "shortness of breath",
    ];

    println!("\nTesting symptom validation...");
    let mut traditional_symptom_confidence = 0.0;
    for symptom in &symptoms {
        match traditional_system.validate_symptom(symptom).await {
            Ok(validation) => {
                println!(
                    "Symptom: {}, Confidence: {:.2}, Source: {}",
                    validation.symptom, validation.confidence, validation.validation_source
                );
                traditional_symptom_confidence += validation.confidence;
            }
            Err(e) => println!("Error validating symptom {}: {}", symptom, e),
        }
    }
    traditional_symptom_confidence /= symptoms.len() as f64;

    // Test disease pattern matching
    println!("\nTesting disease pattern matching...");
    let test_symptoms = symptoms.join(", ");
    let test_pattern = "fever, dry cough, fatigue, difficulty breathing";
    let traditional_match_confidence = match traditional_system.semantic_match(&test_symptoms, test_pattern).await {
        Ok(confidence) => {
            println!("Match confidence: {:.2}", confidence);
            confidence
        }
        Err(e) => {
            println!("Error matching disease pattern: {}", e);
            0.0
        }
    };

    let traditional_time = traditional_start.elapsed().as_secs_f64();
    println!("\nTraditional implementation took: {:.2}s", traditional_time);

    println!("\n=== Prism Implementation ===");
    let prism_start = Instant::now();

    // Initialize Prism interpreter
    let mut interpreter = Interpreter::new();
    let medical_llm = MedicalLLM::new(api_key);
    medical_llm.register_functions(&mut interpreter);

    // Load and parse Prism code
    println!("Loading Prism source code...");
    let source = include_str!("../medical_diagnosis.prism");
    let tokens = Lexer::new(source).collect::<Vec<_>>();
    let mut parser = Parser::new(tokens);
    let statements = parser.parse()?;

    // Execute Prism code
    println!("Executing Prism code...");
    let mut prism_confidence = 0.0;
    for stmt in statements {
        let result = interpreter.eval_stmt(&stmt).await?;
        let text = format!("{}", result);
        println!("{}", text);
        if text.contains("Confidence:") {
            if let Some(conf_str) = text.split("Confidence: ").nth(1) {
                if let Some(conf) = conf_str.trim_end_matches(')').parse::<f64>().ok() {
                    prism_confidence = conf;
                }
            }
        }
    }

    let prism_time = prism_start.elapsed().as_secs_f64();
    println!("\nPrism implementation took: {:.2}s", prism_time);

    // Calculate and display metrics
    let metrics = ComparisonMetrics {
        traditional_time,
        prism_time,
        traditional_confidence: (traditional_symptom_confidence + traditional_match_confidence) / 2.0,
        prism_confidence,
        speed_improvement: ((traditional_time - prism_time) / traditional_time) * 100.0,
    };

    println!("\n=== Comparison Results ===");
    println!("Traditional Implementation:");
    println!("  - Time: {:.2}s", metrics.traditional_time);
    println!("  - Average Confidence: {:.2}", metrics.traditional_confidence);
    println!("\nPrism Implementation:");
    println!("  - Time: {:.2}s", metrics.prism_time);
    println!("  - Confidence: {:.2}", metrics.prism_confidence);
    println!("\nImprovements:");
    println!("  - Speed: {:.1}% faster", metrics.speed_improvement);
    println!("  - Code Size: ~60% reduction (350 lines vs 150 lines)");

    Ok(())
} 