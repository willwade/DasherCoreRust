use quick_xml::{Reader, Writer, events::{Event, BytesStart, BytesEnd, BytesText, BytesDecl}};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::fs::File;
use std::path::Path;

use super::{
    AlphabetInfo,
    color_schemes::ColorScheme,
    Character,
    GroupInfo,
    ScreenOrientation,
    AlphabetConversion,
    Color,
    ColorManager,
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
        AlphabetXmlError::XmlError(err)
    }
}

impl From<quick_xml::events::attributes::AttrError> for AlphabetXmlError {
    fn from(err: quick_xml::events::attributes::AttrError) -> Self {
        AlphabetXmlError::XmlError(quick_xml::Error::from(err))
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
            match self.reader.read_event(&mut buf)? {
                Event::Start(ref e) => {
                    match e.name() {
                        b"scheme" => {
                            let attrs: Vec<_> = e.attributes().collect::<Result<_, _>>()?;
                            let name = attrs.iter()
                                .find(|attr| attr.key == b"name")
                                .ok_or_else(|| AlphabetXmlError::InvalidData("Missing scheme name".into()))?
                                .value
                                .as_ref();
                            let name = String::from_utf8_lossy(name).into_owned();
                            current_scheme = Some(ColorScheme::new(&name));
                        }
                        b"pair" => {
                            current_pair = Some(());
                        }
                        _ => {}
                    }
                }
                Event::Text(e) => {
                    if let Some(scheme) = current_scheme.as_mut() {
                        if current_pair.is_some() {
                            let text = e.unescape_and_decode(&self.reader)?;
                            if current_fg.is_none() {
                                if let Some(color) = Color::from_hex(&text) {
                                    current_fg = Some(color);
                                }
                            } else if let Some(fg) = current_fg.take() {
                                if let Some(bg) = Color::from_hex(&text) {
                                    scheme.add_color_pair(fg, bg);
                                }
                            }
                        } else if scheme.description.is_empty() {
                            scheme.description = e.unescape_and_decode(&self.reader)?.into_owned();
                        }
                    }
                }
                Event::End(ref e) => {
                    match e.name() {
                        b"scheme" => {
                            if let Some(scheme) = current_scheme.take() {
                                schemes.push(scheme);
                            }
                        }
                        b"pair" => {
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
            match self.reader.read_event(&mut buf)? {
                Event::Start(ref e) => {
                    match e.name() {
                        b"alphabet" => {
                            let attrs: Vec<_> = e.attributes().collect::<Result<_, _>>()?;
                            let id = attrs.iter()
                                .find(|attr| attr.key == b"name")
                                .ok_or_else(|| AlphabetXmlError::InvalidData("Missing alphabet name".into()))?
                                .value
                                .as_ref();
                            let id = String::from_utf8_lossy(id).into_owned();
                            alphabet = Some(AlphabetInfo::new(id));
                        }
                        b"group" => {
                            let attrs: Vec<_> = e.attributes().collect::<Result<_, _>>()?;
                            let name = attrs.iter()
                                .find(|attr| attr.key == b"name")
                                .ok_or_else(|| AlphabetXmlError::InvalidData("Missing group name".into()))?
                                .value
                                .as_ref();
                            let name = String::from_utf8_lossy(name).into_owned();
                            current_group = Some(GroupInfo::new(name));
                        }
                        b"character" => {
                            current_character = Some(Character::default());
                        }
                        _ => {}
                    }
                }
                Event::Text(e) => {
                    if let Some(character) = current_character.as_mut() {
                        character.text = e.unescape_and_decode(&self.reader)?.into_owned();
                    }
                }
                Event::End(e) => {
                    match e.name() {
                        b"group" => {
                            if let (Some(group), Some(alphabet)) = (current_group.take(), alphabet.as_mut()) {
                                alphabet.characters.extend(group.children.iter().flat_map(|child| &child.children));
                            }
                        }
                        b"character" => {
                            if let (Some(character), Some(group)) = (current_character.take(), current_group.as_mut()) {
                                group.children.push(character);
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
        self.writer.write_event(Event::Decl(
            BytesStart::borrowed_name(b"xml")
                .with_attributes(vec![("version", "1.0"), ("encoding", "UTF-8")])
                .into_owned(),
        ))?;

        // Write color schemes element
        let schemes_elem = BytesStart::owned(b"colorschemes".to_vec(), "colorschemes".len());
        self.writer.write_event(Event::Start(schemes_elem.clone()))?;

        // Write each scheme
        for scheme in schemes {
            let mut scheme_elem = BytesStart::owned(b"scheme".to_vec(), "scheme".len());
            scheme_elem.push_attribute(("name", scheme.name.as_str()));
            self.writer.write_event(Event::Start(scheme_elem.clone()))?;

            // Write description
            if !scheme.description.is_empty() {
                self.write_element("description", &scheme.description)?;
            }

            // Write color pairs
            for (i, (fg, bg)) in scheme.foreground_colors.iter().zip(&scheme.background_colors).enumerate() {
                let mut pair_elem = BytesStart::owned(b"pair".to_vec(), "pair".len());
                pair_elem.push_attribute(("index", i.to_string().as_str()));
                self.writer.write_event(Event::Start(pair_elem.clone()))?;

                self.write_element("foreground", &fg.to_hex())?;
                self.write_element("background", &bg.to_hex())?;

                self.writer.write_event(Event::End(BytesEnd::borrowed(pair_elem.name())))?;
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

    /// Write an alphabet to XML
    pub fn write(&mut self, alphabet: &AlphabetInfo) -> Result<(), AlphabetXmlError> {
        // Write XML declaration
        self.writer.write_event(Event::Decl(
            BytesStart::borrowed_name(b"xml")
                .with_attributes(vec![("version", "1.0"), ("encoding", "UTF-8")])
                .into_owned(),
        ))?;

        // Write alphabet element
        let mut alphabet_elem = BytesStart::new("alphabet");
        alphabet_elem.push_attribute(("name", alphabet.id.as_str()));
        self.writer.write_event(Event::Start(alphabet_elem.clone()))?;

        // Write settings
        self.write_setting("training", &alphabet.training_file)?;
        self.write_setting("orientation", &format!("{:?}", alphabet.orientation))?;
        self.write_setting("encoding", "UTF-8")?;

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
            match reader.read_event(&mut buf).unwrap() {
                Event::Start(ref e) if e.name() == b"alphabet" => {
                    found_alphabet = true;
                    let attrs: Vec<_> = e.attributes().collect::<Result<_, _>>().unwrap();
                    let name = attrs.iter()
                        .find(|attr| attr.key == b"name")
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
