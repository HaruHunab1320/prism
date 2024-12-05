use std::error::Error;
use prism::Interpreter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = std::env::var("PRISM_API_KEY").unwrap_or_else(|_| "test_key".to_string());
    let mut interpreter = Interpreter::new(api_key);

    let source = r#"
    let symptoms = ["fever", "cough", "fatigue"];
    let diseases = ["flu", "covid", "cold"];

    // Validate each symptom
    let validated_symptoms = [];
    for symptom in symptoms {
        let confidence = medical.validate_symptom(symptom);
        if confidence > 0.7 {
            validated_symptoms.push(symptom ~> confidence);
        }
    }

    // Match symptoms against diseases
    let results = [];
    for disease in diseases {
        let pattern = medical.get_disease_pattern(disease);
        let match_score = medical.semantic_match(validated_symptoms, pattern);
        
        uncertain if (match_score > 0.8) {
            results.push({
                "disease": disease,
                "confidence": match_score,
                "severity": "high"
            });
        } medium {
            results.push({
                "disease": disease,
                "confidence": match_score,
                "severity": "medium"
            });
        } low {
            results.push({
                "disease": disease,
                "confidence": match_score,
                "severity": "low"
            });
        }
    }

    // Sort results by confidence
    results.sort((a, b) => b.confidence - a.confidence);
    results
    "#;

    let result = interpreter.eval(source.to_string()).await?;
    println!("Diagnosis Results: {:#?}", result);
    Ok(())
} 