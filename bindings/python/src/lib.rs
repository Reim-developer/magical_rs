//! Native Python bindings for [`magical_rs`].
//!
//! The compiled module is deliberately small. It exposes only the primitives
//! the Python layer needs, and returns plain strings rather than wrapper
//! objects, because the rich `FileKind` enum is declared on the Python side.
//! See `python/magical_py/_kinds.py`.
//!
//! Custom rules are split the same way. Level 1 needs this module because the
//! signature table lives in Rust. Level 2 needs it for one primitive,
//! [`signatures_match`], so that matching a declarative rule costs no Python
//! call; levels 3 and 4 need nothing here, because their matchers are Python
//! callables and the work is in the Python layer. See
//! `python/magical_py/_levels.py`.
//!
//! I/O errors are surfaced as Python's own `OSError` subclasses rather than a
//! library-specific exception, so `detect("missing.png")` raising
//! `FileNotFoundError` behaves the way callers already expect from `open()`.
//!
//! [`magical_rs`]: https://docs.rs/magical_rs

use {
    magical_rs::magical::{
        bytes_read::{
            DEFAULT_MAX_BYTES_READ, DEFAULT_OFFSET, ISO_MAX_BYTES_READ, ISO_OFFSETS,
            TAR_MAX_BYTES_READ, TAR_OFFSETS, max_bytes, read_file_header, with_bytes_read,
        },
        magic::FileKind,
        match_rules::MatchRules,
        signatures::SIGNATURE_KIND,
    },
    pyo3::{
        exceptions::{
            PyFileNotFoundError, PyIsADirectoryError, PyOSError, PyPermissionError, PyValueError,
        },
        prelude::*,
    },
    std::io,
};

/// Translates an I/O failure into the closest matching Python exception.
///
/// Mapping to the built-in subclasses rather than a custom error type is what
/// lets callers write ordinary `except FileNotFoundError` or `except
/// PermissionError` around a `detect()` call.
fn io_error_to_py(error: &io::Error) -> PyErr {
    let message = error.to_string();
    match error.kind() {
        io::ErrorKind::NotFound => PyFileNotFoundError::new_err(message),
        io::ErrorKind::PermissionDenied => PyPermissionError::new_err(message),
        io::ErrorKind::IsADirectory => PyIsADirectoryError::new_err(message),
        _ => PyOSError::new_err(message),
    }
}

/// Defines the `FileKind` to variant-name mapping, and back.
///
/// `stringify!` is what makes this worth doing: each variant's name is
/// written exactly once, and the returned string is derived from it. Writing
/// the arms out longhand would have meant typing every name twice, and a
/// typo in the string half would not have been caught by the compiler — it
/// would only have shown up as a `KeyError` in Python at runtime.
///
/// Both directions come from the one list, because the reverse is what lets
/// Python name a format and have the table looked up. `FileKind` is not
/// `#[repr]`-stable and carries no `FromStr`, so a second hand-written table
/// would be a second place for a name to be wrong.
macro_rules! define_kinds {
    ($($variant:ident),+ $(,)?) => {
        const fn file_kind_name(kind: FileKind) -> &'static str {
            match kind {
                $(FileKind::$variant => stringify!($variant),)+
            }
        }

        // Not `const`: matching on `&str` needs `PartialEq` in const context,
        // which is not stable yet. Nothing here is called at compile time.
        fn file_kind_from_name(name: &str) -> Option<FileKind> {
            match name {
                $(stringify!($variant) => Some(FileKind::$variant),)+
                _ => None,
            }
        }
    };
}

