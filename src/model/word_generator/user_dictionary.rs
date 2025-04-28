//! User Dictionary Word Generator

use crate::alphabet::{AlphabetInfo, AlphabetMap};
use super::{WordGenerator, WordGeneratorError, BaseWordGenerator};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct UserDictionaryWordGenerator {
    base: BaseWordGenerator,
    dict_path: PathBuf,
    words: Vec<String>,
    current_index: usize,
}

impl UserDictionaryWordGenerator {
    pub fn new(alphabet_info: AlphabetInfo, alphabet_map: AlphabetMap, dict_path: PathBuf) -> Self {
        let mut gen = Self {
            base: BaseWordGenerator::new(alphabet_info, alphabet_map),
            dict_path,
            words: Vec::new(),
            current_index: 0,
        };
        gen.load_words();
        gen
    }

    fn load_words(&mut self) {
        self.words.clear();
        if let Ok(file) = File::open(&self.dict_path) {
            let reader = BufReader::new(file);
            for word in reader.lines().flatten() {
                if !word.trim().is_empty() {
                    self.words.push(word.trim().to_string());
                }
            }
        }
    }

    pub fn add_word(&mut self, word: &str) -> Result<(), WordGeneratorError> {
        let word = word.trim();
        if word.is_empty() { return Ok(()); }
        if !self.words.contains(&word.to_string()) {
            self.words.push(word.to_string());
            let mut file = OpenOptions::new().create(true).append(true).open(&self.dict_path)?;
            writeln!(file, "{}", word)?;
        }
        Ok(())
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
    }
}

impl WordGenerator for UserDictionaryWordGenerator {
    fn next_word(&mut self) -> Option<String> {
        if self.current_index < self.words.len() {
            let word = self.words[self.current_index].clone();
            self.current_index += 1;
            Some(word)
        } else {
            None
        }
    }

    fn get_symbols(&self, word: &str) -> Vec<u32> {
        self.base.string_to_symbols(word)
    }

    fn generate_words(&mut self, context: &str) -> Vec<String> {
        // Simple context filter: return words starting with context
        if context.is_empty() {
            self.words.clone()
        } else {
            self.words.iter().filter(|w| w.starts_with(context)).cloned().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::{AlphabetInfo, AlphabetMap};

    use tempfile::NamedTempFile;

    #[test]
    fn test_user_dictionary_basic() {
        let file = NamedTempFile::new().unwrap();
        let alphabet_info = AlphabetInfo::default();
        let alphabet_map = AlphabetMap::default();
        let mut gen = UserDictionaryWordGenerator::new(alphabet_info, alphabet_map, file.path().to_path_buf());
        assert_eq!(gen.generate_words(""), Vec::<String>::new());
        gen.add_word("hello").unwrap();
        gen.add_word("world").unwrap();
        gen.add_word("help").unwrap();
        assert!(gen.generate_words("").contains(&"hello".to_string()));
        assert!(gen.generate_words("").contains(&"world".to_string()));
        assert_eq!(gen.generate_words("hel"), vec!["hello".to_string(), "help".to_string()]);
    }
}
