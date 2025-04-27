use super::WordGenerator;
use crate::model::language::LanguageModel;

/// A word generator that uses a language model to predict words
pub struct PredictiveWordGenerator {
    /// Base word generator functionality (for symbol conversion)
    pub base: crate::model::word_generator::BaseWordGenerator,
    /// The language model used for prediction
    language_model: Box<dyn LanguageModel>,
    /// Maximum number of predictions to return
    max_predictions: usize,
    /// Current context for prediction
    context: String,
    /// Buffer of current predictions
    prediction_buffer: Vec<String>,
    /// Index into the prediction buffer
    buffer_index: usize,
}

impl PredictiveWordGenerator {
    /// Create a new predictive word generator
    pub fn new(language_model: Box<dyn LanguageModel>, max_predictions: usize, base: crate::model::word_generator::BaseWordGenerator) -> Self {
        Self {
            base,
            language_model,
            max_predictions,
            context: String::new(),
            prediction_buffer: Vec::new(),
            buffer_index: 0,
        }
    }

    /// Set the context (e.g., previous text or prefix)
    pub fn set_context(&mut self, context: &str) {
        self.context = context.to_string();
        self.refill_predictions();
    }

    /// Generate and store predictions in the buffer
    fn refill_predictions(&mut self) {
        use std::collections::HashMap;
        // Get probability distribution from the language model
        let probs: HashMap<char, f64> = self.language_model.get_probs(&self.context);
        // Sort by probability descending
        let mut sorted: Vec<(char, f64)> = probs.into_iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        // Take top-N characters and turn them into strings (single-char predictions)
        self.prediction_buffer = sorted.iter().take(self.max_predictions).map(|(c, _)| c.to_string()).collect();
        self.buffer_index = 0;
    }

    /// Get all predictions at once (refreshes buffer)
    pub fn generate_predictions(&mut self, context: &str) -> Vec<String> {
        self.set_context(context);
        self.prediction_buffer.clone()
    }

    /// Get the probability of a specific word given the current context
    pub fn get_probability(&mut self, word: &str) -> f64 {
        let probs = self.language_model.get_probs(&self.context);
        if word.len() == 1 {
            probs.get(&word.chars().next().unwrap()).cloned().unwrap_or(0.0)
        } else {
            // For multi-char words, multiply probabilities (naive approach)
            let mut ctx = self.context.clone();
            let mut prob = 1.0;
            for c in word.chars() {
                let probs = self.language_model.get_probs(&ctx);
                prob *= probs.get(&c).cloned().unwrap_or(1e-9);
                ctx.push(c);
            }
            prob
        }
    }
}

impl WordGenerator for PredictiveWordGenerator {
    fn next_word(&mut self) -> Option<String> {
        if self.buffer_index >= self.prediction_buffer.len() {
            self.refill_predictions();
        }
        if self.prediction_buffer.is_empty() {
            None
        } else {
            let word = self.prediction_buffer[self.buffer_index].clone();
            self.buffer_index += 1;
            Some(word)
        }
    }
    fn get_symbols(&self, word: &str) -> Vec<u32> {
        self.base.string_to_symbols(word)
    }
    fn generate_words(&mut self, context: &str) -> Vec<String> {
        self.generate_predictions(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_predictive_word_generator() {
        // let mut mock_model = Box::new(MockLanguageModel::new());
        // let mut generator = PredictiveWordGenerator::new(mock_model, 5);

        // let context = vec!["hello".to_string()];
        // let predictions = generator.generate_words(&context);
        // assert!(!predictions.is_empty());

        // let prob = generator.get_probability(&context, "world");
        // assert!(prob >= 0.0 && prob <= 1.0);
    }
}
