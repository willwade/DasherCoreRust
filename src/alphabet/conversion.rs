use std::collections::HashMap;
use super::info::{AlphabetInfo, AlphabetConversion};

/// A conversion rule for converting between symbol sets
#[derive(Debug, Clone)]
pub struct ConversionRule {
    /// The input symbol (what is shown in Dasher)
    pub input: String,
    /// The output symbol (what is written)
    pub output: String,
    /// The context in which this rule applies (empty for context-insensitive)
    pub context: String,
    /// Whether this rule is trainable
    pub trainable: bool,
}

/// A conversion table for converting between symbol sets
#[derive(Debug, Clone)]
pub struct ConversionTable {
    /// The rules in this table
    rules: Vec<ConversionRule>,
    /// Map from input to rule indices for fast lookup
    input_map: HashMap<String, Vec<usize>>,
    /// Map from context to rule indices for fast lookup
    context_map: HashMap<String, Vec<usize>>,
}

impl From<ConversionRule> for ConversionTable {
    fn from(rule: ConversionRule) -> Self {
        let mut table = ConversionTable::new();
        table.add_rule(rule);
        table
    }
}

impl Default for ConversionTable {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversionTable {
    /// Get all rules in this table
    pub fn get_all_rules(&self) -> &[ConversionRule] {
        &self.rules
    }
    
    /// Create a new empty conversion table
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            input_map: HashMap::new(),
            context_map: HashMap::new(),
        }
    }

    /// Add a conversion rule
    pub fn add_rule(&mut self, rule: ConversionRule) {
        let index = self.rules.len();
        
        // Add to input map
        self.input_map
            .entry(rule.input.clone())
            .or_default()
            .push(index);
            
        // Add to context map if context is not empty
        if !rule.context.is_empty() {
            self.context_map
                .entry(rule.context.clone())
                .or_default()
                .push(index);
        }
        
        self.rules.push(rule);
    }

    /// Get all rules that match an input
    pub fn get_rules_for_input(&self, input: &str) -> Vec<&ConversionRule> {
        self.input_map
            .get(input)
            .map(|indices| indices.iter().map(|&i| &self.rules[i]).collect())
            .unwrap_or_default()
    }

    /// Get all rules that match a context
    pub fn get_rules_for_context(&self, context: &str) -> Vec<&ConversionRule> {
        self.context_map
            .get(context)
            .map(|indices| indices.iter().map(|&i| &self.rules[i]).collect())
            .unwrap_or_default()
    }
}

/// Manager for alphabet conversions
#[derive(Debug)]
pub struct ConversionManager {
    /// Tables for each conversion type
    tables: HashMap<AlphabetConversion, ConversionTable>,
    /// Current context for context-sensitive conversions
    current_context: String,
}

impl Default for ConversionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversionManager {
    /// Get rules for a specific conversion type
    pub fn get_rules(&self, conversion_type: AlphabetConversion) -> Option<&ConversionTable> {
        self.tables.get(&conversion_type)
    }
    
