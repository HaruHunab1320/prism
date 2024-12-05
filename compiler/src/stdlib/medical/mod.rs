use crate::ast::Value;
use crate::interpreter::Interpreter;

pub fn register_medical_functions(interpreter: &Interpreter) {
    interpreter.register_native_function("validate_symptom", |interpreter: &Interpreter, args: Vec<Value>| {
        let api_key = interpreter.get_api_key().to_string();
        Box::pin(async move {
            if args.len() != 1 {
                return Err("validate_symptom() takes exactly one argument".into());
            }
            
            let symptom = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("Argument must be a string".into()),
            };

            // Use LLM to validate if this is a real medical symptom
            let pattern = "This is a valid medical symptom";
            let client = crate::stdlib::llm::LLMClient::new(api_key);
            let confidence = client.semantic_match(&symptom, pattern).await?;

            Ok(Value::Number(confidence))
        })
    });

    interpreter.register_native_function("get_disease_pattern", |interpreter: &Interpreter, args: Vec<Value>| {
        let api_key = interpreter.get_api_key().to_string();
        Box::pin(async move {
            if args.len() != 1 {
                return Err("get_disease_pattern() takes exactly one argument".into());
            }
            
            let disease = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("Argument must be a string".into()),
            };

            // Create a pattern that describes the typical symptoms of the disease
            let pattern = format!("Common symptoms of {} include:", disease);
            let client = crate::stdlib::llm::LLMClient::new(api_key);
            let _confidence = client.semantic_match(&disease, &pattern).await?;

            Ok(Value::Pattern(pattern))
        })
    });
}
