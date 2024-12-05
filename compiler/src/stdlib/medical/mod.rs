use crate::ast::Value;
use crate::interpreter::Interpreter;

pub fn register_medical_functions(interpreter: &mut Interpreter) {
    interpreter.register_native_function("validate_symptom", |_, args| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("validate_symptom() takes exactly one argument".into());
            }
            
            let symptom = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("Argument must be a string".into()),
            };

            // TODO: Implement actual symptom validation
            // For now, return a mock confidence score
            let confidence = match symptom.as_str() {
                "fever" => 0.9,
                "cough" => 0.85,
                "fatigue" => 0.8,
                _ => 0.5,
            };

            Ok(Value::Number(confidence))
        })
    });

    interpreter.register_native_function("get_disease_pattern", |_, args| {
        Box::pin(async move {
            if args.len() != 1 {
                return Err("get_disease_pattern() takes exactly one argument".into());
            }
            
            let disease = match &args[0] {
                Value::String(s) => s.clone(),
                _ => return Err("Argument must be a string".into()),
            };

            // TODO: Implement actual disease pattern lookup
            // For now, return mock patterns
            let pattern = match disease.as_str() {
                "flu" => "fever AND (cough OR fatigue)",
                "covid" => "fever AND cough AND fatigue",
                "cold" => "(cough OR fever) AND NOT fatigue",
                _ => "unknown",
            };

            Ok(Value::String(pattern.to_string()))
        })
    });
}
