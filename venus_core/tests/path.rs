#[test]
fn test_is_file_exists() {
    use venus_core::utils::path::FilePath;
    assert!(FilePath::new_with_path("Cargo.toml").is_file_exists());
}

#[test]
fn test_get_abs_path() {
    use venus_core::utils::path::FilePath;

    let abs_path = FilePath::new_with_path("Cargo.toml").get_abs_path();
    assert!(abs_path.is_some());
}

#[test]
fn test_get_user_home() {
    use venus_core::utils::path::get_user_home;

    assert!(get_user_home().is_some());
}
