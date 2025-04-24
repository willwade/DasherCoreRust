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
}
