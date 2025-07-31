#[test]
fn test_iso_detect() {
    use magical_rs::magical::bytes_read::DEFAULT_MAX_BYTES_READ;
    use magical_rs::magical::{
        bytes_read::{read_file_header, with_bytes_read},
        magic::FileKind,
    };

    let iso_file = "tests/2.iso";
    let bytes_max = with_bytes_read();
    let wrong_max = DEFAULT_MAX_BYTES_READ;

    let header_bytes = read_file_header(iso_file, bytes_max).unwrap();
    let wrong_header = read_file_header(iso_file, wrong_max).unwrap();

    assert_eq!(FileKind::match_types(&header_bytes), FileKind::ISO);
    assert_ne!(FileKind::match_types(&wrong_header), FileKind::ISO);
}
