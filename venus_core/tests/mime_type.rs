use std::{
    fs::File,
    io::{BufReader, Read},
};

#[derive(Debug)]
enum FileKind {
    Png,
}

const MAX_BYTES_READ: usize = 2048;

const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
static FILE_KIND: &[(&[u8], FileKind)] = &[(PNG_SIGNATURE, FileKind::Png)];

fn read_file_header(file_path: &str) -> Vec<u8> {
    let file = File::open(file_path).unwrap();
    let mut reader = BufReader::new(file);
    let mut buffer = vec![0u8; MAX_BYTES_READ];

    let _ = reader.read(&mut buffer).unwrap();

    buffer
}

#[test]
fn test_mime_type() {
    use std::path;

    let abs_path = path::absolute("tests/1.png")
        .unwrap()
        .to_string_lossy()
        .to_string();

    let file_data = read_file_header(&abs_path);
    assert!(file_data.starts_with(PNG_SIGNATURE));

    for &(signature, ref file_kind) in FILE_KIND {
        assert!(
            !signature.is_empty(),
            "Empty signature detect in : {file_kind:?}",
        );
    }
}
