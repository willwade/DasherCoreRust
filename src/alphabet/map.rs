use std::collections::HashMap;
use crate::alphabet::info::AlphabetInfo;

/// Maps between characters and their symbol indices
#[derive(Debug, Clone)]
pub struct AlphabetMap {
    /// Map from characters to symbol indices
    char_to_symbol: HashMap<char, usize>,
    /// Map from strings to symbol indices
    string_to_symbol: HashMap<String, usize>,
    /// Reference to alphabet information
    alphabet_info: AlphabetInfo,
}

impl AlphabetMap {
    /// Create a new alphabet map
    pub fn new(alphabet_info: AlphabetInfo) -> Self {
        let mut map = Self {
            char_to_symbol: HashMap::new(),
            string_to_symbol: HashMap::new(),
            alphabet_info,
        };
        map.build_maps();
        map
    }
}

impl Default for AlphabetMap {
    fn default() -> Self {
        Self::new(AlphabetInfo::default())
    }
}

impl AlphabetMap {
    /// Build the character and string maps
    fn build_maps(&mut self) {
        for (i, character) in self.alphabet_info.characters.iter().enumerate() {
            let symbol = i + 1; // Symbols are 1-based
            
            // Add single character mapping if the text is a single char
            if let Some(ch) = character.text.chars().next() {
                if character.text.chars().count() == 1 {
                    self.char_to_symbol.insert(ch, symbol);
                }
            }

            // Add string mapping
            self.string_to_symbol.insert(character.text.clone(), symbol);
        }
    }

    /// Get the symbol index for a character
    pub fn char_to_index(&self, ch: char) -> Option<usize> {
        self.char_to_symbol.get(&ch).copied()
    }

    /// Get the symbol index for a string
    pub fn string_to_index(&self, s: &str) -> Option<usize> {
        self.string_to_symbol.get(s).copied()
    }

    /// Get the display text for a symbol
    pub fn get_display_text(&self, symbol: usize) -> Option<&str> {
        self.alphabet_info.get_display_text(symbol)
    }

    /// Get the text for a symbol
    pub fn get_text(&self, symbol: usize) -> Option<&str> {
        self.alphabet_info.get_text(symbol)
    }

    /// Check if a symbol is a space character
    pub fn is_space(&self, symbol: usize) -> bool {
        self.alphabet_info.is_space_character(symbol)
    }

    /// Check if a symbol prints a newline
    pub fn is_newline(&self, symbol: usize) -> bool {
        self.alphabet_info.prints_newline(symbol)
    }

    /// Get the number of symbols in the alphabet
    pub fn len(&self) -> usize {
        self.alphabet_info.characters.len()
    }

    /// Check if the alphabet is empty
    pub fn is_empty(&self) -> bool {
        self.alphabet_info.characters.is_empty()
    }

    /// Get the alphabet information
    pub fn alphabet_info(&self) -> &AlphabetInfo {
        &self.alphabet_info
    }

    /// Convert a string to a sequence of symbols
    pub fn text_to_symbols(&self, text: &str) -> Vec<usize> {
        let mut symbols = Vec::new();
        let mut current_text = String::new();

        for ch in text.chars() {
            current_text.push(ch);
            
            // Try to match the current text
            if let Some(symbol) = self.string_to_symbol.get(&current_text) {
                symbols.push(*symbol);
                current_text.clear();
            } else {
                // If no match and we have accumulated text, try character by character
                if current_text.len() > 1 {
                    for prev_ch in current_text[..current_text.len()-1].chars() {
                        if let Some(symbol) = self.char_to_symbol.get(&prev_ch) {
                            symbols.push(*symbol);
                        }
                    }
                    current_text = ch.to_string();
                }
            }
        }

        // Handle any remaining text
        if !current_text.is_empty() {
            if let Some(symbol) = self.string_to_symbol.get(&current_text) {
                symbols.push(*symbol);
            } else {
                for ch in current_text.chars() {
                    if let Some(symbol) = self.char_to_symbol.get(&ch) {
                        symbols.push(*symbol);
                    }
                }
            }
        }

        symbols
    }

    /// Convert symbols to text
    pub fn symbols_to_text(&self, symbols: &[usize]) -> String {
        let mut text = String::new();
        for &symbol in symbols {
            if let Some(s) = self.get_text(symbol) {
                text.push_str(s);
            }
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alphabet::info::Character;

    fn create_test_alphabet() -> AlphabetInfo {
        let mut info = AlphabetInfo::new("test".to_string());
        
        // Add basic characters
        info.characters.extend_from_slice(&[
            Character {
                display: "a".to_string(),
                text: "a".to_string(),
                ..Default::default()
            },
            Character {
                display: "b".to_string(),
                text: "b".to_string(),
                ..Default::default()
            },
            Character {
                display: "_".to_string(),
                text: " ".to_string(),
                ..Default::default()
            },
            Character {
                display: "ch".to_string(),
                text: "ch".to_string(),
                ..Default::default()
            },
        ]);

        info
    }

    #[test]
    fn test_alphabet_map() {
        let info = create_test_alphabet();
        let map = AlphabetMap::new(info);

        assert_eq!(map.char_to_index('a'), Some(1));
        assert_eq!(map.string_to_index("ch"), Some(4));
        assert!(map.is_space(3));
        assert_eq!(map.text_to_symbols("a ch b"), vec![1, 3, 4, 3, 2]);
        assert_eq!(map.symbols_to_text(&[1, 3, 4, 3, 2]), "a ch b");
    }
}
