use dasher_core::alphabet::{Color, ColorScheme, ColorManager};

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
    let gray_dark = Color::new(100, 100, 100);
    let gray_light = Color::new(180, 180, 180);
    assert!(white.has_good_contrast_with(&black));
    assert!(!gray_light.has_good_contrast_with(&white));
    assert!(!gray_dark.has_good_contrast_with(&black));
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
