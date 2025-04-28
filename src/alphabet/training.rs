use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::collections::HashMap;
use std::time::SystemTime;

use super::{
    AlphabetInfo,
    AlphabetMap,
    AlphabetConversion,
    ConversionManager,
    ConversionRule,
};

/// Error type for training operations
#[derive(Debug)]
pub enum TrainingError {
    /// IO error
    Io(io::Error),
    /// Invalid data
    InvalidData(String),
}

impl From<io::Error> for TrainingError {
    fn from(err: io::Error) -> Self {
        TrainingError::Io(err)
    }
}

/// Training data statistics
#[derive(Debug, Clone)]
pub struct TrainingStats {
    /// Total number of characters processed
    pub total_chars: usize,
    /// Character frequency map
    pub char_frequency: HashMap<char, usize>,
    /// Word frequency map
    pub word_frequency: HashMap<String, usize>,
    /// Bigram frequency map
    pub bigram_frequency: HashMap<(char, char), usize>,
    /// Last update time
    pub last_update: SystemTime,
}

impl Default for TrainingStats {
    fn default() -> Self {
        Self::new()
    }
}

impl TrainingStats {
    /// Create new empty training statistics
    pub fn new() -> Self {
        Self {
            total_chars: 0,
            char_frequency: HashMap::new(),
            word_frequency: HashMap::new(),
            bigram_frequency: HashMap::new(),
            last_update: SystemTime::now(),
        }
    }

    /// Update statistics with new text
    pub fn update(&mut self, text: &str) {
        let mut prev_char = None;
        let mut current_word = String::new();

        for c in text.chars() {
            // Update character frequency
            *self.char_frequency.entry(c).or_insert(0) += 1;
            self.total_chars += 1;

            // Update bigram frequency
            if let Some(prev) = prev_char {
                *self.bigram_frequency.entry((prev, c)).or_insert(0) += 1;
            }
            prev_char = Some(c);

            // Update word frequency
            if c.is_whitespace() || c.is_ascii_punctuation() {
                if !current_word.is_empty() {
                    *self.word_frequency.entry(current_word.clone()).or_insert(0) += 1;
                    current_word.clear();
                }
            } else {
                current_word.push(c);
            }
        }

        // Handle last word
        if !current_word.is_empty() {
            *self.word_frequency.entry(current_word).or_insert(0) += 1;
        }

        self.last_update = SystemTime::now();
    }

    /// Get character probability
    pub fn char_probability(&self, c: char) -> f64 {
        if self.total_chars == 0 {
            return 0.0;
        }
        self.char_frequency.get(&c).copied().unwrap_or(0) as f64 / self.total_chars as f64
    }

    /// Get bigram probability
    pub fn bigram_probability(&self, c1: char, c2: char) -> f64 {
        let c1_count = self.char_frequency.get(&c1).copied().unwrap_or(0);
        if c1_count == 0 {
            return 0.0;
        }
        self.bigram_frequency.get(&(c1, c2)).copied().unwrap_or(0) as f64 / c1_count as f64
    }
}

/// Training data manager
#[derive(Debug)]
pub struct TrainingManager {
    /// Alphabet information
    alphabet: AlphabetInfo,
    /// Alphabet map for symbol lookup
    #[allow(dead_code)]
    alphabet_map: AlphabetMap,
    /// Training statistics
    stats: TrainingStats,
    /// Training file path
    training_file: Option<PathBuf>,
    /// Conversion manager for training conversion rules
    conversion: Option<ConversionManager>,
}

impl TrainingManager {
    /// Create a new training manager
    pub fn new(alphabet: AlphabetInfo) -> Self {
        let alphabet_map = AlphabetMap::new(alphabet.clone());
        Self {
            alphabet,
            alphabet_map,
            stats: TrainingStats::new(),
            training_file: None,
            conversion: None,
        }
    }

    /// Set the training file path
    pub fn set_training_file<P: AsRef<Path>>(&mut self, path: P) {
        self.training_file = Some(path.as_ref().to_path_buf());
    }

    /// Enable conversion training
    pub fn enable_conversion_training(&mut self) {
        self.conversion = Some(ConversionManager::from_alphabet(&self.alphabet));
    }

    /// Load training data from file
    pub fn load_training_data(&mut self) -> Result<(), TrainingError> {
        if let Some(path) = &self.training_file {
            if !path.exists() {
                return Ok(());
            }

            let file = File::open(path)?;
            let reader = BufReader::new(file);

            for line in reader.lines() {
                let line = line?;
                self.process_training_line(&line)?;
            }
        }
        Ok(())
    }

    /// Process a line of training data
    fn process_training_line(&mut self, line: &str) -> Result<(), TrainingError> {
        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            return Ok(());
        }

