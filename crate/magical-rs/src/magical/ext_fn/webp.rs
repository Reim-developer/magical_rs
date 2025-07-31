const RIFF: &[u8; 4] = b"RIFF";
const WEBP: &[u8; 4] = b"WEBP";

#[must_use]
pub fn is_webp(bytes: &[u8]) -> bool {
    if bytes.len() < 12 {
        return false;
    }

    if &bytes[0..4] != RIFF {
        return false;
    }

    if &bytes[8..12] != WEBP {
        return false;
    }

    let file_size = match bytes.get(4..8) {
        Some([a, b, c, d]) => u32::from_le_bytes([*a, *b, *c, *d]) as usize,
        _ => return false,
    };

    file_size > 4
}