    /// Create a new conversion manager
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
            current_context: String::new(),
        }
    }

    /// Set the current context
    pub fn set_context(&mut self, context: String) {
        self.current_context = context;
    }

    /// Get the current context
    pub fn get_context(&self) -> &str {
        &self.current_context
    }

    /// Add a conversion table for a conversion type
    pub fn add_table(&mut self, conversion_type: AlphabetConversion, table: ConversionTable) {
        self.tables.insert(conversion_type, table);
    }

    /// Convert text using the specified conversion type
    pub fn convert(&self, text: &str, conversion_type: AlphabetConversion) -> String {
        match conversion_type {
            AlphabetConversion::None => text.to_string(),
            AlphabetConversion::Mandarin => self.convert_mandarin(text),
            AlphabetConversion::RoutingContextInsensitive => self.convert_context_insensitive(text),
            AlphabetConversion::RoutingContextSensitive => self.convert_context_sensitive(text),
        }
    }

    /// Convert text using Mandarin conversion
    fn convert_mandarin(&self, text: &str) -> String {
        if let Some(table) = self.tables.get(&AlphabetConversion::Mandarin) {
            let mut result = String::new();
            let mut remaining = text;
            while !remaining.is_empty() {
                let mut matched = false;
                // Try to match the longest possible substring
                for len in (1..=remaining.len()).rev() {
                    let input = &remaining[..len];
                    if let Some(rule) = table.get_rules_for_input(input).first() {
                        result.push_str(&rule.output);
                        remaining = &remaining[len..];
                        matched = true;
                        break;
                    }
                }
                if !matched {
                    // No rule matched, copy first char
                    let ch = remaining.chars().next().unwrap();
                    result.push(ch);
                    remaining = &remaining[ch.len_utf8()..];
                }
            }
            result
        } else {
            text.to_string()
        }
    }

    /// Convert text using context-insensitive routing
    fn convert_context_insensitive(&self, text: &str) -> String {
        if let Some(table) = self.tables.get(&AlphabetConversion::RoutingContextInsensitive) {
            let mut result = String::new();
            let mut remaining = text;
            
            while !remaining.is_empty() {
                let mut matched = false;
                
                // Try to match the longest possible input
                for len in (1..=remaining.len()).rev() {
                    let input = &remaining[..len];
                    if let Some(rule) = table.get_rules_for_input(input).first() {
                        result.push_str(&rule.output);
                        remaining = &remaining[len..];
                        matched = true;
                        break;
                    }
                }
                
                // If no match found, copy the first character as is
                if !matched {
                    result.push(remaining.chars().next().unwrap());
                    remaining = &remaining[1..];
                }
            }
            
            result
        } else {
            text.to_string()
        }
    }

    /// Convert text using context-sensitive routing
    fn convert_context_sensitive(&self, text: &str) -> String {
        if let Some(table) = self.tables.get(&AlphabetConversion::RoutingContextSensitive) {
            let mut result = String::new();
            let mut remaining = text;
            
            while !remaining.is_empty() {
                let mut matched = false;
                
                // Try to match rules with the current context first
                for len in (1..=remaining.len()).rev() {
                    let input = &remaining[..len];
                    let context_rules: Vec<_> = table.get_rules_for_context(&self.current_context)
                        .into_iter()
                        .filter(|r| r.input == input)
                        .collect();
                    
                    if let Some(rule) = context_rules.first() {
                        result.push_str(&rule.output);
                        remaining = &remaining[len..];
                        matched = true;
                        break;
                    }
                }
                
                // If no context match, try context-insensitive rules
                if !matched {
                    for len in (1..=remaining.len()).rev() {
                        let input = &remaining[..len];
                        let rules = table.get_rules_for_input(input);
                        if let Some(rule) = rules.iter().find(|r| r.context.is_empty()) {
                            result.push_str(&rule.output);
                            remaining = &remaining[len..];
                            matched = true;
                            break;
                        }
                    }
                }
                
                // If still no match, copy the first character as is
                if !matched {
                    result.push(remaining.chars().next().unwrap());
                    remaining = &remaining[1..];
                }
            }
            
            result
        } else {
            text.to_string()
        }
    }

    /// Create conversion tables from alphabet info
    pub fn from_alphabet(alphabet: &AlphabetInfo) -> Self {
        let mut manager = Self::new();
        
        match alphabet.conversion_type {
            AlphabetConversion::None => {}
            AlphabetConversion::Mandarin => {
                let mut table = ConversionTable::new();
                for character in &alphabet.characters {
                    if !character.text.is_empty() && character.text != character.display {
                        table.add_rule(ConversionRule {
                            input: character.display.clone(),
                            output: character.text.clone(),
                            context: String::new(),
                            trainable: false,
                        });
                    }
                }
                manager.add_table(AlphabetConversion::Mandarin, table);
            }
            AlphabetConversion::RoutingContextInsensitive |
            AlphabetConversion::RoutingContextSensitive => {
                let mut table = ConversionTable::new();
                let mut in_training = false;
                let mut current_input = String::new();
                
                for character in &alphabet.characters {
                    if character.text == alphabet.conversion_train_start {
                        in_training = true;
                        current_input.clear();
                    } else if character.text == alphabet.conversion_train_stop {
                        in_training = false;
                    } else if in_training {
                        current_input.push_str(&character.text);
                    } else if !current_input.is_empty() {
                        table.add_rule(ConversionRule {
                            input: current_input.clone(),
                            output: character.text.clone(),
                            context: String::new(),
                            trainable: true,
                        });
                        current_input.clear();
                    }
                }
                
                manager.add_table(alphabet.conversion_type, table);
            }
        }
        
        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mandarin_conversion() {
        let mut table = ConversionTable::new();
        table.add_rule(ConversionRule {
            input: "pin".to_string(),
            output: "拼".to_string(),
            context: String::new(),
            trainable: false,
        });
        
        let mut manager = ConversionManager::new();
        manager.add_table(AlphabetConversion::Mandarin, table);
        
        assert_eq!(manager.convert("pin", AlphabetConversion::Mandarin), "拼");
        assert_eq!(manager.convert("test", AlphabetConversion::Mandarin), "test");
    }

    #[test]
    fn test_context_sensitive_conversion() {
        let mut table = ConversionTable::new();
        table.add_rule(ConversionRule {
            input: "th".to_string(),
            output: "θ".to_string(),
            context: "greek".to_string(),
            trainable: false,
        });
        table.add_rule(ConversionRule {
            input: "th".to_string(),
            output: "th".to_string(),
            context: String::new(),
            trainable: false,
        });
        
        let mut manager = ConversionManager::new();
        manager.add_table(AlphabetConversion::RoutingContextSensitive, table);
        
        // Test default context
        assert_eq!(manager.convert("th", AlphabetConversion::RoutingContextSensitive), "th");
        
        // Test Greek context
        manager.set_context("greek".to_string());
        assert_eq!(manager.convert("th", AlphabetConversion::RoutingContextSensitive), "θ");
    }
}
