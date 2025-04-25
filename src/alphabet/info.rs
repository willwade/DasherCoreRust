use std::collections::HashMap;
use crate::alphabet::group::GroupInfo;

/// Screen orientation for the alphabet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenOrientation {
    /// Left to right orientation
    LeftToRight,
    /// Right to left orientation
    RightToLeft,
    /// Bottom to top orientation
    BottomToTop,
    /// Top to bottom orientation
    TopToBottom,
}

/// Alphabet conversion type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphabetConversion {
    /// Normal alphabet, contains symbols to output
    None,
    /// Mandarin: symbols are phonemes that map to output symbols
    Mandarin,
    /// Context-insensitive routing
    RoutingContextInsensitive,
    /// Context-sensitive routing
    RoutingContextSensitive,
}

/// Character information in the alphabet
#[derive(Debug, Clone)]
pub struct Character {
    /// Display text (rendered on canvas)
    pub display: String,
    /// Output text (written/read)
    pub text: String,
    /// Parent group information
    pub parent_group: Option<GroupInfo>,
    /// Color group offset within parent group
    pub color_group_offset: i32,
    /// Fixed probability for the character (-1 if not fixed)
    pub fixed_probability: f32,
    /// Speed factor for the character (-1 if not modified)
    pub speed_factor: f32,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            display: String::new(),
            text: String::new(),
            parent_group: None,
            color_group_offset: -1,
            fixed_probability: -1.0,
            speed_factor: -1.0,
        }
    }
}

/// Complete description of an alphabet
#[derive(Debug, Clone)]
pub struct AlphabetInfo {
    /// Unique identifier for the alphabet
    pub id: String,
    /// Path to training file
    pub training_file: String,
    /// Preferred color palette
    pub preferred_colors: String,
    /// Screen orientation
    pub orientation: ScreenOrientation,
    /// Default context
    pub default_context: String,
    /// Context escape character
    pub context_escape_char: String,
    /// Conversion type
    pub conversion_type: AlphabetConversion,
    /// Conversion train start marker
    pub conversion_train_start: String,
    /// Conversion train stop marker
    pub conversion_train_stop: String,
    /// Characters in the alphabet
    pub characters: Vec<Character>,
    /// Character actions (do)
    pub character_do_actions: Vec<Vec<String>>,
    /// Character actions (undo)
    pub character_undo_actions: Vec<Vec<String>>,
}

impl AlphabetInfo {
    /// Create a new alphabet information structure
    pub fn new(id: String) -> Self {
        Self {
            id,
            training_file: String::new(),
            preferred_colors: String::new(),
            orientation: ScreenOrientation::LeftToRight,
            default_context: String::new(),
            context_escape_char: String::from("§"),
            conversion_type: AlphabetConversion::None,
            conversion_train_start: String::from("<"),
            conversion_train_stop: String::from(">"),
            characters: Vec::new(),
            character_do_actions: Vec::new(),
            character_undo_actions: Vec::new(),
        }
    }

    /// Get the display text for a symbol
    pub fn get_display_text(&self, symbol: usize) -> Option<&str> {
        self.characters.get(symbol - 1).map(|c| c.display.as_str())
    }

    /// Get the text for a symbol
    pub fn get_text(&self, symbol: usize) -> Option<&str> {
        self.characters.get(symbol - 1).map(|c| c.text.as_str())
    }

    /// Check if a symbol is a space character
    pub fn is_space_character(&self, symbol: usize) -> bool {
        self.characters
            .get(symbol - 1)
            .map(|c| c.text.chars().next().map(char::is_whitespace).unwrap_or(false))
            .unwrap_or(false)
    }

    /// Check if a symbol prints a newline character
    pub fn prints_newline(&self, symbol: usize) -> bool {
        self.characters
            .get(symbol - 1)
            .map(|c| c.text == "\n")
            .unwrap_or(false)
    }

    /// Get the fixed probability for a symbol
    pub fn get_fixed_probability(&self, symbol: usize) -> f32 {
        self.characters
            .get(symbol - 1)
            .map(|c| c.fixed_probability)
            .unwrap_or(-1.0)
    }

    /// Get the speed multiplier for a symbol
    pub fn get_speed_multiplier(&self, symbol: usize) -> f32 {
        self.characters
            .get(symbol - 1)
            .map(|c| c.speed_factor)
            .unwrap_or(-1.0)
    }

    /// Get the color group offset for a symbol
    pub fn get_color_group_offset(&self, symbol: usize) -> i32 {
        self.characters
            .get(symbol - 1)
            .map(|c| c.color_group_offset)
            .unwrap_or(-1)
    }

    /// Get the color group for a symbol
    pub fn get_color_group(&self, symbol: usize) -> Option<&str> {
        self.characters
            .get(symbol - 1)
            .and_then(|c| c.parent_group.as_ref())
            .map(|g| g.color_group.as_str())
    }

    /// Escape a character for training files
    pub fn escape(&self, ch: &str) -> String {
        if ch == self.context_escape_char {
            format!("{}{}", ch, ch)
        } else {
            ch.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alphabet_info() {
        let mut info = AlphabetInfo::new("test".to_string());
        
        // Add a space character
        info.characters.push(Character {
            display: "_".to_string(),
            text: " ".to_string(),
            ..Default::default()
        });

        // Add a normal character
        info.characters.push(Character {
            display: "a".to_string(),
            text: "a".to_string(),
            ..Default::default()
        });

        assert!(info.is_space_character(1));
        assert!(!info.is_space_character(2));
        assert_eq!(info.get_display_text(1), Some("_"));
        assert_eq!(info.get_text(1), Some(" "));
    }
}
