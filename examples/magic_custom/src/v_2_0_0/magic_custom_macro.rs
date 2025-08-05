/*
* In version `0.2.0 and` later of `magical_rs`, you can now use
* macros, specifically with the `MagicCustom` module.
* Here are the basic instructions for using macros,
* and how to use some new functions in this version.
*/

use magical_rs::{all_matches, any_matches, magic_custom, match_custom};

/*
* We can use any name for the enum.
* Here is 'FileKind'
*/
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FileKind {
    Shoujo,
    UnknownFallback,
}

fn is_shoujo(bytes: &[u8]) -> bool {
    bytes.starts_with(b"Magic!")
}

fn is_not_shoujo(bytes: &[u8]) -> bool {
    !bytes.starts_with(b"Magic!")
}

/*
* With this function, as you can see, even though it seems like
* there is a condition that `is_not_shoujo` return false.
* we still get the result `Shoujo`
*/
pub fn magic_custom_any() {
    let rule = magic_custom! (
        signatures: [b""],
        offsets: [0],
        max_bytes_read: 2451,
        kind: FileKind::Shoujo,
        rules: any_matches!(is_shoujo, is_not_shoujo)
    );

    let result = match_custom! (
        bytes: b"Magic!",
        rules: [rule],
        fallback: FileKind::UnknownFallback
    );

    assert_eq!(result, FileKind::Shoujo);
    assert_ne!(result, FileKind::UnknownFallback);
}

/*
* But in this function, just one condition return false.
* All will return fallback.
*/
pub fn magic_custom_all() {
    let rule = magic_custom! (
        signatures: [b"Magic!"],
        offsets: [0],
        max_bytes_read: 2451,
        kind: FileKind::Shoujo,
        rules: all_matches!(is_shoujo, is_not_shoujo)
    );

    let result = match_custom! (
        bytes: b"Magic!",
        rules: [rule],
        fallback: FileKind::UnknownFallback
    );

    assert_eq!(result, FileKind::UnknownFallback);
    assert_ne!(result, FileKind::Shoujo);
}
