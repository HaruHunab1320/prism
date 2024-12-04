use std::error::Error;
use std::time::Instant;
use dotenv::dotenv;
use std::env;

mod traditional_diagnosis;
use traditional_diagnosis::TraditionalDiagnosisSystem;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();

    let api_key = env::var("GOOGLE_API_KEY").expect("GOOGLE_API_KEY must be set");
    let traditional_system = TraditionalDiagnosisSystem::new(api_key);

    // Test symptom validation
    let symptoms = vec![
        "fever",
        "cough",
        "fatigue",
        "shortness of breath",
        "loss of taste",
    ];

    println!("\nTesting symptom validation...");
    for symptom in symptoms {
        match traditional_system.validate_symptom(symptom).await {
            Ok(validation) => {
                println!(
                    "Symptom: {}, Confidence: {:.2}, Source: {}",
                    validation.symptom, validation.confidence, validation.validation_source
                );
            }
            Err(e) => println!("Error validating symptom {}: {}", symptom, e),
        }
    }

    // Test disease pattern matching
    println!("\nTesting disease pattern matching...");
    let test_symptoms = "fever, cough, fatigue, shortness of breath";
    let test_pattern = "fever, dry cough, fatigue, difficulty breathing, loss of taste and smell";
    match traditional_system.semantic_match(test_symptoms, test_pattern).await {
        Ok(confidence) => println!("Match confidence: {:.2}", confidence),
        Err(e) => println!("Error matching disease pattern: {}", e),
    }

    // Test disease pattern retrieval
    println!("\nTesting disease pattern retrieval...");
    let diseases = vec!["flu", "covid19", "common_cold"];
    for disease in diseases {
        match traditional_system.get_disease_pattern(disease).await {
            Ok(pattern) => println!("Disease: {}, Pattern: {}", disease, pattern),
            Err(e) => println!("Error getting disease pattern for {}: {}", disease, e),
        }
    }

    Ok(())
} 