use dasher_core::alphabet::{AlphabetMap, AlphabetInfo};

fn create_test_alphabet() -> AlphabetInfo {
    let mut info = AlphabetInfo::new("test".to_string());
    info.characters.extend_from_slice(&[
        dasher_core::alphabet::Character {
            display: "a".to_string(),
            text: "a".to_string(),
            ..Default::default()
        },
        dasher_core::alphabet::Character {
            display: "b".to_string(),
            text: "b".to_string(),
            ..Default::default()
        },
        dasher_core::alphabet::Character {
            display: "_".to_string(),
            text: " ".to_string(),
            ..Default::default()
        },
        dasher_core::alphabet::Character {
            display: "ch".to_string(),
            text: "ch".to_string(),
            ..Default::default()
        },
    ]);
    info
}

#[test]
fn test_alphabet_map() {
    let info = create_test_alphabet();
    let map = AlphabetMap::new(info);
    assert_eq!(map.char_to_index('a'), Some(1));
    assert_eq!(map.string_to_index("ch"), Some(4));
    assert!(map.is_space(3));
    assert_eq!(map.text_to_symbols("a ch b"), vec![1, 3, 4, 3, 2]);
    assert_eq!(map.symbols_to_text(&[1, 3, 4, 3, 2]), "a ch b");
}
