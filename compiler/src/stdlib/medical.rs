use crate::llm::LLMClient;
use crate::types::Value;
use std::error::Error;

pub struct MedicalLLM {
    client: LLMClient,
}

impl MedicalLLM {
    pub fn new(api_key: String) -> Self {
        Self {
            client: LLMClient::new(api_key),
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

        let response = self.client.generate(&prompt).await?;
        response.parse::<f64>().map_err(|e| e.into())
    }

    pub async fn semantic_match(&self, symptoms: &str, disease_pattern: &str) -> Result<f64, Box<dyn Error>> {
        let prompt = format!(
            "Compare these symptoms: '{}'\n\
            with this disease pattern: '{}'\n\
            Return a confidence score between 0 and 1 indicating how well they match.\n\
            Consider:\n\
            - Symptom overlap\n\
            - Symptom specificity\n\
            - Pattern completeness\n\
            Return only the number.",
            symptoms, disease_pattern
        );

        let response = self.client.generate(&prompt).await?;
        response.parse::<f64>().map_err(|e| e.into())
    }

    pub async fn get_disease_pattern(&self, condition: &str) -> Result<String, Box<dyn Error>> {
        let prompt = format!(
            "Provide a concise, comma-separated list of the most common symptoms for {}.\n\
            Focus on specific, observable symptoms.\n\
            Return only the symptoms, no additional text.",
            condition
        );

        self.client.generate(&prompt).await
    }
}

// Register medical LLM functions in the standard library
pub fn register_medical_functions(env: &mut Environment) {
    env.define(
        "llm.validate_symptom".to_string(),
        Value::Function(
            vec!["symptom".to_string()],
            vec![Stmt::Expression(Expr::Call {
                function: Box::new(Expr::Identifier("__internal_validate_symptom".to_string())),
                arguments: vec![Expr::Identifier("symptom".to_string())],
            })],
        ),
    );

    env.define(
        "llm.semantic_match".to_string(),
        Value::Function(
            vec!["symptoms".to_string(), "pattern".to_string()],
            vec![Stmt::Expression(Expr::Call {
                function: Box::new(Expr::Identifier("__internal_semantic_match".to_string())),
                arguments: vec![
                    Expr::Identifier("symptoms".to_string()),
                    Expr::Identifier("pattern".to_string()),
                ],
            })],
        ),
    );

    env.define(
        "llm.get_disease_pattern".to_string(),
        Value::Function(
            vec!["condition".to_string()],
            vec![Stmt::Expression(Expr::Call {
                function: Box::new(Expr::Identifier("__internal_get_disease_pattern".to_string())),
                arguments: vec![Expr::Identifier("condition".to_string())],
            })],
        ),
    );
} 