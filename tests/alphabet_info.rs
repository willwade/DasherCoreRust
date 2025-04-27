use dasher_core::alphabet::{AlphabetInfo, Character};

#[test]
fn test_alphabet_info() {
    let mut info = AlphabetInfo::new("test".to_string());
    info.characters.push(Character {
        display: "_".to_string(),
        text: " ".to_string(),
        ..Default::default()
    });
    info.characters.push(Character {
        display: "a".to_string(),
        text: "a".to_string(),
        ..Default::default()
    });
    assert!(info.is_space_character(1));
    assert!(!info.is_space_character(2));
    assert_eq!(info.get_display_text(1), Some("_"));
    assert_eq!(info.get_text(1), Some(" "));
}
