use std::collections::HashMap;
use crate::model::word_generator::WordGenerator;

use crate::model::language::LanguageModel;

/// Manages word prediction and generation for the Dasher model
pub struct WordPredictionManager {
    /// The word generators available
    generators: Vec<Box<dyn crate::model::word_generator::WordGenerator>>,
    /// Cache of predicted words for each context
    prediction_cache: HashMap<String, Vec<String>>,
    /// Maximum number of predictions to cache
    max_predictions: usize,
    /// Maximum size of context to consider
    max_context_size: usize,
}

impl WordPredictionManager {
    /// Create a new word prediction manager
    pub fn new(max_predictions: usize, max_context_size: usize) -> Self {
        Self {
            generators: Vec::new(),
            prediction_cache: HashMap::new(),
            max_predictions,
            max_context_size,
        }
    }

    /// Add a word generator
    pub fn add_generator(&mut self, generator: Box<dyn WordGenerator>) {
        self.generators.push(generator);
    }

    /// Get predictions for the current context
    pub fn get_predictions(&mut self, context: &str) -> Vec<String> {
        // Use the context as the cache key
        let cache_key = context.to_string();
        
        // Return cached predictions if available
        if let Some(predictions) = self.prediction_cache.get(&cache_key) {
            return predictions.clone();
        }

        // Generate new predictions
        let mut predictions = Vec::new();
        
        // Get predictions from all generators
        for generator in &mut self.generators {
            let words = generator.generate_words(context);
            for word in words {
                if !predictions.contains(&word) {
                    predictions.push(word);
                    if predictions.len() >= self.max_predictions {
                        break;
                    }
                }
            }
        }

        // Cache the predictions
        self.prediction_cache.insert(cache_key, predictions.clone());
        
        predictions
    }

    /// Clear the prediction cache
    pub fn clear_cache(&mut self) {
        self.prediction_cache.clear();
    }

    /// Update the context and get new predictions
    pub fn update_context(&mut self, context: &str) -> Vec<String> {
        // Take only the last max_context_size words
        let context = if context.len() > self.max_context_size {
            &context[context.len() - self.max_context_size..]
        } else {
            context
        };

        self.get_predictions(context)
    }
}

/// Create a word prediction manager with common configurations
pub fn create_default_manager<M: LanguageModel + 'static>(
    _language_model: M,
    max_predictions: usize,
) -> WordPredictionManager {
    let manager = WordPredictionManager::new(max_predictions, 3);

    manager
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::word_generator::{PredictiveWordGenerator, BaseWordGenerator, WordGenerator};
    use crate::alphabet::{AlphabetInfo, AlphabetMap};
    use crate::model::language::LanguageModel;
    use std::collections::HashMap;

    struct MockLanguageModel;
    use std::any::Any;
    impl LanguageModel for MockLanguageModel {
        fn get_probs(&self, _context: &str) -> HashMap<char, f64> {
            let mut map = HashMap::new();
            map.insert('t', 0.7);
            map.insert('w', 0.3);
            map
        }
        fn enter_symbol(&mut self, _symbol: char) {}
        fn reset(&mut self) {}
        fn as_any(&mut self) -> &mut dyn Any { self }
    }

    #[test]
    fn test_word_prediction_manager_with_predictive() {
        let mut manager = WordPredictionManager::new(2, 2);
        let model = Box::new(MockLanguageModel);
        let alphabet_info = AlphabetInfo::default();
        let alphabet_map = AlphabetMap::default();
        let base = BaseWordGenerator::new(alphabet_info, alphabet_map);
        let mut predictor = PredictiveWordGenerator::new(model, 2, base);
        predictor.set_context("");
        manager.add_generator(Box::new(predictor));

        let predictions = manager.get_predictions("");
        assert_eq!(predictions.len(), 2);
        assert!(predictions.contains(&"t".to_string()));
        assert!(predictions.contains(&"w".to_string()));
    }
}
