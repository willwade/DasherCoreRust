use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use quick_xml::Reader;
use quick_xml::events::Event;

use super::AlphabetXmlError;

/// Color scheme for alphabet visualization
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// Name of the color scheme
    pub name: String,
    /// Description of the color scheme
    pub description: String,
    /// Colors in the scheme (RGBA)
    pub colors: Vec<(u8, u8, u8, u8)>,
}

impl ColorScheme {
    /// Create a new color scheme
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            colors: Vec::new(),
        }
    }

    /// Add a color to the scheme
    pub fn add_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.colors.push((r, g, b, a));
    }
}

/// Load color schemes from an XML file
pub fn load_color_schemes<P: AsRef<Path>>(path: P) -> Result<Vec<ColorScheme>, AlphabetXmlError> {
    let file = std::fs::File::open(path)?;
    let file_reader = BufReader::new(file);
    let mut reader = Reader::from_reader(file_reader);
    let mut buf = Vec::new();
    let mut schemes = Vec::new();
    let mut current_scheme = None;

    loop {
        match reader.read_event_into(&mut buf)? {
            Event::Start(ref e) => {
                match e.name().as_ref() {
                    b"colorscheme" => {
                        let name = e.attributes()
                            .find(|a| a.as_ref().map(|a| a.key.as_ref() == b"name").unwrap_or(false))
                            .map(|a| -> Result<String, AlphabetXmlError> {
                                let attr = a?;
                                Ok(String::from_utf8_lossy(&attr.value).to_string())
                            })
                            .transpose()?
                            .unwrap_or_default();
                        let description = e.attributes()
                            .find(|a| a.as_ref().map(|a| a.key.as_ref() == b"description").unwrap_or(false))
                            .map(|a| -> Result<String, AlphabetXmlError> {
                                let attr = a?;
                                Ok(String::from_utf8_lossy(&attr.value).to_string())
                            })
                            .transpose()?
                            .unwrap_or_default();
                        current_scheme = Some(ColorScheme::new(name, description));
                    }
                    b"color" => {
                        if let Some(ref mut scheme) = current_scheme {
                            let r = e.attributes()
                                .find(|a| a.as_ref().map(|a| a.key.as_ref() == b"r").unwrap_or(false))
                                .map(|a| -> Result<u8, AlphabetXmlError> {
                                    let attr = a?;
                                    let val = String::from_utf8_lossy(&attr.value).parse::<u8>()?;
                                    Ok(val)
                                })
                                .transpose()?
                                .unwrap_or(0);
                            let g = e.attributes()
                                .find(|a| a.as_ref().map(|a| a.key.as_ref() == b"g").unwrap_or(false))
                                .map(|a| -> Result<u8, AlphabetXmlError> {
                                    let attr = a?;
                                    let val = String::from_utf8_lossy(&attr.value).parse::<u8>()?;
                                    Ok(val)
                                })
                                .transpose()?
                                .unwrap_or(0);
                            let b = e.attributes()
                                .find(|a| a.as_ref().map(|a| a.key.as_ref() == b"b").unwrap_or(false))
                                .map(|a| -> Result<u8, AlphabetXmlError> {
                                    let attr = a?;
                                    let val = String::from_utf8_lossy(&attr.value).parse::<u8>()?;
                                    Ok(val)
                                })
                                .transpose()?
                                .unwrap_or(0);
                            let a = e.attributes()
                                .find(|a| a.as_ref().map(|a| a.key.as_ref() == b"a").unwrap_or(false))
                                .map(|a| -> Result<u8, AlphabetXmlError> {
                                    let attr = a?;
                                    let val = String::from_utf8_lossy(&attr.value).parse::<u8>()?;
                                    Ok(val)
                                })
                                .transpose()?
                                .unwrap_or(255);
                            scheme.add_color(r, g, b, a);
                        }
                    }
                    _ => {}
                }
            }
            Event::End(ref e) => {
                if e.name().as_ref() == b"colorscheme" {
                    if let Some(scheme) = current_scheme.take() {
                        schemes.push(scheme);
                    }
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(schemes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_load_color_schemes() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test_schemes.xml");
        let mut file = File::create(&file_path)?;

        write!(file, r#"<?xml version="1.0" encoding="UTF-8"?>
<colorschemes>
    <colorscheme name="Default" description="Default color scheme">
        <color r="255" g="0" b="0" a="255"/>
        <color r="0" g="255" b="0" a="255"/>
        <color r="0" g="0" b="255" a="255"/>
    </colorscheme>
    <colorscheme name="Dark" description="Dark theme">
        <color r="50" g="50" b="50" a="255"/>
        <color r="100" g="100" b="100" a="255"/>
    </colorscheme>
</colorschemes>"#)?;

        let schemes = load_color_schemes(&file_path)?;
        assert_eq!(schemes.len(), 2);
        
        let default_scheme = &schemes[0];
        assert_eq!(default_scheme.name, "Default");
        assert_eq!(default_scheme.description, "Default color scheme");
        assert_eq!(default_scheme.colors.len(), 3);
        assert_eq!(default_scheme.colors[0], (255, 0, 0, 255));

        let dark_scheme = &schemes[1];
        assert_eq!(dark_scheme.name, "Dark");
        assert_eq!(dark_scheme.description, "Dark theme");
        assert_eq!(dark_scheme.colors.len(), 2);
        assert_eq!(dark_scheme.colors[0], (50, 50, 50, 255));

        Ok(())
    }
}
