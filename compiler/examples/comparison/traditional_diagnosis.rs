use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub struct MedicalAPI {
    client: Client,
    api_key: String,
}

impl MedicalAPI {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }

    pub async fn validate_symptom(&self, symptom: &str) -> Result<f64, Box<dyn Error>> {
        let response = self.client
            .post("https://api.medical.ai/validate")
            .header("Authorization", &self.api_key)
            .json(&json!({
                "symptom": symptom
            }))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["confidence"].as_f64().unwrap_or(0.0))
    }

    pub async fn get_disease_pattern(&self, disease: &str) -> Result<String, Box<dyn Error>> {
        let response = self.client
            .get(&format!("https://api.medical.ai/patterns/{}", disease))
            .header("Authorization", &self.api_key)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["pattern"].as_str().unwrap_or("").to_string())
    }

    pub async fn semantic_match(&self, symptoms: &[&str], pattern: &str) -> Result<f64, Box<dyn Error>> {
        let response = self.client
            .post("https://api.medical.ai/match")
            .header("Authorization", &self.api_key)
            .json(&json!({
                "symptoms": symptoms,
                "pattern": pattern
            }))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["score"].as_f64().unwrap_or(0.0))
    }
}

pub async fn diagnose(api_key: String) -> Result<Vec<serde_json::Value>, Box<dyn Error>> {
    let api = MedicalAPI::new(api_key);
    let symptoms = vec!["fever", "cough", "fatigue"];
    let diseases = vec!["flu", "covid", "cold"];

    let mut validated_symptoms = Vec::new();
    for symptom in symptoms {
        let confidence = api.validate_symptom(symptom).await?;
        if confidence > 0.7 {
            validated_symptoms.push(symptom);
        }
    }

    let mut results = Vec::new();
    for disease in diseases {
        let pattern = api.get_disease_pattern(disease).await?;
        let match_score = api.semantic_match(&validated_symptoms, &pattern).await?;

        let severity = if match_score > 0.8 {
            "high"
        } else if match_score > 0.5 {
            "medium"
        } else {
            "low"
        };

        results.push(json!({
            "disease": disease,
            "confidence": match_score,
            "severity": severity
        }));
    }

    results.sort_by(|a, b| {
        b["confidence"].as_f64().unwrap_or(0.0)
            .partial_cmp(&a["confidence"].as_f64().unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(results)
} 