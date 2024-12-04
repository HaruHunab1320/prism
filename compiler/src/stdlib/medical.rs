use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use crate::interpreter::Interpreter;
use crate::error::RuntimeError;
use crate::types::Value;
use google_generative_ai_rs::v1::api::Client;
use google_generative_ai_rs::v1::gemini::{Content, Part, Role, ResponseType};
use google_generative_ai_rs::v1::gemini::request::GenerationConfig;
use futures::StreamExt;

pub struct MedicalLLM {
    client: Arc<Client>,
}

impl MedicalLLM {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Arc::new(Client::new(api_key)),
        }
    }

    pub fn register_functions(&self, interpreter: &mut Interpreter) {
        let client = self.client.clone();
        interpreter.register_native_function(
            "llm.validate_symptom",
            move |_interpreter: &mut Interpreter, args: Vec<Value>| {
                let client = client.clone();
                Box::pin(async move {
                    if args.len() != 1 {
                        return Err(RuntimeError::TypeError(format!("validate_symptom() takes exactly 1 argument, got {}", args.len())));
                    }

                    let symptom = match &args[0] {
                        Value::String(s) => s,
                        _ => return Err(RuntimeError::TypeError(format!("validate_symptom() argument must be a string, got {}", args[0].get_type()))),
                    };

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

                    let response = (*client).generate_content_async("gemini-pro", &content, Some(&config), ResponseType::GenerateContent).await?;
                    if let Some(text) = response.candidates.first().and_then(|c| c.content.parts.first()).and_then(|p| p.text.as_ref()) {
                        match text.trim().parse::<f64>() {
                            Ok(confidence) => Ok(Value::Float(confidence)),
                            Err(e) => Err(RuntimeError::AsyncError(format!("Failed to parse confidence: {}", e))),
                        }
                    } else {
                        Err(RuntimeError::AsyncError("No response text received".to_string()))
                    }
                })
            },
        );

        let client = self.client.clone();
        interpreter.register_native_function(
            "llm.semantic_match",
            move |_interpreter: &mut Interpreter, args: Vec<Value>| {
                let client = client.clone();
                Box::pin(async move {
                    if args.len() != 2 {
                        return Err(RuntimeError::TypeError(format!("semantic_match() takes exactly 2 arguments, got {}", args.len())));
                    }

                    let symptoms = match &args[0] {
                        Value::String(s) => s,
                        _ => return Err(RuntimeError::TypeError(format!("semantic_match() first argument must be a string, got {}", args[0].get_type()))),
                    };

                    let pattern = match &args[1] {
                        Value::String(s) => s,
                        _ => return Err(RuntimeError::TypeError(format!("semantic_match() second argument must be a string, got {}", args[1].get_type()))),
                    };

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

                    let response = (*client).generate_content_async("gemini-pro", &content, Some(&config), ResponseType::GenerateContent).await?;
                    if let Some(text) = response.candidates.first().and_then(|c| c.content.parts.first()).and_then(|p| p.text.as_ref()) {
                        match text.trim().parse::<f64>() {
                            Ok(confidence) => Ok(Value::Float(confidence)),
                            Err(e) => Err(RuntimeError::AsyncError(format!("Failed to parse confidence: {}", e))),
                        }
                    } else {
                        Err(RuntimeError::AsyncError("No response text received".to_string()))
                    }
                })
            },
        );

        let client = self.client.clone();
        interpreter.register_native_function(
            "llm.get_disease_pattern",
            move |_interpreter: &mut Interpreter, args: Vec<Value>| {
                let client = client.clone();
                Box::pin(async move {
                    if args.len() != 1 {
                        return Err(RuntimeError::TypeError(format!("get_disease_pattern() takes exactly 1 argument, got {}", args.len())));
                    }

                    let disease = match &args[0] {
                        Value::String(s) => s,
                        _ => return Err(RuntimeError::TypeError(format!("get_disease_pattern() argument must be a string, got {}", args[0].get_type()))),
                    };

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

                    let response = (*client).generate_content_async("gemini-pro", &content, Some(&config), ResponseType::GenerateContent).await?;
                    if let Some(text) = response.candidates.first().and_then(|c| c.content.parts.first()).and_then(|p| p.text.as_ref()) {
                        Ok(Value::String(text.trim().to_string()))
                    } else {
                        Err(RuntimeError::AsyncError("No response text received".to_string()))
                    }
                })
            },
        );
    }
} 