use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use crate::{Interpreter, Value, RuntimeError};

#[derive(Serialize, Deserialize)]
struct PrismValue {
    value: Value,
    confidence: f64,
    context: Option<String>,
}

#[wasm_bindgen]
pub struct PrismRuntime {
    interpreter: Interpreter,
}

#[wasm_bindgen]
impl PrismRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        console_error_panic_hook::set_once();
        Self {
            interpreter: Interpreter::new()
        }
    }

    #[wasm_bindgen]
    pub async fn eval(&mut self, code: &str) -> Result<JsValue, JsError> {
        let result = self.interpreter.eval_async(code).await
            .map_err(|e| JsError::new(&e.to_string()))?;
        
        let prism_value = PrismValue {
            confidence: result.get_confidence().unwrap_or(1.0),
            context: result.get_context().map(|s| s.to_string()),
            value: result,
        };

        serde_wasm_bindgen::to_value(&prism_value)
            .map_err(|e| JsError::new(&e.to_string()))
    }

    #[wasm_bindgen]
    pub fn get_confidence(&self, value: &JsValue) -> Result<f64, JsError> {
        let prism_value: PrismValue = serde_wasm_bindgen::from_value(value.clone())
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(prism_value.confidence)
    }

    #[wasm_bindgen]
    pub fn get_context(&self, value: &JsValue) -> Result<Option<String>, JsError> {
        let prism_value: PrismValue = serde_wasm_bindgen::from_value(value.clone())
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(prism_value.context)
    }
}

// Helper functions for TypeScript
#[wasm_bindgen]
pub fn create_value_with_confidence(value: JsValue, confidence: f64) -> Result<JsValue, JsError> {
    let prism_value = PrismValue {
        value: serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsError::new(&e.to_string()))?,
        confidence,
        context: None,
    };

    serde_wasm_bindgen::to_value(&prism_value)
        .map_err(|e| JsError::new(&e.to_string()))
}

#[wasm_bindgen]
pub fn create_value_in_context(value: JsValue, context: String) -> Result<JsValue, JsError> {
    let prism_value = PrismValue {
        value: serde_wasm_bindgen::from_value(value)
            .map_err(|e| JsError::new(&e.to_string()))?,
        confidence: 1.0,
        context: Some(context),
    };

    serde_wasm_bindgen::to_value(&prism_value)
        .map_err(|e| JsError::new(&e.to_string()))
} 