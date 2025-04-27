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
}

impl PredictiveWordGenerator {
    /// Create a new predictive word generator
    pub fn new(language_model: Box<dyn LanguageModel>, max_predictions: usize, base: crate::model::word_generator::BaseWordGenerator) -> Self {
        Self {
            base,
            language_model,
            max_predictions,
        }
    }
}

impl WordGenerator for PredictiveWordGenerator {
    fn next_word(&mut self) -> Option<String> {
        // Not implemented for predictive generator
        None
    }

    fn get_symbols(&self, word: &str) -> Vec<u32> {
        self.base.string_to_symbols(word)
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
