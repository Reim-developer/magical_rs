#[cfg(feature = "magical_dyn")]
#[test]
fn test_dyn_magic_matches() {
    use magical_rs::magical::dyn_magic::DynMagicCustom;

    let rule_fn = |bytes: &[u8]| bytes.starts_with(b"Shoujo");
    let rule = DynMagicCustom::new(rule_fn, "Magical", 32);

    assert!(rule.matches(b"Shoujo<3"));
    assert!(!rule.matches(b"Not Shoujo Here..."));
}

#[cfg(feature = "magical_dyn")]
#[test]
fn test_dyn_magic_downcast() {
    use magical_rs::magical::dyn_magic::DynMagicCustom;

    let rule_fn = |bytes: &[u8]| bytes.len() >= 3 && &bytes[0..3] == b"BIN";
    let rule = DynMagicCustom::new(rule_fn, String::from("BINARY_FORMAT"), 64);

    let kind = rule.kind();
    let as_string = kind.downcast_ref::<String>();

    assert_eq!(as_string, Some(&"BINARY_FORMAT".to_string()));
}

#[cfg(feature = "magical_dyn")]
#[test]
fn test_match_dyn_types_all() {
    use magical_rs::magical::dyn_magic::{DynMagicCustom, match_dyn_types_all};

    let rules = vec![
        DynMagicCustom::new(|byte| byte.contains(&b'X'), "has_x", 32),
        DynMagicCustom::new(|byte| byte.contains(&b'Y'), "has_y", 32),
        DynMagicCustom::new(|byte| byte.contains(&b'Z'), "has_z", 32),
    ];

    let data = b"XYZ";
    let matches = match_dyn_types_all(data, &rules);

    assert_eq!(matches.len(), 3);

    let types: Vec<&str> = matches
        .iter()
        .map(|any| (**any).downcast_ref::<&str>().unwrap())
        .copied()
        .collect();

    assert!(types.contains(&"has_x"));
    assert!(types.contains(&"has_y"));
    assert!(types.contains(&"has_z"));
}

#[cfg(feature = "magical_dyn")]
#[test]
fn test_dyn_detect_file() {
    use magical_rs::magical::dyn_magic::{DynMagicCustom, match_dyn_types_as};

    let rules = vec![
        DynMagicCustom::new(|bytes: &[u8]| bytes.starts_with(b"PNG"), "image/png", 8),
        DynMagicCustom::new(
            |bytes: &[u8]| bytes.starts_with(b"Shoujo"),
            String::from("MagicalGirl"),
            6969,
        ),
    ];

    let data = b"Shoujo";
    let result = match_dyn_types_as::<String>(data, &rules).unwrap();

    assert_eq!(result, &"MagicalGirl".to_string());
}
