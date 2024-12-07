use prism::Interpreter;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Initialize the interpreter
    let interpreter = Interpreter::new();

    // Load the medical diagnosis program
    let source = r#"
        fn diagnose(symptoms) {
            let conditions = [
                { name: "Cold", symptoms: ["fever", "cough", "runny nose"] },
                { name: "Flu", symptoms: ["fever", "body aches", "fatigue"] },
                { name: "Allergies", symptoms: ["sneezing", "itchy eyes", "runny nose"] }
            ];

            let matches = [];
            for condition in conditions {
                let match_count = 0;
                for symptom in condition.symptoms {
                    if symptoms.contains(symptom) {
                        match_count += 1;
                    }
                }
                if match_count > 0 {
                    matches.push({ condition: condition.name, confidence: match_count / condition.symptoms.length });
                }
            }
            return matches;
        }
    "#;

    let result = interpreter.evaluate(source.to_string()).await?;
    println!("Result: {:?}", result);

    Ok(())
}
