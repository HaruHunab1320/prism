use std::error::Error;
use std::time::Instant;
use std::sync::Arc;
use google_generative_ai_rs::v1::api::Client;
use google_generative_ai_rs::v1::gemini::{Content, Part, Role};
use google_generative_ai_rs::v1::gemini::request::{Request, GenerationConfig};

// Traditional implementation requires explicit confidence handling
#[derive(Debug, Clone)]
pub struct SymptomValidation {
    pub symptom: String,
    pub confidence: f64,
    pub validation_source: String,
}

#[derive(Debug)]
pub struct DiagnosisResult {
    pub condition: String,
    pub confidence: f64,
    pub supporting_evidence: Vec<String>,
}

pub struct TraditionalDiagnosisSystem {
    client: Arc<Client>,
}

impl TraditionalDiagnosisSystem {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Arc::new(Client::new(api_key)),
        }
    }

    pub async fn validate_symptom(&self, symptom: &str) -> Result<SymptomValidation, Box<dyn Error>> {
        let start = Instant::now();

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

        let response = (*self.client).post(60, &request).await?.rest().ok_or_else(|| "No response received")?;
        let confidence = if let Some(text) = response.candidates.first().and_then(|c| c.content.parts.first()).and_then(|p| p.text.as_ref()) {
            text.trim().parse::<f64>()?
        } else {
            return Err("No response text received".into());
        };

        let duration = start.elapsed();
        println!("Traditional validate_symptom took: {:?}", duration);

        Ok(SymptomValidation {
            symptom: symptom.to_string(),
            confidence,
            validation_source: "Gemini API".to_string(),
        })
    }

    pub async fn semantic_match(&self, symptoms: &str, pattern: &str) -> Result<f64, Box<dyn Error>> {
        let start = Instant::now();

        let prompt = format!(
            "Compare these symptoms: '{}'\n\
            with this disease pattern: '{}'\n\
            Return a confidence score between 0 and 1 indicating how well they match.\n\
            Consider:\n\
            - Symptom overlap\n\
            - Symptom specificity\n\
            - Pattern completeness\n\
            Return only the number.",
            symptoms, pattern
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

        let response = (*self.client).post(60, &request).await?.rest().ok_or_else(|| "No response received")?;
        let confidence = if let Some(text) = response.candidates.first().and_then(|c| c.content.parts.first()).and_then(|p| p.text.as_ref()) {
            text.trim().parse::<f64>()?
        } else {
            return Err("No response text received".into());
        };

        let duration = start.elapsed();
        println!("Traditional semantic_match took: {:?}", duration);

        Ok(confidence)
    }

    pub async fn get_disease_pattern(&self, disease: &str) -> Result<String, Box<dyn Error>> {
        let start = Instant::now();

        let prompt = format!(
            "Provide a concise, comma-separated list of the most common symptoms for {}.\n\
            Focus on specific, observable symptoms.\n\
            Return only the symptoms, no additional text.",
            disease
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
            max_output_tokens: Some(100),
            stop_sequences: Some(vec![]),
        };

        let request = Request {
            contents: vec![content],
            generation_config: Some(config),
            safety_settings: vec![],
            tools: vec![],
        };

        let response = (*self.client).post(60, &request).await?.rest().ok_or_else(|| "No response received")?;
        let pattern = if let Some(text) = response.candidates.first().and_then(|c| c.content.parts.first()).and_then(|p| p.text.as_ref()) {
            text.trim().to_string()
        } else {
            return Err("No response text received".into());
        };

        let duration = start.elapsed();
        println!("Traditional get_disease_pattern took: {:?}", duration);

        Ok(pattern)
    }
} 