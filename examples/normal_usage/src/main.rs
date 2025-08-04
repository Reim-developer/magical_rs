use magical_rs::magical::bytes_read::{read_file_header, with_bytes_read};
use magical_rs::magical::magic::FileKind;

/*
* A simple example function to detect 'PNG' file.
*/
fn detect_png() {
    /* Under normal circumstances. Use
     * `DEFAULT_MAX_BYTES_READ` is enough:
     *
     * let max_bytes_read = DEFAULT_MAX_BYTES_READ;
     */

    /* Howerver, you should use `with_bytes_read`
     * if you don't care about the number of bytes to
     * read and leave everything automatic.
     */
    let max_bytes_read = with_bytes_read();
    let bytes = read_file_header("img/1.png", max_bytes_read).unwrap();

    /*
     * The result returned is Option<FileKind>.
     * If you are not sure the return type is not None,
     * use match., like:
     *
     *  let kind = FileKind::match_types(&bytes);
     *  match kind {
     *       Some(k) => println!("{k:?}"),
     *       None => println!("Could not detect any files"),
     *   }
     */
    let kind = FileKind::match_types(&bytes).unwrap();

    println!("{kind:?}");
}

/*
* A simple function to detect the ISO file format
*/
fn detect_iso() {
    /*
     * In this case, you should use with_bytes_read
     * to ensure result is always correct.
     * For more information, please read:
     * https://docs.rs/magical_rs/0.1.2/magical_rs/#warning-use-with_bytes_read-for-correct-detection
     */
    let max_byte_read = with_bytes_read();

    let bytes = read_file_header("img/2.iso", max_byte_read).unwrap();

    match FileKind::match_types(&bytes) {
        Some(k) => println!("{k:?}"),
        None => println!("Could not detect ISO file."),
    }
}

fn main() {
    detect_png(); /* PNG */
    detect_iso(); /* ISO */
}
