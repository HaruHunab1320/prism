use std::collections::HashMap;

pub struct ConfidenceEngine {
    decay_rate: f64,
    current_values: HashMap<String, f64>,
}

impl ConfidenceEngine {
    pub fn new(decay_rate: f64) -> Self {
        Self {
            decay_rate,
            current_values: HashMap::new(),
        }
    }

    pub fn new_with_values(decay_rate: f64, initial_values: HashMap<String, f64>) -> Self {
        Self {
            decay_rate,
            current_values: initial_values,
        }
    }

    pub fn set(&mut self, key: &str, value: f64) {
        if value >= 0.0 && value <= 1.0 {
            self.current_values.insert(key.to_string(), value);
        }
    }

    pub fn get(&self, key: &str) -> Option<f64> {
        self.current_values.get(key).copied()
    }

    pub fn get_decayed(&self, key: &str) -> Option<f64> {
        self.current_values.get(key).map(|&value| {
            value * (1.0 - self.decay_rate)
        })
    }

    pub fn decay_all(&mut self) {
        for value in self.current_values.values_mut() {
            *value *= 1.0 - self.decay_rate;
        }
    }

    pub fn combine(&self, values: &[f64]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut product = 1.0;
        for &value in values {
            product *= value;
        }
        product
    }

    pub fn combine_weighted(&self, values: &[(f64, f64)]) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let mut weighted_sum = 0.0;
        let mut weight_sum = 0.0;

        for &(value, weight) in values {
            weighted_sum += value * weight;
            weight_sum += weight;
        }

        if weight_sum == 0.0 {
            0.0
        } else {
            weighted_sum / weight_sum
        }
    }

    pub fn clear(&mut self) {
        self.current_values.clear();
    }

    pub fn remove(&mut self, key: &str) -> Option<f64> {
        self.current_values.remove(key)
    }

    pub fn keys(&self) -> Vec<String> {
        self.current_values.keys().cloned().collect()
    }

    pub fn values(&self) -> Vec<f64> {
        self.current_values.values().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_set_get() {
        let mut engine = ConfidenceEngine::new(0.1);
        engine.set("test", 0.8);
        assert_eq!(engine.get("test"), Some(0.8));
    }

    #[test]
    fn test_confidence_decay() {
        let mut engine = ConfidenceEngine::new(0.1);
        engine.set("test", 1.0);
        assert_eq!(engine.get_decayed("test"), Some(0.9));
        engine.decay_all();
        assert_eq!(engine.get("test"), Some(0.9));
    }

    #[test]
    fn test_confidence_combine() {
        let engine = ConfidenceEngine::new(0.1);
        let values = vec![0.8, 0.9, 0.7];
        assert!((engine.combine(&values) - 0.504).abs() < f64::EPSILON);
    }

    #[test]
    fn test_confidence_combine_weighted() {
        let engine = ConfidenceEngine::new(0.1);
        let values = vec![(0.8, 2.0), (0.9, 1.0), (0.7, 3.0)];
        let result = engine.combine_weighted(&values);
        let expected = (0.8 * 2.0 + 0.9 * 1.0 + 0.7 * 3.0) / (2.0 + 1.0 + 3.0);
        assert!((result - expected).abs() < f64::EPSILON);
    }

    #[test]
    fn test_confidence_clear_remove() {
        let mut engine = ConfidenceEngine::new(0.1);
        engine.set("test1", 0.8);
        engine.set("test2", 0.9);
        
        assert_eq!(engine.remove("test1"), Some(0.8));
        assert_eq!(engine.get("test1"), None);
        
        engine.clear();
        assert_eq!(engine.get("test2"), None);
    }

    #[test]
    fn test_confidence_keys_values() {
        let mut engine = ConfidenceEngine::new(0.1);
        engine.set("test1", 0.8);
        engine.set("test2", 0.9);
        
        let mut keys = engine.keys();
        keys.sort();
        assert_eq!(keys, vec!["test1", "test2"]);
        
        let mut values = engine.values();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(values, vec![0.8, 0.9]);
    }
} 