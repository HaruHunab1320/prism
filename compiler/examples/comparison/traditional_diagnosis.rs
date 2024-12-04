use google_generative_ai_rs::v1::{
    api::Client,
    gemini::{Content, Part, Role, request::{Request, GenerationConfig}},
};
use std::error::Error;

pub struct MedicalDiagnosisSystem {
    client: Client,
}

impl MedicalDiagnosisSystem {
    pub fn new(api_key: &str) -> Self {
        Self {
            client: Client::new(api_key.to_string()),
        }
    }

    pub async fn validate_symptom(&self, symptom: &str) -> Result<f64, Box<dyn Error>> {
        let prompt = format!(
            "Validate if '{}' is a clear and valid medical symptom. \
            Return a confidence score between 0 and 1, where:\n\
            1.0 = Clear, specific medical symptom\n\
            0.7-0.9 = Valid but could be more specific\n\
            0.4-0.6 = Ambiguous or general\n\
            0.0-0.3 = Not a valid medical symptom\n\
            Return only the number.",
            symptom
        );

        let content = Content {
            parts: vec![Part {
                text: Some(prompt),
                inline_data: None,
                file_data: None,
                video_metadata: None,
            }],
            role: Role::User,
        };

        let config = GenerationConfig {
            temperature: Some(0.0),
            top_p: Some(1.0),
            top_k: Some(1),
            candidate_count: Some(1),
            max_output_tokens: Some(1),
            stop_sequences: Some(vec![]),
        };

        let request = Request {
            contents: vec![content],
            generation_config: Some(config),
            safety_settings: vec![],
            tools: vec![],
        };

        let response = self.client.post(60, &request).await?
            .rest()
            .ok_or("No response received")?;

        let text = response.candidates.first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| p.text.as_ref())
            .ok_or("No response text received")?;

        text.trim().parse::<f64>()
            .map_err(|e| format!("Failed to parse confidence: {}", e).into())
    }

    pub async fn process_symptoms(&self, symptoms: &str) -> Result<DiagnosisResult, Box<dyn Error>> {
        // Split and format symptoms
        let symptom_list: Vec<String> = symptoms
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        // Validate each symptom
        let mut total_confidence = 0.0;
        for symptom in &symptom_list {
            total_confidence += self.validate_symptom(symptom).await?;
        }
        let avg_confidence = total_confidence / symptom_list.len() as f64;

        // Get diagnosis
        let formatted_symptoms = symptom_list.join("; ");
        let diagnosis = self.get_diagnosis(&formatted_symptoms).await?;

        // Check against known patterns
        let match_score = self.check_pattern(&formatted_symptoms, &diagnosis).await?;

        Ok(DiagnosisResult {
            formatted_symptoms,
            confidence: avg_confidence,
            diagnosis,
            pattern_match: match_score,
        })
    }

    pub async fn get_diagnosis(&self, symptoms: &str) -> Result<String, Box<dyn Error>> {
        let prompt = format!(
            "Based on these symptoms: {}\n\
            Provide a likely diagnosis. Be concise.",
            symptoms
        );

        let content = Content {
            parts: vec![Part {
                text: Some(prompt),
                inline_data: None,
                file_data: None,
                video_metadata: None,
            }],
            role: Role::User,
        };

        let config = GenerationConfig {
            temperature: Some(0.7),
            top_p: Some(1.0),
            top_k: Some(1),
            candidate_count: Some(1),
            max_output_tokens: Some(100),
            stop_sequences: Some(vec![]),
        };

        let request = Request {
            contents: vec![content],
            generation_config: Some(config),
            safety_settings: vec![],
            tools: vec![],
        };

        let response = self.client.post(60, &request).await?
            .rest()
            .ok_or("No response received")?;

        response.candidates.first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| p.text.as_ref())
            .map(|t| t.trim().to_string())
            .ok_or_else(|| "No diagnosis received".into())
    }

    pub async fn check_pattern(&self, symptoms: &str, diagnosis: &str) -> Result<f64, Box<dyn Error>> {
        let prompt = format!(
            "Compare these symptoms: '{}'\n\
            with the typical pattern for: '{}'\n\
            Return a confidence score between 0 and 1 indicating how well they match.\n\
            Consider symptom overlap and specificity.\n\
            Return only the number.",
            symptoms, diagnosis
        );

        let content = Content {
            parts: vec![Part {
                text: Some(prompt),
                inline_data: None,
                file_data: None,
                video_metadata: None,
            }],
            role: Role::User,
        };

        let config = GenerationConfig {
            temperature: Some(0.0),
            top_p: Some(1.0),
            top_k: Some(1),
            candidate_count: Some(1),
            max_output_tokens: Some(1),
            stop_sequences: Some(vec![]),
        };

        let request = Request {
            contents: vec![content],
            generation_config: Some(config),
            safety_settings: vec![],
            tools: vec![],
        };

        let response = self.client.post(60, &request).await?
            .rest()
            .ok_or("No response received")?;

        let text = response.candidates.first()
            .and_then(|c| c.content.parts.first())
            .and_then(|p| p.text.as_ref())
            .ok_or("No response text received")?;

        text.trim().parse::<f64>()
            .map_err(|e| format!("Failed to parse match score: {}", e).into())
    }
}

#[derive(Debug)]
pub struct DiagnosisResult {
    pub formatted_symptoms: String,
    pub confidence: f64,
    pub diagnosis: String,
    pub pattern_match: f64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load API key from environment
    let api_key = std::env::var("GOOGLE_API_KEY")?;
    
    // Initialize the system
    let system = MedicalDiagnosisSystem::new(&api_key);
    
    // Process symptoms
    let symptoms = "severe headache, sensitivity to light, nausea";
    let result = system.process_symptoms(symptoms).await?;
    
    // Output results
    println!("Diagnosis Results:");
    println!("Symptoms: {}", result.formatted_symptoms);
    println!("Symptom Confidence: {:.2}", result.confidence);
    println!("Diagnosis: {}", result.diagnosis);
    println!("Pattern Match: {:.2}", result.pattern_match);
    
    Ok(())
} 