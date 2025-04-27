use std::collections::HashMap;
use std::str::FromStr;

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    /// Red component (0-255)
    pub r: u8,
    /// Green component (0-255)
    pub g: u8,
    /// Blue component (0-255)
    pub b: u8,
    /// Alpha component (0-255)
    pub a: u8,
}

impl Color {
    /// Create a new color from RGB components
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    /// Convert color to hex string (e.g., "#FF0000")
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    /// Create a color from a hex string (e.g., "#FF0000" or "FF0000")
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }
        
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        
        Some(Self::new(r, g, b))
    }



    /// Get the luminance of the color (perceived brightness)
    pub fn luminance(&self) -> f32 {
        // Using the formula from W3C: https://www.w3.org/TR/WCAG20/#relativeluminancedef
        let r = (self.r as f32) / 255.0;
        let g = (self.g as f32) / 255.0;
        let b = (self.b as f32) / 255.0;
        
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Check if the color provides good contrast with another color
    pub fn has_good_contrast_with(&self, other: &Color) -> bool {
        let l1 = self.luminance();
        let l2 = other.luminance();
        
        // Calculate contrast ratio according to WCAG 2.0
        let lighter = f32::max(l1, l2);
        let darker = f32::min(l1, l2);
        
        (lighter + 0.05) / (darker + 0.05) >= 4.5
    }
}

impl FromStr for Color {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_hex(s).ok_or("Invalid hex color")
    }
}

/// Color scheme for alphabet groups
#[derive(Debug, Clone)]
pub struct ColorScheme {
    /// Name of the color scheme
    pub name: String,
    /// Description of the color scheme
    pub description: String,
    /// Background colors for groups
    pub background_colors: Vec<Color>,
    /// Foreground colors for groups
    pub foreground_colors: Vec<Color>,
}

impl ColorScheme {
    /// Create a new color scheme
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: String::new(),
            background_colors: Vec::new(),
            foreground_colors: Vec::new(),
        }
    }

    /// Add a color pair (foreground and background)
    pub fn add_color_pair(&mut self, foreground: Color, background: Color) {
        self.foreground_colors.push(foreground);
        self.background_colors.push(background);
    }

    /// Get a color pair by index
    pub fn get_color_pair(&self, index: usize) -> Option<(Color, Color)> {
        if index < self.background_colors.len() && index < self.foreground_colors.len() {
            Some((self.foreground_colors[index], self.background_colors[index]))
        } else {
            None
        }
    }
}

/// Manager for color schemes
#[derive(Debug, Default)]
pub struct ColorManager {
    /// Available color schemes
    schemes: HashMap<String, ColorScheme>,
}

impl ColorManager {
    /// Create a new color manager
    pub fn new() -> Self {
        let mut manager = Self::default();
        manager.add_default_schemes();
        manager
    }

    /// Add a color scheme
    pub fn add_scheme(&mut self, scheme: ColorScheme) {
        self.schemes.insert(scheme.name.clone(), scheme);
    }

    /// Get a color scheme by name
    pub fn get_scheme(&self, name: &str) -> Option<&ColorScheme> {
        self.schemes.get(name)
    }

    /// Get all available color scheme names
    pub fn scheme_names(&self) -> Vec<String> {
        self.schemes.keys().cloned().collect()
    }

    /// Add default color schemes
    fn add_default_schemes(&mut self) {
        // Default color scheme
        let mut default = ColorScheme::new("Default");
        default.description = "Default Dasher color scheme".to_string();
        
        // Add some sensible default colors with good contrast
        default.add_color_pair(
            Color::from_hex("#000000").unwrap(), // Black text
            Color::from_hex("#FFFFFF").unwrap()  // White background
        );
        default.add_color_pair(
            Color::from_hex("#FFFFFF").unwrap(), // White text
            Color::from_hex("#0000FF").unwrap()  // Blue background
        );
        default.add_color_pair(
            Color::from_hex("#FFFFFF").unwrap(), // White text
            Color::from_hex("#FF0000").unwrap()  // Red background
        );
        default.add_color_pair(
            Color::from_hex("#000000").unwrap(), // Black text
            Color::from_hex("#FFFF00").unwrap()  // Yellow background
        );
        
        self.add_scheme(default);

        // High contrast scheme
        let mut high_contrast = ColorScheme::new("High Contrast");
        high_contrast.description = "High contrast color scheme for better visibility".to_string();
        
        high_contrast.add_color_pair(
            Color::from_hex("#FFFFFF").unwrap(), // White text
            Color::from_hex("#000000").unwrap()  // Black background
        );
        high_contrast.add_color_pair(
            Color::from_hex("#000000").unwrap(), // Black text
            Color::from_hex("#FFFF00").unwrap()  // Yellow background
        );
        
        self.add_scheme(high_contrast);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex() {
        let color = Color::from_hex("#FF0000").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 0);
        assert_eq!(color.b, 0);

        let color = Color::from_hex("00FF00").unwrap();
        assert_eq!(color.r, 0);
        assert_eq!(color.g, 255);
        assert_eq!(color.b, 0);
    }

    #[test]
    fn test_color_contrast() {
        let white = Color::new(255, 255, 255);
        let black = Color::new(0, 0, 0);
        let gray = Color::new(120, 120, 120);

        assert!(white.has_good_contrast_with(&black));
        assert!(!gray.has_good_contrast_with(&white));
        assert!(!gray.has_good_contrast_with(&black));
    }

    #[test]
    fn test_color_scheme() {
        let mut scheme = ColorScheme::new("Test");
        let fg = Color::new(0, 0, 0);
        let bg = Color::new(255, 255, 255);
        scheme.add_color_pair(fg, bg);

        let (fg2, bg2) = scheme.get_color_pair(0).unwrap();
        assert_eq!(fg, fg2);
        assert_eq!(bg, bg2);
    }

    #[test]
    fn test_color_manager() {
        let manager = ColorManager::new();
        assert!(manager.get_scheme("Default").is_some());
        assert!(manager.get_scheme("High Contrast").is_some());
        assert!(manager.get_scheme("NonExistent").is_none());
    }
}
