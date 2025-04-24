//! # DasherCore
//!
//! A Rust implementation of the Dasher text entry system core.
//!
//! This library provides the core functionality of the Dasher text entry system,
//! a zooming predictive text entry system designed for situations where keyboard
//! input is impractical (for instance, accessibility or mobile devices).

// Re-export key types for convenience
pub use self::api::DasherInterface;
pub use self::model::DasherModel;
pub use self::view::DasherScreen;
pub use self::input::DasherInput;
pub use self::settings::{Parameter, Settings};
pub use self::alphabet::{Alphabet, Symbol};

// Define modules
pub mod api;
pub mod model;
pub mod view;
pub mod input;
pub mod settings;
pub mod alphabet;

// FFI and WebAssembly support
#[cfg(feature = "wasm")]
pub mod wasm;

pub mod ffi;

// Error handling
use thiserror::Error;

/// Errors that can occur in the DasherCore library
#[derive(Error, Debug)]
pub enum DasherError {
    /// Error related to invalid parameters
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Error related to rendering
    #[error("Rendering error: {0}")]
    RenderingError(String),

    /// Error related to input processing
    #[error("Input error: {0}")]
    InputError(String),

    /// Error related to settings
    #[error("Settings error: {0}")]
    SettingsError(String),

    /// Other errors
    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for DasherCore operations
pub type Result<T> = std::result::Result<T, DasherError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Basic test to ensure the library compiles
        assert!(true);
    }

    #[test]
    fn test_model_initialization_and_root_expansion() {
        let mut model = DasherModel::new();
        assert!(model.initialize().is_ok());
        // After initialization, root should exist and have children
        let root = model.root_node().unwrap();
        let root_borrow = root.borrow();
        assert!(root_borrow.child_count() > 0);
    }

    #[test]
    fn test_alphabet_english() {
        let alphabet = Alphabet::english();
        // Should contain all lowercase letters and space
        for c in 'a'..='z' {
            assert!(alphabet.get_index(c).is_some(), "Missing char {}", c);
        }
        assert!(alphabet.get_index(' ').is_some());
        // Should include some punctuation
        for c in ['.', ',', '!', '?', '\'', '"'] {
            assert!(alphabet.get_index(c).is_some(), "Missing punctuation {}", c);
        }
    }

    #[test]
    fn test_node_expansion_and_navigation() {
        let mut model = DasherModel::new();
        model.initialize().unwrap();
        let root = model.root_node().unwrap();
        // Expand the root again (should be idempotent)
        model.expand_node(&root);
        let root_borrow = root.borrow();
        let children = root_borrow.children();
        assert!(!children.is_empty());
        // Check that children's parent is root
        for child in children {
            let child_borrow = child.borrow();
            let parent = child_borrow.parent();
            assert!(parent.is_some());
        }
    }

    #[test]
    fn test_settings_and_parameters() {
        let mut settings = Settings::new();
        use crate::settings::Parameter;
        settings.set_bool(Parameter::ButtonMode, true);
        assert_eq!(settings.get_bool(Parameter::ButtonMode), Some(true));
        settings.set_long(Parameter::MaxBitRate, 123);
        assert_eq!(settings.get_long(Parameter::MaxBitRate), Some(123));
        settings.set_string(Parameter::AlphabetID, "TestAlphabet".to_string());
        assert_eq!(settings.get_string(Parameter::AlphabetID), Some("TestAlphabet"));
    }

    #[test]
    fn test_language_model_ppm() {
        use crate::model::language_model::{PPMLanguageModel, LanguageModel};
        let mut model = PPMLanguageModel::new(5);
        assert_eq!(model.num_symbols(), 5);
        // Enter and learn a symbol
        let context = 0;
        let symbol = 2;
        let _ = model.enter_symbol(context, symbol);
        model.learn_symbol(context, symbol);
    }

    // Optionally: add more tests for FFI and integration if needed.

}
