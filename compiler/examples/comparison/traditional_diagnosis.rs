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
            "You are a medical symptom validator. Your task is to validate if '{}' is a clear and valid medical symptom.\n\
            Return ONLY a number between 0 and 1 representing the confidence score, where:\n\
            1.0 = Clear, specific, well-defined medical symptom (e.g., 'fever', 'shortness of breath')\n\
            0.7-0.9 = Valid but could be more specific (e.g., 'pain', 'discomfort')\n\
            0.4-0.6 = Ambiguous or general (e.g., 'feeling bad', 'not well')\n\
            0.0-0.3 = Not a valid medical symptom (e.g., 'blue thoughts', 'happy')\n\n\
            Respond with ONLY the number, no other text.\n\
            Example responses: '0.95' or '0.3' or '0.0'\n\n\
            Confidence score:",
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
            max_output_tokens: Some(5),
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
            // Clean up the response text to handle potential formatting
            let cleaned_text = text.trim()
                .replace("Confidence score:", "")
                .replace("confidence:", "")
                .trim()
                .to_string();
            cleaned_text.parse::<f64>().map_err(|_| format!("Failed to parse confidence from response: {}", text))?
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
            "You are a medical symptom matcher. Compare these two sets of symptoms and return ONLY a number between 0 and 1 indicating how well they match.\n\n\
            Set 1: '{}'\n\
            Set 2: '{}'\n\n\
            Consider:\n\
            - Symptom overlap (exact and semantic matches)\n\
            - Symptom specificity\n\
            - Pattern completeness\n\n\
            Examples:\n\
            - Perfect match = 1.0\n\
            - Strong match with minor variations = 0.8-0.9\n\
            - Moderate match with some differences = 0.5-0.7\n\
            - Poor match with major differences = 0.2-0.4\n\
            - No meaningful match = 0.0-0.1\n\n\
            Return ONLY the number, no other text.\n\
            Example responses: '0.95' or '0.3' or '0.0'\n\n\
            Match score:",
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
            max_output_tokens: Some(5),
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
            // Clean up the response text to handle potential formatting
            let cleaned_text = text.trim()
                .replace("Match score:", "")
                .replace("score:", "")
                .trim()
                .to_string();
            cleaned_text.parse::<f64>().map_err(|_| format!("Failed to parse confidence from response: {}", text))?
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
            "You are a medical knowledge base. List the most common symptoms for {}.\n\
            Rules:\n\
            1. Return ONLY a comma-separated list of symptoms\n\
            2. Focus on specific, observable symptoms\n\
            3. List them in order of frequency/importance\n\
            4. Use standard medical terminology\n\
            5. Include 5-10 key symptoms\n\
            6. NO additional text or explanations\n\n\
            Example response format:\n\
            fever, cough, fatigue, shortness of breath, body aches\n\n\
            Symptoms:",
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
            text.trim()
                .replace("Symptoms:", "")
                .trim()
                .to_string()
        } else {
            return Err("No response text received".into());
        };

        let duration = start.elapsed();
        println!("Traditional get_disease_pattern took: {:?}", duration);

        Ok(pattern)
    }
} 