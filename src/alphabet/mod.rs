//! # Alphabet Module
//!
//! This module contains the implementation of the alphabet used by Dasher.
//! It provides comprehensive support for alphabet management, including:
//! - Alphabet information and configuration
//! - Symbol mapping and conversion
//! - Group management
//! - Color and display handling

mod info;
mod group;
mod map;
mod xml;
mod colors;
mod conversion;
mod discovery;
mod training;
mod color_schemes;

pub use info::{AlphabetInfo, ScreenOrientation, AlphabetConversion, Character};
pub use group::GroupInfo;
pub use map::AlphabetMap;
pub use xml::{AlphabetXmlError, save_alphabet, load_alphabet};
pub use colors::{Color, ColorManager};
pub use conversion::{ConversionManager, ConversionTable, ConversionRule};
pub use discovery::{AlphabetDiscovery, DiscoveryError, DiscoveryResult};
pub use training::{TrainingManager, TrainingStats, TrainingError};

use std::collections::HashMap;

/// A symbol in the alphabet
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// The character represented by this symbol
    pub character: char,
    
    /// The display text for this symbol
    pub display_text: String,
    
    /// The foreground color for this symbol
    pub foreground_color: Color,
    
    /// The background color for this symbol
    pub background_color: Color,
}

impl Symbol {
    /// Create a new symbol
    pub fn new(character: char, display_text: &str, foreground_color: Color, background_color: Color) -> Self {
        Self {
            character,
            display_text: display_text.to_string(),
            foreground_color,
            background_color,
        }
    }
    
    /// Create a new symbol with default colors
    pub fn with_default_colors(character: char, display_text: &str) -> Self {
        Self::new(
            character,
            display_text,
            Color::new(0, 0, 0),     // Black text
            Color::new(255, 255, 255) // White background
        )
    }
}

/// An alphabet for Dasher
pub struct Alphabet {
    /// The symbols in this alphabet
    symbols: Vec<Symbol>,
    
    /// Map from character to symbol index
    char_to_index: HashMap<char, usize>,
    
    /// The name of this alphabet
    name: String,
}

impl Alphabet {
    /// Create a conversion manager for this alphabet
    pub fn create_conversion_manager(&self, info: &AlphabetInfo) -> ConversionManager {
        ConversionManager::from_alphabet(info)
    }
    /// Create an alphabet from alphabet info
    pub fn from_info(info: AlphabetInfo) -> Self {
        let mut alphabet = Self::new(&info.id);
        
        for character in &info.characters {
            let symbol = Symbol::new(
                character.text.chars().next().unwrap_or(' '),
                &character.display,
                Color::new(0, 0, 0), // Default colors, should be from color group
                Color::new(255, 255, 255)
            );
            alphabet.add_symbol(symbol);
        }
        
        alphabet
    }
    /// Create a new empty alphabet
    pub fn new(name: &str) -> Self {
        Self {
            symbols: Vec::new(),
            char_to_index: HashMap::new(),
            name: name.to_string(),
        }
    }
    
    /// Create a new English alphabet
    pub fn english() -> Self {
        let mut alphabet = Self::new("English");
        
        // Add letters
        for c in 'a'..='z' {
            alphabet.add_symbol(Symbol::with_default_colors(c, &c.to_string()));
        }
        
        // Add space
        alphabet.add_symbol(Symbol::with_default_colors(' ', "_"));
        
        // Add punctuation
        alphabet.add_symbol(Symbol::with_default_colors('.', "."));
        alphabet.add_symbol(Symbol::with_default_colors(',', ","));
        alphabet.add_symbol(Symbol::with_default_colors('!', "!"));
        alphabet.add_symbol(Symbol::with_default_colors('?', "?"));
        alphabet.add_symbol(Symbol::with_default_colors('\'', "'"));
        alphabet.add_symbol(Symbol::with_default_colors('\"', "\""));
        
        alphabet
    }
    
    /// Add a symbol to the alphabet
    pub fn add_symbol(&mut self, symbol: Symbol) {
        let index = self.symbols.len();
        self.char_to_index.insert(symbol.character, index);
        self.symbols.push(symbol);
    }
    
    /// Get a symbol by index
    pub fn get_symbol(&self, index: usize) -> Option<&Symbol> {
        self.symbols.get(index)
    }
    
    /// Get a symbol by character
    pub fn get_symbol_by_char(&self, c: char) -> Option<&Symbol> {
        if let Some(&index) = self.char_to_index.get(&c) {
            self.get_symbol(index)
        } else {
            None
        }
    }
    
    /// Get the index of a character
    pub fn get_index(&self, c: char) -> Option<usize> {
        self.char_to_index.get(&c).copied()
    }
    
    /// Get the number of symbols in the alphabet
    pub fn size(&self) -> usize {
        self.symbols.len()
    }
    
    /// Get the name of the alphabet
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get all symbols
    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }
}
