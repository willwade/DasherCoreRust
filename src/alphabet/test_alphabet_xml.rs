// This file has been removed. Data integrity tests are now in tests/data_integrity.rs.

use std::fs;
use std::path::Path;
use crate::alphabet::load_alphabet;

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
