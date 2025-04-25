use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

/// Dictionary entry with frequency information
#[derive(Debug, Clone)]
pub struct DictionaryEntry {
    /// Word text
    pub text: String,
    /// Word frequency (0-1)
    pub frequency: f64,
    /// Whether this is a user-added word
    pub user_added: bool,
}

/// Dictionary for word prediction
#[derive(Debug)]
pub struct Dictionary {
    /// Word entries
    entries: HashMap<String, DictionaryEntry>,
    /// Prefix cache for quick lookup
    prefix_cache: HashMap<String, HashSet<String>>,
    /// Maximum prefix length to cache
    max_prefix_length: usize,
}

impl Dictionary {
    /// Create a new empty dictionary
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            prefix_cache: HashMap::new(),
            max_prefix_length: 4,
        }
    }

    /// Load dictionary from file
    pub fn load<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse line: word\tfrequency
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() != 2 {
                continue;
            }

            let word = parts[0].trim();
            if let Ok(frequency) = parts[1].trim().parse::<f64>() {
                self.add_word(word, frequency, false);
            }
        }

        Ok(())
    }

    /// Add word to dictionary
    pub fn add_word(&mut self, word: &str, frequency: f64, user_added: bool) {
        let entry = DictionaryEntry {
            text: word.to_string(),
            frequency,
            user_added,
        };
        self.entries.insert(word.to_string(), entry);

        // Update prefix cache
        for i in 1..=word.len().min(self.max_prefix_length) {
            let prefix = &word[..i];
            self.prefix_cache
                .entry(prefix.to_string())
                .or_insert_with(HashSet::new)
                .insert(word.to_string());
        }
    }

    /// Remove word from dictionary
    pub fn remove_word(&mut self, word: &str) {
        if self.entries.remove(word).is_some() {
            // Update prefix cache
            for i in 1..=word.len().min(self.max_prefix_length) {
                let prefix = &word[..i];
                if let Some(words) = self.prefix_cache.get_mut(prefix) {
                    words.remove(word);
                    if words.is_empty() {
                        self.prefix_cache.remove(prefix);
                    }
                }
            }
        }
    }

    /// Get word entry
    pub fn get_word(&self, word: &str) -> Option<&DictionaryEntry> {
        self.entries.get(word)
    }

    /// Find words with given prefix
    pub fn find_words_with_prefix(&self, prefix: &str) -> Vec<&DictionaryEntry> {
        let mut results = Vec::new();

        // Check prefix cache first
        if prefix.len() <= self.max_prefix_length {
            if let Some(cached) = self.prefix_cache.get(prefix) {
                results.extend(cached.iter().filter_map(|word| self.entries.get(word)));
                return results;
            }
        }

        // Fall back to linear search
        for entry in self.entries.values() {
            if entry.text.starts_with(prefix) {
                results.push(entry);
            }
        }

        // Sort by frequency
        results.sort_by(|a, b| b.frequency.partial_cmp(&a.frequency).unwrap());
        results
    }

    /// Get total word count
    pub fn word_count(&self) -> usize {
        self.entries.len()
    }

    /// Clear dictionary
    pub fn clear(&mut self) {
        self.entries.clear();
        self.prefix_cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_dictionary_basic() {
        let mut dict = Dictionary::new();

        // Add words
        dict.add_word("hello", 0.5, false);
        dict.add_word("help", 0.3, false);
        dict.add_word("world", 0.4, false);

        // Test word lookup
        assert!(dict.get_word("hello").is_some());
        assert_eq!(dict.get_word("hello").unwrap().frequency, 0.5);

        // Test prefix search
        let results = dict.find_words_with_prefix("hel");
        assert_eq!(results.len(), 2);
        assert!(results.iter().any(|e| e.text == "hello"));
        assert!(results.iter().any(|e| e.text == "help"));
    }

    #[test]
    fn test_dictionary_file() -> io::Result<()> {
        // Create temporary dictionary file
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "hello\t0.5")?;
        writeln!(temp_file, "help\t0.3")?;
        writeln!(temp_file, "world\t0.4")?;

        // Load dictionary
        let mut dict = Dictionary::new();
        dict.load(temp_file.path())?;

        assert_eq!(dict.word_count(), 3);
        assert!(dict.get_word("hello").is_some());
        assert!(dict.get_word("help").is_some());
        assert!(dict.get_word("world").is_some());

        Ok(())
    }

    #[test]
    fn test_dictionary_remove() {
        let mut dict = Dictionary::new();

        // Add and remove words
        dict.add_word("hello", 0.5, false);
        dict.add_word("help", 0.3, false);
        assert_eq!(dict.word_count(), 2);

        dict.remove_word("hello");
        assert_eq!(dict.word_count(), 1);
        assert!(dict.get_word("hello").is_none());
        assert!(dict.get_word("help").is_some());

        // Test prefix cache update
        let results = dict.find_words_with_prefix("hel");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].text, "help");
    }
}
