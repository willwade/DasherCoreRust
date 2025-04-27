use super::{WordGenerator, WordGeneratorError};
use crate::model::language_model::LanguageModel;

/// A word generator that uses a language model for prediction
pub struct PredictiveWordGenerator<M: LanguageModel> {
    /// Base word generator functionality (for symbol conversion)
    pub base: crate::model::word_generator::BaseWordGenerator,
    /// The language model used for prediction
    language_model: M,
    /// Current context for prediction
    context: Vec<String>,
    /// Maximum number of predictions to generate
    max_predictions: usize,
    /// Current predictions
    current_predictions: Vec<String>,
    /// Current prediction index
    current_index: usize,
}

impl<M: LanguageModel> PredictiveWordGenerator<M> {
    /// Create a new predictive word generator
    pub fn new(language_model: M, max_predictions: usize, base: crate::model::word_generator::BaseWordGenerator) -> Self {
        Self {
            base,
            language_model,
            context: Vec::new(),
            max_predictions,
            current_predictions: Vec::new(),
            current_index: 0,
        }
    }

    /// Update the context for prediction
    pub fn update_context(&mut self, context: Vec<String>) {
        self.context = context;
        self.refresh_predictions();
    }

    /// Add a word to the context
    pub fn add_to_context(&mut self, word: String) {
        self.context.push(word);
        self.refresh_predictions();
    }

    /// Refresh the current predictions based on context
    fn refresh_predictions(&mut self) {
        self.current_predictions = self.language_model
            .predict_next(&self.context, self.max_predictions);
        self.current_index = 0;
    }

    /// Get the probability of the next word
    pub fn get_probability(&self, word: &str) -> f64 {
        self.language_model.get_probability(&self.context, word)
    }
}

impl<M: LanguageModel> WordGenerator for PredictiveWordGenerator<M> {
    fn next_word(&mut self) -> Option<String> {
        if self.current_index >= self.current_predictions.len() {
            None
        } else {
            let word = self.current_predictions[self.current_index].clone();
            self.current_index += 1;
            Some(word)
        }
    }

    fn get_symbols(&self, word: &str) -> Vec<u32> {
        self.base.string_to_symbols(word)
    }
}

/// A word generator that combines multiple word generators
pub struct CompositeWordGenerator {
    pub base: crate::model::word_generator::BaseWordGenerator,
    generators: Vec<Box<dyn WordGenerator>>,
    current_generator: usize,
}

impl CompositeWordGenerator {
    /// Create a new composite word generator
    pub fn new(generators: Vec<Box<dyn WordGenerator>>, base: crate::model::word_generator::BaseWordGenerator) -> Self {
        Self {
            base,
            generators,
            current_generator: 0,
        }
    }
}

impl WordGenerator for CompositeWordGenerator {
    fn next_word(&mut self) -> Option<String> {
        while self.current_generator < self.generators.len() {
            if let Some(word) = self.generators[self.current_generator].next_word() {
                return Some(word);
            }
            self.current_generator += 1;
        }
        None
    }

    fn get_symbols(&self, word: &str) -> Vec<u32> {
        self.base.string_to_symbols(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    // Mock language model for testing
    struct MockLanguageModel {
        predictions: HashMap<String, Vec<String>>,
    }

    impl MockLanguageModel {
        fn new() -> Self {
            let mut predictions = HashMap::new();
            predictions.insert(
                "hello".to_string(),
                vec!["world".to_string(), "there".to_string()],
            );
            Self { predictions }
        }
    }

    impl LanguageModel for MockLanguageModel {
        fn predict_next(&self, context: &[String], max_count: usize) -> Vec<String> {
            if let Some(last_word) = context.last() {
                if let Some(predictions) = self.predictions.get(last_word) {
                    return predictions[..max_count.min(predictions.len())].to_vec();
                }
            }
            Vec::new()
        }

        fn get_probability(&self, _context: &[String], _word: &str) -> f64 {
            1.0
        }
    }

    #[test]
    fn test_predictive_word_generator() {
        let model = MockLanguageModel::new();
        let alphabet_info = crate::alphabet::AlphabetInfo::default();
        let alphabet_map = crate::alphabet::AlphabetMap::default();
        let base = crate::model::word_generator::BaseWordGenerator::new(alphabet_info, alphabet_map);
        let mut generator = PredictiveWordGenerator::new(model, 2, base);

        generator.update_context(vec!["hello".to_string()]);

        assert_eq!(generator.next_word(), Some("world".to_string()));
        assert_eq!(generator.next_word(), Some("there".to_string()));
        assert_eq!(generator.next_word(), None);
    }
}
