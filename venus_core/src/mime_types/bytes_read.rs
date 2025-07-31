use std::fs::File;
use std::io;
use std::io::{BufReader, Read};

/*
* File offsets defined !
*/

pub const DEFAULT_MAX_BYTES_READ: usize = 2048;

pub const DEFAULT_OFFSET: usize = 0;
pub const ISO_OFFSETS: &[usize] = &[32769, 34817, 36865];
pub const TAR_OFFSETS: &[usize] = &[257];
pub const ISO_MAX_BYTES_READ: usize = max_bytes(ISO_OFFSETS, b"CD001");
pub const TAR_MAX_BYTES_READ: usize = max_bytes(TAR_OFFSETS, b"ustar");

#[must_use]
pub const fn max_bytes(offsets: &[usize], signature: &[u8]) -> usize {
    let mut max_offset = 0;
    let mut index = 0;

    while index < offsets.len() {
        if offsets[index] > max_offset {
            max_offset = offsets[index];
        }

        index += 1;
    }

    max_offset + signature.len()
}

/// Read the file header with limit caption as `2048 KB`.
///
/// # Errors
/// `File::open` failed.
/// `Buffer Reader` failed.
pub fn read_file_header(file_path: &str, max_bytes: usize) -> Result<Vec<u8>, io::Error> {
    let file = File::open(file_path)?;
    let mut reader = BufReader::new(file);

    let max_bytes_read = max_bytes;
    let mut buffer = vec![0u8; max_bytes_read];

    let mut total_read: usize = 0;

    while total_read < max_bytes_read {
        match reader.read(&mut buffer[total_read..]) {
            Ok(0) => break, /* EOF */
            Ok(index) => total_read = total_read.saturating_add(index),
            Err(error) => return Err(error),
        }
    }

    Ok(buffer)
}