define_kinds!(
    _8BPS,
    AceCompressed,
    Aiff,
    Amr,
    AppleDiskImage,
    AppleIconImage,
    Arj,
    Asf,
    AuAudioFileFormat,
    Avro,
    BinaryPlist,
    Bitmap,
    BitTorrent,
    BLENDER,
    Bzip,
    Cabinet,
    Chm,
    Class,
    CoreAudio,
    Cpio,
    CreativeVoiceFile,
    Cursor,
    Dalvik,
    Dds,
    Deb,
    Djvu,
    DoomWad,
    ELF,
    FbxBinary,
    Fits,
    Flac,
    FlashVideo,
    FontCollection,
    Gguf,
    GIF,
    GimpXcf,
    GltfBinary,
    GoogleChromeExtension,
    Gzip,
    Hdf5,
    ICO,
    ISO,
    IsoMedia,
    JPEG2000,
    JpegXl,
    Jpg,
    Ktx,
    Ktx2,
    Lua,
    Lz4,
    Lzh,
    MachO,
    Matlab,
    MatroskaMediaContainer,
    Midi,
    Mobipocket,
    ModuleForEvenvironmentModules,
    MonkeyAudio,
    MP3,
    MpegProgramStream,
    MSDOS,
    NoodlesoftHazel,
    Numpy,
    OGG,
    OleCompoundFile,
    OpenExr,
    OpenGLIrisPerformer,
    OpenTypeFont,
    Orc,
    Par2,
    Parquet,
    Pcap,
    PcapNg,
    Pcx,
    PDF,
    PhotoCapTemplate,
    Pickle,
    PkgZip,
    Ply,
    Png,
    PostScript,
    Qcow,
    Qcow2,
    Radiance,
    RAR,
    RData,
    RichTextFormat,
    RPM,
    ScriptExecute,
    SerializedJavaData,
    SevenZip,
    Slob,
    SQLite,
    Stuffit,
    StuffitSit,
    Swf,
    Tar,
    Tiff,
    TrueTypeFont,
    VBScriptEncoded,
    VirtualBoxVdi,
    VirtualHd,
    Vmdk,
    WASM,
    WavPack,
    WEBP,
    WindowImagingFormat,
    WindowsShortcut,
    Woff,
    Woff2,
    XML,
    Xz,
    Zlib,
    Zstd,
);

/// Classifies `data` using only the rules that fit inside `limit` bytes.
///
/// This is `FileKind::match_with_max_read_rule`, which the crate compiles only
/// under `not(feature = "std")` — not the configuration this module builds in.
/// `SIGNATURE_KIND` and `Magic::matches` are both public, so the filter is
/// reproduced here rather than the crate's public API being widened to cover a
/// case only the `no_std` build happened to have. This is the same trade
/// `signatures_match` makes, and the same price: eight lines that have to stay
/// in step, pinned by a test on each side rather than trusted.
///
/// `limit` is the window the caller claims to hold, not a length `data` is
/// checked against. A 100-byte file classified with a 2,048-byte window is
/// ordinary, and both this and the crate's version answer it the same way.
fn match_within(data: &[u8], limit: usize) -> Option<FileKind> {
    SIGNATURE_KIND
        .iter()
        .filter(|magic| magic.max_bytes_read <= limit)
        .find(|magic| magic.matches(data))
        .map(|magic| magic.kind)
}

/// The classification step both entry points share.
///
/// Unbounded by default, which is what `FileKind::match_types` does and what
/// this module has always done. A limit narrows the table to the rules whose
/// own `max_bytes_read` fits, so a `None` down that path means "nothing
/// reachable inside the window you named", which is a different statement from
/// the unbounded `None` and the reason the limit is worth having.
fn classify(data: &[u8], max_bytes_read: Option<usize>) -> Option<&'static str> {
    let kind = max_bytes_read.map_or_else(
        || FileKind::match_types(data),
        |limit| match_within(data, limit),
    );
    kind.map(file_kind_name)
}

/// Returns the `FileKind` variant name for the leading bytes of a file.
///
/// Reads `max_bytes_read` of them, or `with_bytes_read()` if no limit is
/// given, so the read is the one the classification is then made over. Asking
/// for a small read makes the read itself smaller, not only the search.
///
/// # Errors
///
/// Returns a Python `OSError` subclass if the file cannot be opened or read.
#[pyfunction]
#[pyo3(signature = (path, /, *, max_bytes_read=None))]
fn detect_path(path: &str, max_bytes_read: Option<usize>) -> PyResult<Option<&'static str>> {
    let header = read_file_header(path, max_bytes_read.unwrap_or_else(with_bytes_read))
        .map_err(|error| io_error_to_py(&error))?;
    Ok(classify(&header, max_bytes_read))
}

/// Returns the `FileKind` variant name for an in-memory buffer.
#[pyfunction]
#[pyo3(signature = (data, /, *, max_bytes_read=None))]
fn detect_bytes(data: &[u8], max_bytes_read: Option<usize>) -> Option<&'static str> {
    classify(data, max_bytes_read)
}

/// Reads up to `max_bytes` bytes from the start of a file, detecting nothing.
///
/// This is the crate's `read_file_header` over the `std` boundary. It is here
/// so a caller that wants to size its own read has a supported way to do it
/// rather than the `open`/`read`/`close` the examples had to teach, and so the
/// figure `bytes_read()` reports can be spent instead of only quoted. The
/// Python layer reads a file object in Python, since a stream has no path to
/// hand over; see `python/magical_py/__init__.py`.
///
/// # Errors
///
/// Returns a Python `OSError` subclass if the file cannot be opened or read.
#[pyfunction]
#[pyo3(signature = (path, /, *, max_bytes=None))]
fn read_header(path: &str, max_bytes: Option<usize>) -> PyResult<Vec<u8>> {
    read_file_header(path, max_bytes.unwrap_or_else(with_bytes_read))
        .map_err(|error| io_error_to_py(&error))
}