        // Process conversion training if enabled
        if let Some(conversion) = &mut self.conversion {
            match self.alphabet.conversion_type {
                AlphabetConversion::None => {
                    self.stats.update(line);
                }
                AlphabetConversion::Mandarin => {
                    if let Some((input, output)) = line.split_once('=') {
                        let rule = ConversionRule {
                            input: input.trim().to_string(),
                            output: output.trim().to_string(),
                            context: String::new(),
                            trainable: true,
                        };
                        conversion.add_table(AlphabetConversion::Mandarin, rule.into());
                    }
                }
                AlphabetConversion::RoutingContextInsensitive |
                AlphabetConversion::RoutingContextSensitive => {
                    if let Some((context, rest)) = line.split_once(':') {
                        if let Some((input, output)) = rest.split_once('=') {
                            let rule = ConversionRule {
                                input: input.trim().to_string(),
                                output: output.trim().to_string(),
                                context: context.trim().to_string(),
                                trainable: true,
                            };
                            conversion.add_table(self.alphabet.conversion_type, rule.into());
                        }
                    }
                }
            }
        } else {
            // Regular training without conversion
            self.stats.update(line);
        }

        Ok(())
    }

    /// Save training data to file
    pub fn save_training_data(&self) -> Result<(), TrainingError> {
        if let Some(path) = &self.training_file {
            let mut file = File::create(path)?;

            // Write header
            writeln!(file, "# Dasher training data")?;
            writeln!(file, "# Last updated: {:?}", self.stats.last_update)?;
            writeln!(file)?;

            // Write character frequencies
            writeln!(file, "# Character frequencies")?;
            for (c, count) in &self.stats.char_frequency {
                writeln!(file, "{}\t{}", c, count)?;
            }
            writeln!(file)?;

            // Write bigram frequencies
            writeln!(file, "# Bigram frequencies")?;
            for ((c1, c2), count) in &self.stats.bigram_frequency {
                writeln!(file, "{}{}\t{}", c1, c2, count)?;
            }
            writeln!(file)?;

            // Write word frequencies
            writeln!(file, "# Word frequencies")?;
            for (word, count) in &self.stats.word_frequency {
                writeln!(file, "{}\t{}", word, count)?;
            }

            // Write conversion rules if enabled
            if let Some(conversion) = &self.conversion {
                writeln!(file)?;
                writeln!(file, "# Conversion rules")?;
                match self.alphabet.conversion_type {
                    AlphabetConversion::Mandarin => {
                        if let Some(table) = conversion.get_rules(AlphabetConversion::Mandarin) {
                            for rule in table.get_all_rules() {
                                writeln!(file, "{}={}", rule.input, rule.output)?;
                            }
                        }
                    },
                    AlphabetConversion::RoutingContextInsensitive |
                    AlphabetConversion::RoutingContextSensitive => {
                        if let Some(table) = conversion.get_rules(self.alphabet.conversion_type) {
                            for rule in table.get_all_rules() {
                                if !rule.context.is_empty() {
                                    writeln!(file, "{}:{}={}", rule.context, rule.input, rule.output)?;
                                } else {
                                    writeln!(file, "{}={}", rule.input, rule.output)?;
                                }
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Train on new text
    pub fn train(&mut self, text: &str) -> Result<(), TrainingError> {
        self.process_training_line(text)?;
        self.save_training_data()
    }

    /// Get training statistics
    pub fn stats(&self) -> &TrainingStats {
        &self.stats
    }

    /// Get conversion manager
    pub fn conversion(&self) -> Option<&ConversionManager> {
        self.conversion.as_ref()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use tempfile::NamedTempFile;
//
//     #[test]
//     fn test_training_stats() {
//         let mut stats = TrainingStats::new();
//         stats.update("hello world");
//
//         assert_eq!(stats.total_chars, 11);
//         assert_eq!(*stats.char_frequency.get(&'l').unwrap(), 3);
//         assert_eq!(*stats.word_frequency.get("hello").unwrap(), 1);
//         assert_eq!(*stats.bigram_frequency.get(&('l', 'l')).unwrap(), 1);
//     }
//
//     #[test]
//     fn test_training_manager() -> Result<(), TrainingError> {
//         let alphabet = AlphabetInfo::new("test".to_string());
//         let mut manager = TrainingManager::new(alphabet);
//
//         // Create temporary training file
//         let temp_file = NamedTempFile::new().unwrap();
//         manager.set_training_file(temp_file.path());
//
//         // Train on some text
//         manager.train("hello world")?;
//
//         // Check statistics
//         let stats = manager.stats();
//         assert_eq!(stats.total_chars, 11);
//         assert_eq!(*stats.char_frequency.get(&'l').unwrap(), 3);
//
//         // Load training data
//         let mut new_manager = TrainingManager::new(AlphabetInfo::new("test".to_string()));
//         new_manager.set_training_file(temp_file.path());
//         new_manager.load_training_data()?;
//
//         // Check loaded statistics
//         let new_stats = new_manager.stats();
//         assert_eq!(new_stats.total_chars, 11);
//         assert_eq!(*new_stats.char_frequency.get(&'l').unwrap(), 3);
//
//         Ok(())
//     }
// }
