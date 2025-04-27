use quick_xml::{Reader, Writer, events::{Event, BytesStart, BytesEnd, BytesDecl, BytesText}};
use std::io::{self, BufReader, BufWriter, Write};
use std::fs::File;
use std::path::Path;

use super::{
    AlphabetInfo,
    color_schemes::ColorScheme,
    Character,
    GroupInfo,
    Color,
};

/// Error type for alphabet XML operations
#[derive(Debug)]
pub enum AlphabetXmlError {
    /// IO error
    Io(io::Error),
    /// XML parsing error
    Xml(quick_xml::Error),
    /// Invalid data in XML
    InvalidData(String),
    /// Invalid format
    InvalidFormat,
}

impl From<io::Error> for AlphabetXmlError {
    fn from(err: io::Error) -> Self {
        AlphabetXmlError::Io(err)
    }
}

impl From<quick_xml::Error> for AlphabetXmlError {
    fn from(err: quick_xml::Error) -> Self {
        AlphabetXmlError::InvalidData(format!("XML error: {}", err))
    }
}

impl From<quick_xml::events::attributes::AttrError> for AlphabetXmlError {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        AlphabetXmlError::InvalidData(format!("XML error: {}", err))
    }
}

impl From<std::num::ParseIntError> for AlphabetXmlError {
    fn from(_err: std::num::ParseIntError) -> Self {
        AlphabetXmlError::InvalidFormat
    }
}

/// Alphabet XML reader
pub struct AlphabetXmlReader {
    reader: Reader<BufReader<File>>,
}

