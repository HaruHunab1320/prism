#[cfg(test)]
mod tests {
    use prism::llm::{
        LLMClient, Provider, CompletionRequest, CompletionResponse, ModelConfig, TokenUsage
    };
    use std::env;
    use mockito::Server;
    use tokio::runtime::Runtime;

    fn setup() -> Runtime {
        Runtime::new().expect("Failed to create runtime")
    }

    #[test]
    fn test_gemini_client_creation() {
        let api_key = "test_key".to_string();
        let client = LLMClient::new(Provider::Google(api_key.clone()));
        assert!(matches!(client.get_provider(), &Provider::Google(_)));
    }

    #[test]
    fn test_gemini_client_with_config() {
        let api_key = "test_key".to_string();
        let config = ModelConfig {
            model: "gemini-pro".to_string(),
            temperature: 0.5,
            max_tokens: 500,
            timeout_secs: 20,
            max_retries: 2,
        };
        let client = LLMClient::with_config(Provider::Google(api_key), config.clone());
        let client_config = client.get_config();
        assert_eq!(client_config.model, "gemini-pro");
        assert_eq!(client_config.temperature, 0.5);
        assert_eq!(client_config.max_tokens, 500);
    }

    #[test]
    fn test_gemini_mock_completion() {
        let rt = setup();
        let mut server = Server::new();
        
        let mock_response = r#"{
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "text": "Test response"
                    }]
                },
                "finish_reason": "STOP"
            }],
            "prompt_feedback": {
                "token_count": {
                    "total_tokens": 50,
                    "prompt_tokens": 20
                }
            }
        }"#;

        let _m = server.mock("POST", "/v1/models/gemini-pro:generateContent")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = LLMClient::new(Provider::Google("test_key".to_string()));
        let request = CompletionRequest {
            prompt: "Test prompt".to_string(),
            context: None,
            config: None,
        };

        let response = rt.block_on(async {
            client.complete(request).await.unwrap()
        });

        assert_eq!(response.text, "Test response");
        assert_eq!(response.confidence, 0.95); // STOP finish reason
        assert_eq!(response.model, "gemini-pro");
        assert_eq!(response.usage.total_tokens, 50);
        assert_eq!(response.usage.prompt_tokens, 20);
        assert_eq!(response.usage.completion_tokens, 30);
    }

    #[test]
    fn test_gemini_error_handling() {
        let rt = setup();
        let mut server = Server::new();
        
        let _m = server.mock("POST", "/v1/models/gemini-pro:generateContent")
            .with_status(400)
            .with_body(r#"{"error": {"message": "Invalid request"}}"#)
            .create();

        let client = LLMClient::new(Provider::Google("test_key".to_string()));
        let request = CompletionRequest {
            prompt: "Test prompt".to_string(),
            context: None,
            config: None,
        };

        let result = rt.block_on(async {
            client.complete(request).await
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_gemini_with_context() {
        let rt = setup();
        use prism::context::Context;

        let mut server = Server::new();
        
        let mock_response = r#"{
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "text": "Context-aware response"
                    }]
                },
                "finish_reason": "STOP"
            }],
            "prompt_feedback": {
                "token_count": {
                    "total_tokens": 60,
                    "prompt_tokens": 30
                }
            }
        }"#;

        let _m = server.mock("POST", "/v1/models/gemini-pro:generateContent")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = LLMClient::new(Provider::Google("test_key".to_string()));
        let mut context = Context::new("test_context".to_string());
        context.set_confidence(0.9);

        let request = CompletionRequest {
            prompt: "Test prompt".to_string(),
            context: Some(context),
            config: None,
        };

        let response = rt.block_on(async {
            client.complete(request).await.unwrap()
        });
        assert_eq!(response.text, "Context-aware response");
        assert!(response.confidence > 0.0);
    }

    #[test]
    fn test_gemini_confidence_calculation() {
        let rt = setup();
        let client = LLMClient::new(Provider::Google("test_key".to_string()));
        let mut server = Server::new();
        
        // Test different finish reasons
        let test_cases = vec![
            ("STOP", 0.95),
            ("MAX_TOKENS", 0.7),
            ("OTHER", 0.5),
        ];

        for (finish_reason, expected_confidence) in test_cases {
            let mock_response = format!(r#"{{
                "candidates": [{{
                    "content": {{
                        "role": "model",
                        "parts": [{{
                            "text": "Test response"
                        }}]
                    }},
                    "finish_reason": "{}"
                }}],
                "prompt_feedback": {{
                    "token_count": {{
                        "total_tokens": 50,
                        "prompt_tokens": 20
                    }}
                }}
            }}"#, finish_reason);

            let _m = server.mock("POST", "/v1/models/gemini-pro:generateContent")
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(&mock_response)
                .create();

            let request = CompletionRequest {
                prompt: "Test prompt".to_string(),
                context: None,
                config: None,
            };

            let response = rt.block_on(async {
                client.complete(request).await.unwrap()
            });
            assert_eq!(response.confidence, expected_confidence);
        }
    }

    #[test]
    fn test_gemini_token_usage() {
        let rt = setup();
        let mut server = Server::new();
        
        let mock_response = r#"{
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "text": "Test response"
                    }]
                },
                "finish_reason": "STOP"
            }],
            "prompt_feedback": {
                "token_count": {
                    "total_tokens": 100,
                    "prompt_tokens": 40
                }
            }
        }"#;

        let _m = server.mock("POST", "/v1/models/gemini-pro:generateContent")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create();

        let client = LLMClient::new(Provider::Google("test_key".to_string()));
        let request = CompletionRequest {
            prompt: "Test prompt".to_string(),
            context: None,
            config: Some(ModelConfig {
                max_tokens: 1000,
                ..Default::default()
            }),
        };

        let response = rt.block_on(async {
            client.complete(request).await.unwrap()
        });
        assert_eq!(response.usage.total_tokens, 100);
        assert_eq!(response.usage.prompt_tokens, 40);
        assert_eq!(response.usage.completion_tokens, 60);
    }

    // Integration test with real API (only runs if GOOGLE_API_KEY is set)
    #[test]
    fn test_gemini_live_api() {
        let rt = setup();
        let api_key = match env::var("GOOGLE_API_KEY") {
            Ok(key) => key,
            Err(_) => return, // Skip test if no API key is provided
        };

        let client = LLMClient::new(Provider::Google(api_key));
        let request = CompletionRequest {
            prompt: "What is 2+2? Answer with just the number.".to_string(),
            context: None,
            config: Some(ModelConfig {
                temperature: 0.0, // Set to 0 for deterministic output
                max_tokens: 10,
                ..Default::default()
            }),
        };

        let response = rt.block_on(async {
            client.complete(request).await.unwrap()
        });
        assert!(!response.text.is_empty());
        assert!(response.text.contains("4"));
        assert!(response.confidence > 0.0 && response.confidence <= 1.0);
        assert!(response.usage.total_tokens > 0);
    }
} 