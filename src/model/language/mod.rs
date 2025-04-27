mod ppm;
mod dictionary;

pub use ppm::{PPMLanguageModel, PPMOrder, PPMNode};
pub use dictionary::Dictionary;
use std::collections::{HashMap, HashSet};

/// Language model trait
pub trait LanguageModel {
    /// Create an empty context (stub)
    fn create_empty_context(&mut self) -> String { String::new() }
    /// Get probability distribution for next symbol
    fn get_probs(&self, context: &str) -> HashMap<char, f64>;

    /// Enter symbol into model
    fn enter_symbol(&mut self, symbol: char);

    /// Reset model state
    fn reset(&mut self);

    /// For downcasting
    fn as_any(&mut self) -> &mut dyn std::any::Any;
}

/// Combined language model using PPM and dictionary
pub struct CombinedLanguageModel {
    /// PPM model
    ppm: PPMLanguageModel,
    /// Dictionary
    dictionary: Dictionary,
    /// Weight for PPM model (0-1)
    ppm_weight: f64,
    /// Current word buffer
    current_word: String,
    /// Rolling context buffer for PPM
    context_buffer: String,
    /// Word separator characters
    word_separators: HashSet<char>,
}

impl CombinedLanguageModel {
    /// Create a new combined language model
    pub fn new(max_order: PPMOrder) -> Self {
        let mut word_separators = HashSet::new();
        word_separators.insert(' ');
        word_separators.insert('\t');
        word_separators.insert('\n');
        word_separators.insert('.');
        word_separators.insert(',');
        word_separators.insert('!');
        word_separators.insert('?');

        Self {
            ppm: PPMLanguageModel::new(max_order),
            dictionary: Dictionary::new(),
            ppm_weight: 0.7,
            current_word: String::new(),
            context_buffer: String::new(),
            word_separators,
        }
    }

    /// Set PPM weight
    #[allow(dead_code)]
    pub fn set_ppm_weight(&mut self, weight: f64) {
        self.ppm_weight = weight.clamp(0.0, 1.0);
    }

    /// Add word separator
    #[allow(dead_code)]
    pub fn add_word_separator(&mut self, separator: char) {
        self.word_separators.insert(separator);
    }

    /// Get dictionary reference
    #[allow(dead_code)]
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    /// Get dictionary mutable reference
    pub fn dictionary_mut(&mut self) -> &mut Dictionary {
        &mut self.dictionary
    }

    /// Get PPM model reference
    #[allow(dead_code)]
    pub fn ppm(&self) -> &PPMLanguageModel {
        &self.ppm
    }

    /// Get PPM model mutable reference
    #[allow(dead_code)]
    pub fn ppm_mut(&mut self) -> &mut PPMLanguageModel {
        &mut self.ppm
    }
}

impl LanguageModel for CombinedLanguageModel {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn get_probs(&self, context: &str) -> HashMap<char, f64> {
        // Get PPM probabilities
        let mut probs = self.ppm.get_probs(context);

        // Get dictionary predictions if we're building a word
        if !self.current_word.is_empty() {
            let dict_words = self.dictionary.find_words_with_prefix(&self.current_word);
            let dict_weight = 1.0 - self.ppm_weight;

            for entry in dict_words {
                if self.current_word.len() < entry.text.len() {
                    let next_char = entry.text.chars().nth(self.current_word.len()).unwrap();
                    let prob = entry.frequency * dict_weight;
                    *probs.entry(next_char).or_insert(0.0) += prob;
                }
            }
        }

        // Normalize probabilities
        let total: f64 = probs.values().sum();
        if total > 0.0 {
            for prob in probs.values_mut() {
                *prob /= total;
            }
        }

        probs
    }

    fn enter_symbol(&mut self, symbol: char) {
        // Update PPM model with context buffer
        self.ppm.enter_symbol(&self.context_buffer, symbol);
        // Update context buffer
        let max_order = self.ppm.max_order().value() as usize;
        self.context_buffer.push(symbol);
        if self.context_buffer.len() > max_order {
            self.context_buffer.remove(0);
        }
        // Update word buffer
        if self.word_separators.contains(&symbol) {
            self.current_word.clear();
            self.context_buffer.clear(); // Reset context at word boundary
        } else {
            self.current_word.push(symbol);
        }
    }

    fn reset(&mut self) {
        self.current_word.clear();
        self.context_buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combined_model() {
        let mut model = CombinedLanguageModel::new(PPMOrder::Two);

        // Add dictionary words
        model.dictionary_mut().add_word("hello", 0.5, false);
        model.dictionary_mut().add_word("help", 0.3, false);

        // Train PPM
        for c in "hello world".chars() {
            model.enter_symbol(c);
        }

        // Test predictions
        model.current_word = "hel".to_string();
        let probs = model.get_probs("hel");
        assert!(probs.contains_key(&'l'));
        assert!(probs.contains_key(&'p'));

        // Test word separation
        model.enter_symbol(' ');
        assert_eq!(model.current_word, "");
    }
}