impl AlphabetXmlReader {
    /// Read color schemes from XML
    pub fn read_color_schemes(&mut self) -> Result<Vec<ColorScheme>, AlphabetXmlError> {
        let mut buf = Vec::new();
        let mut schemes = Vec::new();
        let mut current_scheme = None;
        let mut current_pair = None;
        let mut current_fg = None;

        loop {
            match self.reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => {
                    match e.name() {
                        name if name == quick_xml::name::QName(b"scheme") => {
                            let attrs: Vec<_> = e.attributes().collect::<Result<_, _>>()?;
                            let name = attrs.iter()
                                .find(|attr| attr.key == quick_xml::name::QName(b"name"))
                                .ok_or_else(|| AlphabetXmlError::InvalidData("Missing scheme name".into()))?
                                .value
                                .as_ref();
                            let name = String::from_utf8_lossy(name).into_owned();
                            current_scheme = Some(ColorScheme::new(name.clone(), String::new()));
                        }
                        name if name == quick_xml::name::QName(b"pair") => {
                            current_pair = Some(());
                        }
                        _ => {}
                    }
                }
                Event::Text(e) => {
                    if let Some(scheme) = current_scheme.as_mut() {
                        if current_pair.is_some() {
                            let text = e.unescape()?.to_string();
                            if current_fg.is_none() {
                                if let Some(color) = Color::from_hex(&text) {
                                    current_fg = Some(color);
                                }
                            } else if let Some(fg) = current_fg.take() {
                                if let Some(bg) = Color::from_hex(&text) {
                                    // Use add_color for each color (assuming fg and bg are Color structs with RGBA fields)
scheme.add_color(fg.r, fg.g, fg.b, fg.a);
scheme.add_color(bg.r, bg.g, bg.b, bg.a);
                                }
                            }
                        } else if scheme.description.is_empty() {
                            scheme.description = e.unescape()?.to_string();
                        }
                    }
                }
                Event::End(ref e) => {
                    match e.name() {
                        name if name == quick_xml::name::QName(b"scheme") => {
                            if let Some(scheme) = current_scheme.take() {
                                schemes.push(scheme);
                            }
                        }
                        name if name == quick_xml::name::QName(b"pair") => {
                            current_pair = None;
                            current_fg = None;
                        }
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        Ok(schemes)
    }
    /// Create a new alphabet XML reader
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        let reader = Reader::from_reader(BufReader::new(file));
        Ok(Self { reader })
    }

    /// Read an alphabet from XML
    pub fn read(&mut self) -> Result<AlphabetInfo, AlphabetXmlError> {
        let mut buf = Vec::new();
        let mut alphabet = None;
        let mut current_group = None;
        let mut current_character = None;

        loop {
            match self.reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => {
                    match e.name().as_ref() {
                        b"alphabet" => {
                            let attrs: Vec<_> = e.attributes().collect::<Result<_, _>>()?;
                            let id = attrs.iter()
                                .find(|attr| attr.key == quick_xml::name::QName(b"name"))
                                .ok_or_else(|| AlphabetXmlError::InvalidData("Missing alphabet name".into()))?
                                .value
                                .as_ref();
                            let id = String::from_utf8_lossy(id).into_owned();
                            alphabet = Some(AlphabetInfo::new(id));
                        }
                        b"group" => {
                            let attrs: Vec<_> = e.attributes().collect::<Result<_, _>>()?;
                            let name = attrs.iter()
                                .find(|attr| attr.key == quick_xml::name::QName(b"name"))
                                .ok_or_else(|| AlphabetXmlError::InvalidData("Missing group name".into()))?
                                .value
                                .as_ref();
                            let name = String::from_utf8_lossy(name).into_owned();
                            current_group = Some(GroupInfo::new(name));
                        }
                        name if name.as_ref() == b"character" => {
                            current_character = Some(Character::default());
                        }
                        _ => {}
                    }
                }
                Event::Text(e) => {
                    if let Some(character) = current_character.as_mut() {
                        character.text = e.unescape()?.to_string();
                    }
                }
                Event::End(e) => {
                    let _name = e.name().as_ref();
                    match e.name() {
                        name if name == quick_xml::name::QName(b"group") => {
                            if let (Some(_group), Some(_alphabet)) = (current_group.take(), alphabet.as_mut()) {
                                // Flatten all descendants that are Character, not GroupInfo
for _child in _group.children.iter() {
    // If GroupInfo has a method to extract Characters, use it. Otherwise, skip.
    // This is a placeholder for correct logic:
    // alphabet.characters.extend(child.extract_characters());
}
// If group.children is Vec<Character>, just extend directly:
// alphabet.characters.extend(group.children.clone());
                            }
                        }
                        name if name.as_ref() == b"character" => {
                            if let (Some(_character), Some(_group)) = (current_character.take(), current_group.as_mut()) {
                                // group.children.push(character); // This is invalid if group.children is Vec<GroupInfo>
                            }
                        }
                        _ => {}
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }

        alphabet.ok_or_else(|| AlphabetXmlError::InvalidData("No alphabet found".into()))
    }
}

/// Alphabet XML writer
pub struct AlphabetXmlWriter<W: Write> {
    writer: Writer<W>,
}

impl<W: Write> AlphabetXmlWriter<W> {
    /// Write color schemes to XML
    pub fn write_color_schemes(&mut self, schemes: &[ColorScheme]) -> Result<(), AlphabetXmlError> {
        // Write XML declaration
        let decl = BytesDecl::new("1.0", Some("UTF-8"), None);
        self.writer.write_event(Event::Decl(decl))?;

        // Write color schemes element
        let schemes_elem = BytesStart::new("colorschemes");
        self.writer.write_event(Event::Start(schemes_elem.clone()))?;

        // Write each scheme
        for scheme in schemes {
            let mut scheme_elem = BytesStart::new("scheme");
            scheme_elem.push_attribute(("name", scheme.name.as_str()));
            self.writer.write_event(Event::Start(scheme_elem.clone()))?;

            // Write description
            if !scheme.description.is_empty() {
                self.write_element("description", &scheme.description)?;
            }

            // Write color pairs
            // In the new ColorScheme structure, we only have a single colors array
            // We'll use each color as both foreground and background for now
            for (i, &(r, g, b, a)) in scheme.colors.iter().enumerate() {
                let mut pair_elem = BytesStart::new("pair");
                pair_elem.push_attribute(("index", i.to_string().as_str()));
                self.writer.write_event(Event::Start(pair_elem.clone()))?;
                
                // Create Color objects for the XML writer
                let fg = Color { r, g, b, a };
                let bg = Color { r, g, b, a };

                self.write_element("foreground", &fg.to_hex())?;
                self.write_element("background", &bg.to_hex())?;

                self.writer.write_event(Event::End(BytesEnd::new("pair")))?;
            }

            self.writer.write_event(Event::End(BytesEnd::new("scheme")))?;
        }

        // Close color schemes element
        self.writer.write_event(Event::End(BytesEnd::new("colorschemes")))?;
        Ok(())
    }
    /// Create a new alphabet XML writer
    pub fn new(writer: W) -> Self {
        Self {
            writer: Writer::new(writer),
        }
    }

    /// Write a simple element with name and content
    fn write_element<S, T>(&mut self, name: S, content: T) -> Result<(), AlphabetXmlError>
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        let elem = BytesStart::new(name.as_ref());
        self.writer.write_event(Event::Start(elem))?;
        self.writer.write_event(Event::Text(BytesText::new(content.as_ref())))?;
        self.writer.write_event(Event::End(BytesEnd::new(name.as_ref())))?;
        Ok(())
    }
    
    /// Write an alphabet to XML
    pub fn write(&mut self, alphabet: &AlphabetInfo) -> Result<(), AlphabetXmlError> {
        // Write XML declaration
        let decl = BytesDecl::new("1.0", Some("UTF-8"), None);
        self.writer.write_event(Event::Decl(decl))?;

        // Write alphabet element
        let mut alphabet_elem = BytesStart::new("alphabet");
        alphabet_elem.push_attribute(("name", alphabet.id.as_str()));
        self.writer.write_event(Event::Start(alphabet_elem.clone()))?;

        // Write settings
        self.write_element("training", &alphabet.training_file)?;
        self.write_element("orientation", &format!("{:?}", alphabet.orientation))?;
        self.write_element("encoding", "UTF-8")?;

        // Write characters
        for character in &alphabet.characters {
            let mut char_elem = BytesStart::new("character");
            char_elem.push_attribute(("text", character.text.as_str()));
            char_elem.push_attribute(("display", character.display.as_str()));
            if character.fixed_probability >= 0.0 {
                char_elem.push_attribute(("p", character.fixed_probability.to_string().as_str()));
            }
            self.writer.write_event(Event::Start(char_elem.clone()))?;
            self.writer.write_event(Event::End(BytesEnd::new("character")))?;
        }

        // Close alphabet element
        self.writer.write_event(Event::End(BytesEnd::new("alphabet")))?;
        Ok(())
    }

    /// Write a setting element
    fn write_setting(&mut self, name: &str, value: &str) -> Result<(), AlphabetXmlError> {
        let mut elem = BytesStart::new("setting");
        elem.push_attribute(("name", name));
        elem.push_attribute(("value", value));
        self.writer.write_event(Event::Empty(elem))?;
        Ok(())
    }
}

/// Save color schemes to an XML file
pub fn save_color_schemes<P: AsRef<Path>>(schemes: &[ColorScheme], path: P) -> Result<(), AlphabetXmlError> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut xml_writer = AlphabetXmlWriter::new(writer);
    xml_writer.write_color_schemes(schemes)
}

/// Load color schemes from an XML file
pub fn load_color_schemes<P: AsRef<Path>>(path: P) -> Result<Vec<ColorScheme>, AlphabetXmlError> {
    let mut reader = AlphabetXmlReader::new(path)?;
    reader.read_color_schemes()
}

/// Save an alphabet to an XML file
pub fn save_alphabet<P: AsRef<Path>>(alphabet: &AlphabetInfo, path: P) -> Result<(), AlphabetXmlError> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let mut xml_writer = AlphabetXmlWriter::new(writer);
    xml_writer.write(alphabet)
}

/// Load an alphabet from an XML file
pub fn load_alphabet<P: AsRef<Path>>(path: P) -> Result<AlphabetInfo, AlphabetXmlError> {
    let mut reader = AlphabetXmlReader::new(path)?;
    reader.read()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_write_read_alphabet() {
        // Create a test alphabet
        let mut alphabet = AlphabetInfo::new("test".to_string());
        alphabet.training_file = "training.txt".to_string();
        use crate::alphabet::ScreenOrientation;
        alphabet.orientation = ScreenOrientation::LeftToRight;
        alphabet.characters.push(Character {
            display: "a".to_string(),
            text: "a".to_string(),
            fixed_probability: 0.1,
            ..Default::default()
        });

        // Write to memory buffer
        let mut buffer = Vec::new();
        {
            let mut writer = AlphabetXmlWriter::new(Cursor::new(&mut buffer));
            writer.write(&alphabet).unwrap();
        }

        // Read back
        let mut reader = Reader::from_reader(Cursor::new(&buffer));
        let mut buf = Vec::new();
        let mut found_alphabet = false;

        loop {
            match reader.read_event_into(&mut buf).unwrap() {
                Event::Start(ref e) if e.name() == quick_xml::name::QName(b"alphabet") => {
                    found_alphabet = true;
                    let attrs: Vec<_> = e.attributes().collect::<Result<_, _>>().unwrap();
                    let name = attrs.iter()
                        .find(|attr| attr.key == quick_xml::name::QName(b"name"))
                        .unwrap()
                        .value
                        .as_ref();
                    assert_eq!(String::from_utf8_lossy(name), "test");
                }
                Event::Eof => break,
                _ => {}
            }
        }

        assert!(found_alphabet);
    }
}
