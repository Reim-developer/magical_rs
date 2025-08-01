#[test]
#[cfg(not(feature = "no_std"))]
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
    assert_ne!(FileKind::match_types(&header_bytes), FileKind::Unknown);
    assert_ne!(FileKind::match_types(&wrong_header), FileKind::ISO);
    assert_eq!(FileKind::match_types(&wrong_header), FileKind::Unknown);
}

#[test]
#[cfg(not(feature = "no_std"))]
fn test_png_detect() {
    use magical_rs::magical::{
        bytes_read::{read_file_header, with_bytes_read},
        magic::FileKind,
    };

    let png_file = "tests/1.png";
    let bytes_max = with_bytes_read();
    let header_bytes = read_file_header(png_file, bytes_max).unwrap();

    assert_eq!(FileKind::match_types(&header_bytes), FileKind::Png);
    assert_ne!(FileKind::match_types(&header_bytes), FileKind::Unknown);
}

#[test]
#[cfg(not(feature = "no_std"))]
fn test_class_detect() {
    use magical_rs::magical::{
        bytes_read::{read_file_header, with_bytes_read},
        magic::FileKind,
    };

    let png_file = "tests/3.class";
    let bytes_max = with_bytes_read();
    let header_bytes = read_file_header(png_file, bytes_max).unwrap();

    assert_eq!(FileKind::match_types(&header_bytes), FileKind::Class);
    assert_ne!(FileKind::match_types(&header_bytes), FileKind::Unknown);
}

#[test]
#[cfg(not(feature = "no_std"))]
fn test_webp_detect() {
    use magical_rs::magical::{
        bytes_read::{read_file_header, with_bytes_read},
        magic::FileKind,
    };

    let webp_file = "tests/4.webp";
    let bytes_max = with_bytes_read();
    let header_bytes = read_file_header(webp_file, bytes_max).unwrap();

    assert_eq!(FileKind::match_types(&header_bytes), FileKind::WEBP);
    assert_ne!(FileKind::match_types(&header_bytes), FileKind::Unknown);
}