/// Returns every `FileKind` variant name the detection table can produce.
///
/// This exists so the test suite can assert the Python enum has not fallen
/// out of step with the Rust table.
#[pyfunction]
#[pyo3(signature = ())]
fn all_kinds() -> Vec<&'static str> {
    SIGNATURE_KIND
        .iter()
        .map(|magic| file_kind_name(magic.kind))
        .collect()
}

/// Returns the table's read sizes and offsets as the crate declares them.
///
/// The crate's constants are `pub` and mean things a caller cannot derive: the
/// furthest signature sits at offset 36,865, and every rule declares a read
/// size of at least 2,048, so a window below that excludes PNG along with
/// everything else. `bytes_read` reports the total, but nothing in Python could
/// say which formats a smaller window would drop.
///
/// A tuple rather than a dict, matching [`TableRow`]: the values differ in type
/// — the offsets are lists and the rest are integers — and a `dict[str,
/// object]` would hand every caller six casts. A named struct in Rust would
/// cross the boundary as an opaque wrapper, which is the thing this module
/// avoids elsewhere.
#[pyfunction]
#[pyo3(signature = ())]
fn read_limits() -> (
    usize,
    usize,
    Vec<usize>,
    usize,
    Vec<usize>,
    usize,
) {
    (
        DEFAULT_MAX_BYTES_READ,
        DEFAULT_OFFSET,
        ISO_OFFSETS.to_vec(),
        ISO_MAX_BYTES_READ,
        TAR_OFFSETS.to_vec(),
        TAR_MAX_BYTES_READ,
    )
}

/// Returns the read size an offset and signature require, as `max_bytes` does.
///
/// The crate's `max_bytes` is a `const fn` over slices, so it is not callable
/// from Python. It is the arithmetic behind `ISO_MAX_BYTES_READ` and
/// `TAR_MAX_BYTES_READ`, and it is what a caller needs to size a read for a
/// level 2 rule of their own.
// `offsets` is taken by value because that is the only form pyo3 can extract
// for a `Vec`, the same reason `signatures_match` takes its two `Vec`s. The
// vector is borrowed for the call rather than moved, so nothing is consumed.
#[allow(clippy::needless_pass_by_value)]
#[pyfunction]
#[pyo3(signature = (offsets, signature_len, /))]
fn max_bytes_read_for(offsets: Vec<usize>, signature_len: usize) -> usize {
    // The crate takes the signature to measure it. Only its length is used, so
    // a zeroed buffer of the right size is passed rather than widening the
    // crate's signature to take a length.
    let probe = vec![0u8; signature_len];
    max_bytes(&offsets, &probe)
}

/// One row of the detection table, as the Python layer receives it.
///
/// A tuple rather than a class: the extension returns plain data and the
/// Python side wraps it, which is why `FileKind` is declared in Python too.
/// Named here so the four places that build one agree.
type TableRow = (String, Vec<Vec<u8>>, Vec<usize>, usize, bool);

/// Flattens one `Magic` into a [`TableRow`].
///
/// A format matched by a function rather than a fixed pattern — `ScriptExecute`
/// and `WEBP` — reports no signatures and `uses_predicate` true, because there
/// are no bytes to report and inventing some would misdescribe how the rule
/// decides.
fn table_row(magic: &magical_rs::magical::signatures::Magic) -> TableRow {
    let uses_predicate = matches!(magic.rules, MatchRules::WithFn(_));
    let signatures = if uses_predicate {
        Vec::new()
    } else {
        magic.signatures.iter().map(|signature| signature.to_vec()).collect()
    };

    (
        file_kind_name(magic.kind).to_string(),
        signatures,
        magic.offsets.to_vec(),
        magic.max_bytes_read,
        uses_predicate,
    )
}

/// Returns one table entry, so a caller can ask why a format is or is not
/// matched.
///
/// `(kind, signatures, offsets, max_bytes_read, uses_predicate)`. The
/// signatures are the raw bytes at `offsets`.
#[pyfunction]
#[pyo3(signature = (kind, /))]
fn signature_of(kind: &str) -> Option<TableRow> {
    file_kind_from_name(kind)
        .and_then(|wanted| SIGNATURE_KIND.iter().find(|magic| magic.kind == wanted))
        .map(table_row)
}

