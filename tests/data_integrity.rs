//! Integration tests for loading XML data files: alphabets, colors, training, etc.

use std::fs;
use std::path::Path;
use dasher_core::alphabet::{load_alphabet, load_color_schemes};

#[test]
fn test_all_alphabet_xmls_loadable() {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/alphabets");
    assert!(data_dir.exists(), "data/alphabets directory does not exist");
    let mut failures = Vec::new();
    for entry in fs::read_dir(&data_dir).expect("read_dir failed") {
        let entry = entry.expect("entry error");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("xml") {
            match load_alphabet(&path) {
                Ok(_) => { /* loaded successfully */ },
                Err(e) => failures.push((path.display().to_string(), format!("{:?}", e))),
            }
        }
    }
    if !failures.is_empty() {
        for (file, err) in &failures {
            eprintln!("Failed to load {}: {}", file, err);
        }
        panic!("Some alphabet XML files failed to load");
    }
}

#[test]
fn test_all_color_xmls_loadable() {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/colors");
    if !data_dir.exists() {
        eprintln!("data/colors directory does not exist, skipping");
        return;
    }
    let mut failures = Vec::new();
    for entry in fs::read_dir(&data_dir).expect("read_dir failed") {
        let entry = entry.expect("entry error");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("xml") {
            match load_color_schemes(&path) {
                Ok(_) => { /* loaded successfully */ },
                Err(e) => failures.push((path.display().to_string(), format!("{:?}", e))),
            }
        }
    }
    if !failures.is_empty() {
        for (file, err) in &failures {
            eprintln!("Failed to load color scheme {}: {}", file, err);
        }
        panic!("Some color XML files failed to load");
    }
}

// Placeholder for future training data tests
#[test]
fn test_training_data_dir_exists() {
    let data_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/training");
    assert!(data_dir.exists(), "data/training directory does not exist");
    // Add more comprehensive tests as Rust training data support matures.
}
