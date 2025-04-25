use std::path::Path;
use std::io;

use crate::alphabet::AlphabetMap;
use crate::alphabet::AlphabetInfo;

/// Trait for word generators that can provide words based on various conditions.
/// 
/// Word generators encapsulate logic for generating words based on implementation-specific
/// conditions. This could be from a file, dictionary, or any other source.
pub trait WordGenerator {
    /// Gets the next word from this generator.
    /// Returns None when there are no more words available.
    fn next_word(&mut self) -> Option<String>;

    /// Gets the symbols for the current word.
    /// Returns a vector of symbol indices that correspond to the alphabet map.
    fn get_symbols(&self, word: &str) -> Vec<u32>;
}

/// Error types for word generator operations
#[derive(Debug)]
pub enum WordGeneratorError {
    /// Error occurred while reading from a file
    IoError(io::Error),
    /// Invalid file format or content
    InvalidFormat(String),
    /// Unsupported operation
    Unsupported(String),
}

impl From<io::Error> for WordGeneratorError {
    fn from(error: io::Error) -> Self {
        WordGeneratorError::IoError(error)
    }
}

/// Base implementation for word generators that use an alphabet
pub struct BaseWordGenerator {
    /// The alphabet information
    alphabet_info: AlphabetInfo,
    /// The alphabet map for converting between symbols and indices
    alphabet_map: AlphabetMap,
}

impl BaseWordGenerator {
    /// Create a new base word generator
    pub fn new(alphabet_info: AlphabetInfo, alphabet_map: AlphabetMap) -> Self {
        Self {
            alphabet_info,
            alphabet_map,
        }
    }

    /// Convert a string to symbol indices using the alphabet map
    pub fn string_to_symbols(&self, text: &str) -> Vec<u32> {
        let mut symbols = Vec::new();
        for c in text.chars() {
            if let Some(symbol) = self.alphabet_map.char_to_index(c) {
                symbols.push(symbol);
            }
        }
        symbols
    }
}

/// A word generator that reads words from a file
pub struct FileWordGenerator {
    /// Base word generator functionality
    base: BaseWordGenerator,
    /// Path to the word list file
    path: Box<Path>,
    /// Whether to accept user-added words
    accept_user: bool,
    /// Current line indices in the file
    line_indices: Vec<u64>,
    /// Current position in line_indices
    current_index: usize,
}

impl FileWordGenerator {
    /// Create a new file word generator
    pub fn new(
        alphabet_info: AlphabetInfo,
        alphabet_map: AlphabetMap,
        path: impl AsRef<Path>,
        accept_user: bool,
    ) -> Result<Self, WordGeneratorError> {
        // Verify the file exists and is readable
        let path = path.as_ref().to_owned().into_boxed_path();
        if !path.exists() {
            return Err(WordGeneratorError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                "Word list file not found",
            )));
        }

        let base = BaseWordGenerator::new(alphabet_info, alphabet_map);

        let mut generator = Self {
            base,
            path,
            accept_user,
            line_indices: Vec::new(),
            current_index: 0,
        };

        // Index the file
        generator.index_file()?;

        Ok(generator)
    }

    /// Index the file to get line positions
    fn index_file(&mut self) -> Result<(), WordGeneratorError> {
        use std::io::{BufRead, BufReader};
        use std::fs::File;

        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);

        self.line_indices.clear();
        self.current_index = 0;

        for (index, line) in reader.lines().enumerate() {
            match line {
                Ok(_) => self.line_indices.push(index as u64),
                Err(e) => return Err(WordGeneratorError::IoError(e)),
            }
        }

        Ok(())
    }

    /// Check if there are more lines available
    pub fn has_lines(&self) -> bool {
        self.current_index < self.line_indices.len()
    }

    /// Set whether to accept user-added words
    pub fn set_accept_user(&mut self, accept: bool) {
        self.accept_user = accept;
    }
}

impl WordGenerator for FileWordGenerator {
    fn next_word(&mut self) -> Option<String> {
        use std::io::{BufRead, BufReader, Seek, SeekFrom};
        use std::fs::File;

        if !self.has_lines() {
            return None;
        }

        let file = File::open(&self.path).ok()?;
        let mut reader = BufReader::new(file);

        // Seek to the current line position
        let pos = self.line_indices[self.current_index];
        reader.seek(SeekFrom::Start(pos)).ok()?;

        // Read the line
        let mut line = String::new();
        reader.read_line(&mut line).ok()?;
        self.current_index += 1;

        // Trim whitespace and return
        Some(line.trim().to_string())
    }

    fn get_symbols(&self, word: &str) -> Vec<u32> {
        self.base.string_to_symbols(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "hello").unwrap();
        writeln!(file, "world").unwrap();
        writeln!(file, "test").unwrap();
        file
    }

    #[test]
    fn test_file_word_generator() {
        let file = create_test_file();
        let alphabet_info = AlphabetInfo::default(); // You'll need to implement this
        let alphabet_map = AlphabetMap::default();   // You'll need to implement this

        let mut generator = FileWordGenerator::new(
            alphabet_info,
            alphabet_map,
            file.path(),
            true,
        ).unwrap();

        assert_eq!(generator.next_word(), Some("hello".to_string()));
        assert_eq!(generator.next_word(), Some("world".to_string()));
        assert_eq!(generator.next_word(), Some("test".to_string()));
        assert_eq!(generator.next_word(), None);
    }
}
