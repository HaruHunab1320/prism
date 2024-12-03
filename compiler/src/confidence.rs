use crate::types::Confidence;
use std::collections::HashMap;

pub struct ConfidenceEngine {
    values: HashMap<String, Confidence>,
    decay_rate: f64,
}

impl ConfidenceEngine {
    pub fn new(decay_rate: f64) -> Self {
        Self {
            values: HashMap::new(),
            decay_rate,
        }
    }

    pub fn set(&mut self, name: &str, confidence: Confidence) {
        self.values.insert(name.to_string(), confidence);
    }

    pub fn get(&self, name: &str) -> Option<&Confidence> {
        self.values.get(name)
    }

    pub fn get_decayed(&self, name: &str) -> Option<Confidence> {
        self.values.get(name).map(|c| c.decay(self.decay_rate))
    }

    pub fn combine(&self, conf1: &Confidence, conf2: &Confidence) -> Confidence {
        Confidence::new(conf1.value() * conf2.value()).unwrap_or_else(|_| {
            // This should never happen as multiplication of values between 0 and 1
            // will always result in a value between 0 and 1
            Confidence::new(0.0).unwrap()
        })
    }

    pub fn max<'a>(&self, conf1: &'a Confidence, conf2: &'a Confidence) -> &'a Confidence {
        if conf1.value() >= conf2.value() {
            conf1
        } else {
            conf2
        }
    }

    pub fn flow(&self, source: &Confidence, flow_factor: f64) -> Result<Confidence, String> {
        let new_value = source.value() * flow_factor;
        Confidence::new(new_value)
    }

    pub fn reverse_flow(&self, target: &Confidence, flow_factor: f64) -> Result<Confidence, String> {
        let new_value = target.value() / flow_factor;
        Confidence::new(new_value)
    }
}

pub struct ConfidenceBuilder {
    value: f64,
    source: Option<String>,
}

impl ConfidenceBuilder {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            source: None,
        }
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = Some(source);
        self
    }

    pub fn build(self) -> Result<Confidence, String> {
        match self.source {
            Some(source) => Confidence::with_source(self.value, source),
            None => Confidence::new(self.value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_engine() {
        let mut engine = ConfidenceEngine::new(0.1);
        
        let conf = Confidence::new(0.8).unwrap();
        engine.set("test", conf);
        
        assert_eq!(engine.get("test").unwrap().value(), 0.8);
        
        std::thread::sleep(std::time::Duration::from_secs(1));
        let decayed = engine.get_decayed("test").unwrap();
        assert!(decayed.value() < 0.8);
    }

    #[test]
    fn test_confidence_combination() {
        let engine = ConfidenceEngine::new(0.1);
        
        let conf1 = Confidence::new(0.8).unwrap();
        let conf2 = Confidence::new(0.5).unwrap();
        
        let combined = engine.combine(&conf1, &conf2);
        assert_eq!(combined.value(), 0.4);
    }

    #[test]
    fn test_confidence_flow() {
        let engine = ConfidenceEngine::new(0.1);
        
        let source = Confidence::new(0.8).unwrap();
        let flowed = engine.flow(&source, 0.9).unwrap();
        
        assert_eq!(flowed.value(), 0.72);
    }

    #[test]
    fn test_confidence_builder() {
        let conf = ConfidenceBuilder::new(0.8)
            .with_source("test".to_string())
            .build()
            .unwrap();
        
        assert_eq!(conf.value(), 0.8);
    }
} 