/// Reports whether one named format's rule matches `data`, ignoring the rest
/// of the table.
///
/// `detect` answers "what is this file", and it answers with the first entry
/// that matched, so a format late in the table is invisible to a buffer that an
/// earlier entry also matches. This asks the narrower question: would this one
/// format's own rule match. `Zlib` is the case that matters, since `78 9C` is
/// any deflate stream and the entry is deliberately ordered last for that
/// reason.
///
/// A name the table does not contain is a `ValueError` rather than `False`,
/// because "no such format" and "the format did not match" are different
/// answers and a typo in a format name should not read as a negative result.
#[pyfunction]
#[pyo3(signature = (kind, data, /))]
fn kind_matches(kind: &str, data: &[u8]) -> PyResult<bool> {
    let Some(magic) = file_kind_from_name(kind)
        .and_then(|wanted| SIGNATURE_KIND.iter().find(|magic| magic.kind == wanted))
    else {
        return Err(PyValueError::new_err(format!(
            "{kind} is not a format the detection table can produce"
        )));
    };

    Ok(magic.matches(data))
}

/// Returns every table entry, in the order `detect` tries them.
///
/// The order is the answer to "why did my file come back as this", so it is
/// carried through rather than sorted: a caller that walks the list stops at
/// the first match and gets the same answer `detect` gives.
#[pyfunction]
#[pyo3(signature = ())]
fn signature_table() -> Vec<TableRow> {
    SIGNATURE_KIND.iter().map(table_row).collect()
}

/// The number of bytes needed to detect every format in the table.
///
/// The furthest signature sits at offset 36,865, so a header shorter than
/// this cannot be classified reliably.
#[pyfunction]
#[pyo3(signature = ())]
fn bytes_read() -> usize {
    with_bytes_read()
}

/// Reports whether any signature appears at any of the offsets in `data`.
///
/// This mirrors the `CustomMatchRules::Default` arm of
/// `magical_rs::magical::magic_custom::MagicCustom::matches_custom`, which is
/// the crate's own definition of level 2 matching and the one this function
/// re-exports. The crate cannot be called directly here: `MagicCustom` holds
/// `&'static [&'static [u8]]` and `&'static [usize]`, because a level 2 rule is
/// meant to be a `static`. A rule built from Python data at run time would have
/// to be `Box::leak`ed, and a process that builds rules in a loop would leak
/// without bound. Copying the comparison instead keeps the cost in Rust — which
/// is what separates level 2 from level 3, where the matcher is a Python
/// callable — at the price of eight lines that have to stay in step.
///
/// `tests/magic_custom.rs` in the main crate pins that arm's behaviour, and
/// `tests/test_levels.py` pins this function against every branch of it, so a
/// change on either side shows up as a test failure rather than as a detection
/// that quietly stops working.
///
/// The addition saturates, as the crate's does since `0.6.2`. An offset near
/// `usize::MAX` used to be a debug panic here and a release slice index there;
/// it is now a no-match on both sides. Offsets arrive as Python integers, which
/// can be arbitrarily large and are not the crate's to trust, so this is the
/// more likely place for one to be nonsense. `tests/test_levels.py` asserts it.
// The two `Vec`s are taken by value because that is the only form pyo3 can
// extract: `&[Vec<u8>]` does not implement `PyFunctionArgument`, so the lint's
// suggested `&[Vec<u8>]` cannot be written here. One extraction into owned
// values is also cheaper than walking the Python sequence item by item.
#[allow(clippy::needless_pass_by_value)]
#[pyfunction]
#[pyo3(signature = (data, signatures, offsets, /))]
fn signatures_match(data: &[u8], signatures: Vec<Vec<u8>>, offsets: Vec<usize>) -> bool {
    signatures.iter().any(|signature| {
        let signature = signature.as_slice();
        offsets.iter().any(|&offset| {
            let end = offset.saturating_add(signature.len());
            data.len() >= end && data.get(offset..end) == Some(signature)
        })
    })
}

#[pymodule]
#[pyo3(name = "_magical_rs")]
fn _magical_rs(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add_function(wrap_pyfunction!(detect_path, module)?)?;
    module.add_function(wrap_pyfunction!(detect_bytes, module)?)?;
    module.add_function(wrap_pyfunction!(read_header, module)?)?;
    module.add_function(wrap_pyfunction!(all_kinds, module)?)?;
    module.add_function(wrap_pyfunction!(bytes_read, module)?)?;
    module.add_function(wrap_pyfunction!(read_limits, module)?)?;
    module.add_function(wrap_pyfunction!(max_bytes_read_for, module)?)?;
    module.add_function(wrap_pyfunction!(signature_of, module)?)?;
    module.add_function(wrap_pyfunction!(signature_table, module)?)?;
    module.add_function(wrap_pyfunction!(kind_matches, module)?)?;
    module.add_function(wrap_pyfunction!(signatures_match, module)?)?;
    Ok(())
}
