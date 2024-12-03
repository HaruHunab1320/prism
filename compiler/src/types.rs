use std::fmt;
use ndarray::Array;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Confidence {
    value: f64,
    source: Option<String>,
    timestamp: std::time::SystemTime,
}

impl Confidence {
    pub fn new(value: f64) -> Result<Self, String> {
        if value < 0.0 || value > 1.0 {
            return Err("Confidence value must be between 0.0 and 1.0".to_string());
        }
        Ok(Self {
            value,
            source: None,
            timestamp: std::time::SystemTime::now(),
        })
    }

    pub fn with_source(value: f64, source: String) -> Result<Self, String> {
        let mut conf = Self::new(value)?;
        conf.source = Some(source);
        Ok(conf)
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn decay(&self, decay_rate: f64) -> Self {
        let elapsed = self.timestamp.elapsed().unwrap_or_default();
        let decay_factor = (-decay_rate * elapsed.as_secs_f64()).exp();
        Self {
            value: self.value * decay_factor,
            source: self.source.clone(),
            timestamp: std::time::SystemTime::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tensor {
    data: Array<f64, ndarray::IxDyn>,
    confidence: Confidence,
}

impl Tensor {
    pub fn new(data: Array<f64, ndarray::IxDyn>, confidence: Confidence) -> Self {
        Self { data, confidence }
    }

    pub fn shape(&self) -> &[usize] {
        self.data.shape()
    }

    pub fn confidence(&self) -> &Confidence {
        &self.confidence
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Context {
    name: String,
    vocabulary: Vec<String>,
    confidence_threshold: f64,
    validation_sources: Vec<String>,
}

impl Context {
    pub fn new(
        name: String,
        vocabulary: Vec<String>,
        confidence_threshold: f64,
        validation_sources: Vec<String>,
    ) -> Result<Self, String> {
        if confidence_threshold < 0.0 || confidence_threshold > 1.0 {
            return Err("Confidence threshold must be between 0.0 and 1.0".to_string());
        }
        Ok(Self {
            name,
            vocabulary,
            confidence_threshold,
            validation_sources,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn confidence_threshold(&self) -> f64 {
        self.confidence_threshold
    }

    pub fn validate_confidence(&self, confidence: &Confidence) -> bool {
        confidence.value() >= self.confidence_threshold
    }

    pub fn vocabulary(&self) -> &[String] {
        &self.vocabulary
    }

    pub fn validation_sources(&self) -> &[String] {
        &self.validation_sources
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.value,
            self.source
                .as_ref()
                .map(|s| format!(" (source: {})", s))
                .unwrap_or_default()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_creation() {
        let conf = Confidence::new(0.8).unwrap();
        assert_eq!(conf.value(), 0.8);
        
        let result = Confidence::new(1.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_confidence_decay() {
        let conf = Confidence::new(1.0).unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let decayed = conf.decay(0.5);
        assert!(decayed.value() < 1.0);
    }

    #[test]
    fn test_context_validation() {
        let context = Context::new(
            "medical".to_string(),
            vec!["symptom".to_string()],
            0.7,
            vec!["pubmed".to_string()],
        )
        .unwrap();

        let high_conf = Confidence::new(0.8).unwrap();
        let low_conf = Confidence::new(0.6).unwrap();

        assert!(context.validate_confidence(&high_conf));
        assert!(!context.validate_confidence(&low_conf));
    }
} 