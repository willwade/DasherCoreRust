use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;
use std::env;

use super::{
    AlphabetInfo,
    color_schemes::{ColorScheme, load_color_schemes},
    AlphabetXmlError,
    load_alphabet,
};

/// Error type for alphabet discovery operations
#[derive(Debug)]
pub enum DiscoveryError {
    /// IO error
    Io(std::io::Error),
    /// XML error
    Xml(AlphabetXmlError),
    /// Invalid data
    InvalidData(String),
}

impl From<std::io::Error> for DiscoveryError {
    fn from(err: std::io::Error) -> Self {
        DiscoveryError::Io(err)
    }
}

impl From<AlphabetXmlError> for DiscoveryError {
    fn from(err: AlphabetXmlError) -> Self {
        DiscoveryError::Xml(err)
    }
}

/// Result of alphabet discovery
#[derive(Debug)]
pub struct DiscoveryResult {
    /// Found alphabets
    pub alphabets: HashMap<String, AlphabetInfo>,
    /// Found color schemes
    pub color_schemes: HashMap<String, ColorScheme>,
    /// Search paths that were used
    pub search_paths: Vec<PathBuf>,
}

/// Alphabet file discoverer
#[derive(Debug)]
pub struct AlphabetDiscovery {
    /// Base search paths
    search_paths: Vec<PathBuf>,
}

impl AlphabetDiscovery {
    /// Create a new alphabet discoverer with default search paths
    pub fn new() -> Result<Self, DiscoveryError> {
        let mut discovery = Self {
            search_paths: Vec::new(),
        };
        discovery.add_default_paths()?;
        Ok(discovery)
    }

    /// Add a search path
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Add default search paths
    fn add_default_paths(&mut self) -> Result<(), DiscoveryError> {
        // Add user config directory
        if let Some(config_dir) = dirs::config_dir() {
            let dasher_dir = config_dir.join("dasher");
            if dasher_dir.exists() {
                self.add_search_path(dasher_dir.join("alphabets"));
            }
        }

        // Add system config directory
        #[cfg(unix)]
        {
            self.add_search_path("/usr/share/dasher/alphabets");
            self.add_search_path("/usr/local/share/dasher/alphabets");
        }

        // Add current executable directory
        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                self.add_search_path(exe_dir.join("alphabets"));
            }
        }

        // Add current working directory
        if let Ok(cwd) = env::current_dir() {
            self.add_search_path(cwd.join("alphabets"));
        }

        Ok(())
    }

    /// Discover alphabet files
    pub fn discover(&self) -> Result<DiscoveryResult, DiscoveryError> {
        let mut result = DiscoveryResult {
            alphabets: HashMap::new(),
            color_schemes: HashMap::new(),
            search_paths: self.search_paths.clone(),
        };

        for path in &self.search_paths {
            if !path.exists() {
                continue;
            }

            // Find alphabet XML files
            let entries = fs::read_dir(path)?;
            for entry in entries {
                let entry = entry?;
                let file_path = entry.path();

                if !file_path.is_file() {
                    continue;
                }

                // Check file extension
                if let Some(ext) = file_path.extension() {
                    if ext != "xml" {
                        continue;
                    }

                    // Try to load as alphabet file
                    if let Ok(alphabet) = load_alphabet(&file_path) {
                        result.alphabets.insert(alphabet.id.clone(), alphabet);
                        continue;
                    }

                    // Try to load as color scheme file
                    if let Ok(schemes) = load_color_schemes(&file_path) {
                        for scheme in schemes {
                            result.color_schemes.insert(scheme.name.clone(), scheme);
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Find an alphabet by ID
    pub fn find_alphabet(&self, id: &str) -> Result<Option<AlphabetInfo>, DiscoveryError> {
        for path in &self.search_paths {
            let alphabet_path = path.join(format!("alphabet.{}.xml", id));
            if alphabet_path.exists() {
                return Ok(Some(load_alphabet(&alphabet_path)?));
            }
        }
        Ok(None)
    }

    /// Find color schemes in a specific file
    pub fn find_color_schemes<P: AsRef<Path>>(&self, path: P) -> Result<Vec<ColorScheme>, DiscoveryError> {
        Ok(load_color_schemes(path)?)
    }

    /// Get all search paths
    pub fn search_paths(&self) -> &[PathBuf] {
        &self.search_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_alphabet_discovery() -> Result<(), Box<dyn std::error::Error>> {
        // Create a temporary directory for test files
        let temp_dir = tempdir()?;
        let alphabet_dir = temp_dir.path().join("alphabets");
        fs::create_dir(&alphabet_dir)?;

        // Create a test alphabet file
        let alphabet_path = alphabet_dir.join("alphabet.test.xml");
        let alphabet_content = r#"<?xml version="1.0" encoding="UTF-8"?>
        <alphabet name="test">
            <setting name="encoding" value="UTF-8"/>
            <group name="letters">
                <character text="a" display="a"/>
                <character text="b" display="b"/>
            </group>
        </alphabet>"#;
        let mut file = File::create(&alphabet_path)?;
        file.write_all(alphabet_content.as_bytes())?;

        // Create a test color scheme file
        let scheme_path = alphabet_dir.join("colors.xml");
        let scheme_content = r#"<?xml version="1.0" encoding="UTF-8"?>
        <colorschemes>
            <scheme name="Test">
                <description>Test scheme</description>
                <pair index="0">
                    <foreground>#000000</foreground>
                    <background>#FFFFFF</background>
                </pair>
            </scheme>
        </colorschemes>"#;
        let mut file = File::create(&scheme_path)?;
        file.write_all(scheme_content.as_bytes())?;

        // Create discoverer and add test directory
        let mut discovery = AlphabetDiscovery::new()?;
        discovery.add_search_path(&alphabet_dir);

        // Test discovery
        let result = discovery.discover()?;
        assert!(result.alphabets.contains_key("test"));
        assert!(result.color_schemes.contains_key("Test"));

        // Test find_alphabet
        let alphabet = discovery.find_alphabet("test")?;
        assert!(alphabet.is_some());

        // Test find_color_schemes
        let schemes = discovery.find_color_schemes(&scheme_path)?;
        assert_eq!(schemes.len(), 1);
        assert_eq!(schemes[0].name, "Test");

        Ok(())
    }
}
