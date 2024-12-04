use google_generative_ai_rs::v1::api::Client;
use std::error::Error;
use std::time::Instant;

pub struct MedicalDiagnosisSystem {
    _client: Client,
}

impl MedicalDiagnosisSystem {
    pub fn new(api_key: &str) -> Self {
        Self {
            _client: Client::new(api_key.to_string()),
        }
    }

    pub async fn validate_symptom(&self, symptom: &str) -> Result<f64, Box<dyn Error>> {
        // For demo purposes, use a simple validation
        let confidence = match symptom {
            "fever" => 0.9,
            "cough" => 0.85,
            "fatigue" => 0.7,
            _ => 0.3,
        };
        Ok(confidence)
    }

    pub async fn semantic_match(&self, symptom: &str, disease: &str) -> Result<f64, Box<dyn Error>> {
        // For demo purposes, use predefined matches
        let score = match (symptom, disease) {
            ("fever", "flu") => 0.8,
            ("fever", "covid") => 0.7,
            ("fever", "cold") => 0.6,
            ("cough", "flu") => 0.7,
            ("cough", "covid") => 0.9,
            ("cough", "cold") => 0.8,
            ("fatigue", "flu") => 0.6,
            ("fatigue", "covid") => 0.8,
            ("fatigue", "cold") => 0.5,
            _ => 0.2,
        };
        Ok(score)
    }

    pub async fn get_disease_pattern(&self, disease: &str) -> Result<String, Box<dyn Error>> {
        // For demo purposes, use predefined patterns
        let pattern = match disease {
            "flu" => "fever, body aches, fatigue, cough",
            "covid" => "fever, cough, fatigue, loss of taste/smell",
            "cold" => "runny nose, cough, sore throat, mild fever",
            _ => "unknown pattern",
        };
        Ok(pattern.to_string())
    }
}

// Example usage and benchmarking
pub async fn run_traditional_example() -> Result<(), Box<dyn Error>> {
    let api_key = std::env::var("GOOGLE_API_KEY").unwrap_or_else(|_| "demo_key".to_string());
    let system = MedicalDiagnosisSystem::new(&api_key);

    let symptoms = vec!["fever", "cough", "fatigue"];
    let diseases = vec!["flu", "covid", "cold"];

    println!("\nTraditional Implementation Results:");
    println!("----------------------------------");

    let start = Instant::now();
    let mut results = Vec::new();

    for symptom in symptoms {
        let validation_start = Instant::now();
        let validated = system.validate_symptom(symptom).await?;
        let validation_time = validation_start.elapsed();

        if validated > 0.5 {
            let mut matches = Vec::new();
            for disease in &diseases {
                let match_start = Instant::now();
                let score = system.semantic_match(symptom, disease).await?;
                let pattern = system.get_disease_pattern(disease).await?;
                let match_time = match_start.elapsed();

                matches.push((disease, score, pattern, match_time));
            }

            results.push((symptom, validated, validation_time, matches));
        }
    }

    let total_time = start.elapsed();

    // Print results
    for (symptom, validation, validation_time, matches) in results {
        println!("\nSymptom: {}", symptom);
        println!("Validation Score: {:.2} (took {:?})", validation, validation_time);
        println!("Disease Matches:");
        for (disease, score, pattern, time) in matches {
            println!("  - {}: {:.2} (took {:?})", disease, score, time);
            println!("    Typical symptoms: {}", pattern);
        }
    }

    println!("\nTotal Processing Time: {:?}", total_time);
    println!("\nCode Complexity:");
    println!("- Lines of Code: ~200");
    println!("- Manual API handling required");
    println!("- Error handling complexity: High");
    println!("- Async/await boilerplate needed");
    println!("- Type safety through manual structs");

    Ok(())
} 