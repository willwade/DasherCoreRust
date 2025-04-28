//! # Settings Module
//!
//! This module contains the implementation of settings management for Dasher.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameter types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Parameter {
    // Boolean parameters
    ButtonMode,
    DrawMouse,
    DrawMouseLine,
    
    // Long parameters
    MaxBitRate,
    ViewID,
    Language,
    Orientation,
    
    // String parameters
    AlphabetID,
    ColourID,
    
    // TODO: Add more parameters as needed
}

/// Parameter value types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterValue {
    Bool(bool),
    Long(i64),
    String(String),
}

/// Settings store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    values: HashMap<Parameter, ParameterValue>,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    /// Create a new settings store with default values
    pub fn new() -> Self {
        let mut values = HashMap::new();
        
        // Set default values
        values.insert(Parameter::ButtonMode, ParameterValue::Bool(false));
        values.insert(Parameter::DrawMouse, ParameterValue::Bool(true));
        values.insert(Parameter::DrawMouseLine, ParameterValue::Bool(false));
        values.insert(Parameter::MaxBitRate, ParameterValue::Long(100));
        values.insert(Parameter::ViewID, ParameterValue::Long(1));
        values.insert(Parameter::Language, ParameterValue::Long(0));
        values.insert(Parameter::Orientation, ParameterValue::Long(0)); // Default: LeftToRight
        values.insert(Parameter::AlphabetID, ParameterValue::String("Default".to_string()));
        values.insert(Parameter::ColourID, ParameterValue::String("Default".to_string()));
        
        Self { values }
    }
    
    /// Get a boolean parameter value
    pub fn get_bool(&self, param: Parameter) -> Option<bool> {
        match self.values.get(&param) {
            Some(ParameterValue::Bool(value)) => Some(*value),
            _ => None,
        }
    }
    
    /// Get a long parameter value
    pub fn get_long(&self, param: Parameter) -> Option<i64> {
        match self.values.get(&param) {
            Some(ParameterValue::Long(value)) => Some(*value),
            _ => None,
        }
    }
    
    /// Get a string parameter value
    pub fn get_string(&self, param: Parameter) -> Option<&str> {
        match self.values.get(&param) {
            Some(ParameterValue::String(value)) => Some(value),
            _ => None,
        }
    }
    
    /// Set a boolean parameter value
    pub fn set_bool(&mut self, param: Parameter, value: bool) {
        self.values.insert(param, ParameterValue::Bool(value));
    }
    
    /// Set a long parameter value
    pub fn set_long(&mut self, param: Parameter, value: i64) {
        self.values.insert(param, ParameterValue::Long(value));
    }
    
    /// Set a string parameter value
    pub fn set_string(&mut self, param: Parameter, value: String) {
        self.values.insert(param, ParameterValue::String(value));
    }
    
    /// Reset a parameter to its default value
    pub fn reset_parameter(&mut self, param: Parameter) {
        match param {
            Parameter::ButtonMode => self.set_bool(param, false),
            Parameter::DrawMouse => self.set_bool(param, true),
            Parameter::DrawMouseLine => self.set_bool(param, false),
            Parameter::MaxBitRate => self.set_long(param, 100),
            Parameter::ViewID => self.set_long(param, 1),
            Parameter::Language => self.set_long(param, 0),
            Parameter::Orientation => self.set_long(param, 0),
            Parameter::AlphabetID => self.set_string(param, "Default".to_string()),
            Parameter::ColourID => self.set_string(param, "Default".to_string()),
        }
    }
}
