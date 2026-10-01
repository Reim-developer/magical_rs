//! One [`Adapter`] per library, and the two passes a benchmark makes.
//!
//! An adapter is a thin, boring wrapper: it calls the library the way a caller
//! would and hands back a comparable shape. There is no caching, no
//! pre-normalisation and no early exit that a caller could not have written,
//! because the only thing an adapter is allowed to do is be the library.
//!
//! # Two entry points, because a caller has two
//!
//! [`Adapter::detect`] takes a `&[u8]` the caller already has. [`Adapter::detect_path`]
//! takes a path and lets the library open the file itself. Both are timed, and
//! both tables are printed, because they are different questions with different
//! answers and quoting only the first is how a pure-Rust in-memory crate ends up
//! looking like it beats a C library at a job neither of them is doing.
//!
//! The from-path numbers include opening and reading the file, so they are
//! dominated by I/O for all three libraries and the gap between them is mostly
//! how much each one reads. That is a real cost a real caller pays, and the
//! report says so rather than leaving a reader to assume the second table is
//! measuring detection.
//!
//! # Why `usize`
//!
//! [`Adapter::detect`] returns the length of the answer the caller receives, so
//! the cost of *having* the answer is inside the measurement for all three
//! libraries and none of them is charged for an allocation the other two do not
//! make. A library that finds nothing returns `0`. It is a `usize` rather than
//! a `String` because a measured call that allocated would be measuring this
//! crate's allocator as much as the library, and interning the answer to avoid
//! that would be measuring a hash lookup instead -- which at five nanoseconds is
//! a worse distortion than the allocation it was avoiding.

use std::path::Path;

use crate::corpus::READ_SIZE;

use magical_rs::magical::bytes_read::read_file_header;
use magical_rs::magical::magic::FileKind;

/// What a library said about a buffer. Never compared, only printed.
pub struct Answer {
    /// The identifier this library uses for the format it found.
    pub label: String,
    /// The registered MIME type, if it registers one.
    pub mime: Option<String>,
    /// The conventional extension, if it has one.
    pub extension: Option<String>,
}

impl Answer {
    /// Nothing matched.
    ///
    /// Public because a test adapter needs to say it too, and because a caller
    /// writing their own adapter for their own reasons should not have to build
    /// the struct by hand to answer "nothing".
    #[must_use]
    pub fn none() -> Self {
        Self {
            label: "-".to_owned(),
            mime: None,
            extension: None,
        }
    }
}

/// A one-time cost a library pays before its first detection.
///
/// Reported on its own row rather than folded into the per-call number, because
/// folding it in would be a lie in whichever direction it happened to fall: a
/// database that takes 15 ms to load is not a slow detector, and a library that
/// loads nothing is not a fast one.
pub struct Startup {
    /// What was done.
    pub what: &'static str,
    /// How long it took, in milliseconds.
    pub millis: f64,
    /// Anything else worth saying about it, such as how big the data was.
    pub note: Option<String>,
}

/// A library under measurement.
pub trait Adapter {
    /// The name every table uses.
    fn name(&self) -> &'static str;

    /// The header size this library's own API asks a caller for.
    ///
    /// A [`String`] rather than a `usize` because the honest answers are not all
    /// numbers: `infer` has no documented limit at all, and printing `36870`
    /// next to it would be inventing one. The column exists so a reader can see
    /// the three were not configured identically -- which they are not, and which
    /// is the single biggest reason not to read a speed ratio as a verdict.
    fn read_size(&self) -> String;

    /// The measured in-memory call: identifies a format, returns the length of
    /// its answer.
    fn detect(&self, bytes: &[u8]) -> usize;

    /// The measured call from a file on disk.
    ///
    /// Includes whatever the library does to get the bytes, which is the point:
    /// this is what a caller who has a path actually pays.
    fn detect_path(&self, path: &Path) -> usize;

    /// [`Self::detect`], returning what it said. Not measured.
    fn answer(&self, bytes: &[u8]) -> Answer;

    /// [`Self::detect_path`], returning what it said. Not measured.
    fn answer_path(&self, path: &Path) -> Answer;

    /// A cost paid once, before any call to [`Self::detect`].
    fn startup(&self) -> Option<Startup> {
        None
    }
}

/// The library this repository publishes.
pub struct Magical;

impl Adapter for Magical {
    fn name(&self) -> &'static str {
        "magical_rs"
    }

    fn read_size(&self) -> String {
        // The number is written out rather than computed, so it can be a
        // `&'static str` in a table header, and the two competing numbers are
        // named -- because a reader who sees only this one has been told the
        // library needs 36,870 bytes and has not been told it settles for 2,048
        // for everything except the formats that live further into the file.
        format!("{READ_SIZE} (with_bytes_read), or 2048 by default")
    }

    fn detect(&self, bytes: &[u8]) -> usize {
        // `match_types` is the whole public entry point and it allocates
        // nothing: it reads the index, indexes the table, and copies out a
        // `FileKind`. `variant_name` is the string a caller would print, and
        // reading its length is the work of getting an answer they can use.
        FileKind::match_types(bytes).map_or(0, |kind| kind.variant_name().len())
    }

    fn detect_path(&self, path: &Path) -> usize {
        // The crate's own documented way to go from a path to a kind, used
        // verbatim: `read_file_header` with `READ_SIZE`, then `match_types`. A
        // caller could also read 2,048 bytes and miss ISO, TAR and the rest; the
        // full read is what the library tells a caller to do, so it is what is
        // measured.
        read_file_header(&path.to_string_lossy(), READ_SIZE).map_or(0, |bytes| self.detect(&bytes))
    }

    fn answer(&self, bytes: &[u8]) -> Answer {
        FileKind::match_types(bytes).map_or_else(Answer::none, |kind| Answer {
            label: kind.display_name().to_owned(),
            mime: kind.mime().map(str::to_owned),
            extension: kind.extension().map(str::to_owned),
        })
    }

    fn answer_path(&self, path: &Path) -> Answer {
        read_file_header(&path.to_string_lossy(), READ_SIZE)
            .map_or_else(|_| Answer::none(), |bytes| self.answer(&bytes))
    }
}

/// [`infer`](https://docs.rs/infer), the closest thing to a competitor that
/// exists: a table of magic numbers, a first match wins, no data files.
pub struct Infer;

impl Adapter for Infer {
    fn name(&self) -> &'static str {
        "infer"
    }

    fn read_size(&self) -> String {
        // `infer` exports no read-size constant and documents no limit: it reads
        // whatever slice it is handed and its longest offset is an implementation
        // detail of its own table. Printing a number here would be a claim about
        // it that `infer` does not make, so the honest cell says so.
        "no published limit; reads the slice given".to_owned()
    }

    fn detect(&self, bytes: &[u8]) -> usize {
        infer::get(bytes).map_or(0, |kind| kind.mime_type().len())
    }

    fn detect_path(&self, path: &Path) -> usize {
        // `get_from_path` is `infer`'s own file entry point, used as published.
        // The error case returns 0 rather than panicking, so a library that
        // cannot read a file is not excluded from the timing it lost.
        match infer::get_from_path(path) {
            Ok(Some(kind)) => kind.mime_type().len(),
            Ok(None) | Err(_) => 0,
        }
    }

    fn answer(&self, bytes: &[u8]) -> Answer {
        infer::get(bytes).map_or_else(Answer::none, |kind| Answer {
            label: format!("{:?}", kind.matcher_type()),
            mime: Some(kind.mime_type().to_owned()),
            extension: Some(kind.extension().to_owned()),
        })
    }

    fn answer_path(&self, path: &Path) -> Answer {
        match infer::get_from_path(path) {
            Ok(Some(kind)) => Answer {
                label: format!("{:?}", kind.matcher_type()),
                mime: Some(kind.mime_type().to_owned()),
                extension: Some(kind.extension().to_owned()),
            },
            Ok(None) | Err(_) => Answer::none(),
        }
    }
}

/// libmagic, through `magic-sys`, behind the `libmagic` feature.
///
/// See this crate's `README.md` for how to get a libmagic on each platform. The
/// short version: it is a C library with a 10 MB database, and `magic-sys`'s
/// build script fails rather than degrading when it cannot find one, which is
/// why this is an optional dependency rather than a hard one.
#[cfg(feature = "libmagic")]
pub struct LibMagic {
    /// MIME type only, from a buffer.
    mime: magic_sys::magic_t,
    /// Extension only, from a buffer.
    extension: magic_sys::magic_t,
    /// MIME type from a path, which is the code path a caller with a filename
    /// gets and which is *not* the same code path as `magic_buffer`.
    path: magic_sys::magic_t,
    /// How long opening and loading the database took, in milliseconds.
    load_millis: f64,
    /// The database, described.
    database: String,
}

#[cfg(feature = "libmagic")]
impl LibMagic {
    /// Finds the database, opens three cookies, and times the load.
    ///
    /// # Panics
    ///
    /// If no libmagic database can be found or loaded. The caller asked for
    /// libmagic by enabling the feature, and a libmagic with no database answers
    /// every buffer with `application/octet-stream` -- which would appear in the
    /// report as a library that is wrong about all four files, and would be a
    /// lie. There is no useful degraded mode.
    #[must_use]
    pub fn open() -> Self {
        // Found once per process, before the clock starts.
        //
        // Once, rather than once per adapter: `Report::build` and the criterion
        // target both open more than one, and `tests/harness.rs` runs fourteen
        // tests that each build a report, so fourteen threads would be searching
        // for the database and loading a 10 MB file at the same time. libmagic's
        // cookie API is per-cookie but its database loading is not documented as
        // safe to do concurrently, and it was not: several of those tests
        // panicked on a machine where the database was perfectly findable.
        //
        // Before the clock starts, too, because where `magic.mgc` lives is not
        // part of what is being measured and three failed `magic_load` attempts
        // would otherwise be inside the reported startup time.
        let database = database().unwrap_or_else(|| {
            panic!(
                "no libmagic database found. Tried, in order: $MAGIC, libmagic's own compiled-in \
                 default, the paths distributions put it in, then every vcpkg layout under \
                 $VCPKG_INSTALLED_DIR and $VCPKG_ROOT. Set $MAGIC to a magic.mgc, or install \
                 libmagic where one of those points. The list is in \
                 adapter::database_search_order."
            )
        });

        let started = std::time::Instant::now();
        let mime = cookie(magic_sys::MAGIC_MIME_TYPE, database);
        let extension = cookie(magic_sys::MAGIC_EXTENSION, database);
        let path = cookie(magic_sys::MAGIC_MIME_TYPE, database);
        let load_millis = started.elapsed().as_secs_f64() * 1_000.0;

        Self {
            mime,
            extension,
            path,
            load_millis,
            database: describe_database(database),
        }
    }
}

/// Where libmagic's `magic.mgc` is looked for, in order.
/// documented override and a person who set it meant it. Then the null path,
/// which is libmagic's own compiled-in default. Then the places distributions
/// and package managers actually put it, then the vcpkg layouts.
///
/// **The compiled-in default is not a fixed path.** libmagic's `MAGIC` macro
/// resolves to whatever its build passed, and the man page documents the
/// upstream value as `/usr/local/share/misc/magic` while Debian and Ubuntu ship
/// the file in `/usr/share/misc`. So `magic_load(cookie, NULL)` works on some
/// machines and not others for reasons nothing on this side can see -- and the
/// first version of this crate relied on it alone, so the benchmark workflow's
/// first run on Linux panicked and produced no numbers at all. A green job that
/// measured nothing is worse than a red one.
///
/// Every candidate below is validated by a real `magic_load` in
/// [`find_database`], so being in this list does not mean a path is right. It
/// means it is worth asking.
#[cfg(feature = "libmagic")]
fn database_search_order() -> Vec<Option<std::path::PathBuf>> {
    let mut order = vec![std::env::var_os("MAGIC").map(std::path::PathBuf::from)];
    // `None` is `magic_load(cookie, NULL)`: libmagic's own default search.
    order.push(None);

    // Debian and Ubuntu, Fedora and RHEL, Homebrew, and the two spellings of the
    // upstream default. Written out rather than globbed, because walking the
    // filesystem on every start is a way to find a database the operator did not
    // mean to use, and this list is short enough to read.
    for path in [
        "/usr/share/misc/magic.mgc",
        "/usr/local/share/misc/magic.mgc",
        "/usr/share/file/magic.mgc",
        "/usr/local/share/file/magic.mgc",
        "/opt/homebrew/etc/magic.mgc",
        "/usr/local/etc/magic.mgc",
    ] {
        order.push(Some(std::path::PathBuf::from(path)));
    }

    for root_var in ["VCPKG_INSTALLED_DIR", "VCPKG_ROOT"] {
        let Some(root) = std::env::var_os(root_var).map(std::path::PathBuf::from) else {
            continue;
        };
        // Classic trees put it under `installed/`, manifest trees under
        // `vcpkg_installed/`, and the `vcpkg` crate that `magic-sys` uses to find
        // the library insists on the classic spelling. So: all of them, for the
        // triplet the caller named and then for each host's default, because a
        // benchmark that only works when the triplet happens to match is a
        // benchmark that silently does not run.
        for prefix in ["installed", "vcpkg_installed", ""] {
            for triplet in [
                "x64-windows",
                "x64-windows-static-md",
                "x86_64-unknown-linux-gnu",
            ] {
                order.push(Some(mgc_under(&root.join(prefix).join(triplet))));
            }
            // The manifest layout sometimes drops the triplet level entirely.
            order.push(Some(mgc_under(&root.join(prefix))));
        }
    }
    order
}

/// `<root>/share/libmagic/misc/magic.mgc`, which is where vcpkg puts it.
#[cfg(feature = "libmagic")]
fn mgc_under(root: &Path) -> std::path::PathBuf {
    root.join("share")
        .join("libmagic")
        .join("misc")
        .join("magic.mgc")
}

/// The first database in [`database_search_order`] that libmagic will load, once
/// per process.
///
/// A path being in the list is not the test -- the file has to exist *and*
/// `magic_load` has to accept it, because a stale or wrong-triplet `magic.mgc`
/// is a file that exists and does not work. So the check is a real load on a
/// throwaway cookie, which costs about as much as the first real one and is
/// outside the reported startup figure.
///
/// `OnceLock` and not a plain `static` of the result, because the search has to
/// happen exactly once and a plain cache would be a data race.
#[cfg(feature = "libmagic")]
fn database() -> Option<&'static std::path::Path> {
    static FOUND: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
    FOUND.get_or_init(find_database).as_deref()
}

/// [`database`]'s search, run every time it is called.
#[cfg(feature = "libmagic")]
fn find_database() -> Option<std::path::PathBuf> {
    for candidate in database_search_order() {
        // A named path that is not there is skipped. `None` is not: it is
        // libmagic's own default search, the one that works on a Linux runner
        // and the only candidate that can be right with nothing configured.
        if candidate.as_ref().is_some_and(|path| !path.is_file()) {
            continue;
        }
        // SAFETY: `magic_open` with a flag word; the null check is the whole
        // response to a libmagic that cannot allocate, and the handle is closed
        // on every path out of this loop.
        let handle = unsafe { magic_sys::magic_open(magic_sys::MAGIC_NONE) };
        if handle.is_null() {
            return None;
        }
        // SAFETY: `handle` is non-null, and `candidate` is either null
        // (libmagic's default search) or a `PathBuf` that outlives the call. The
        // C string is NUL-terminated and cannot be truncated.
        let status = {
            let c_path = candidate.as_ref().and_then(|p| {
                std::ffi::CString::new(p.as_os_str().to_string_lossy().as_ref()).ok()
            });
            unsafe {
                magic_sys::magic_load(
                    handle,
                    c_path.as_ref().map_or(std::ptr::null(), |p| p.as_ptr()),
                )
            }
        };
        // SAFETY: `handle` is non-null and closed exactly once here.
        unsafe { magic_sys::magic_close(handle) };
        if status == 0 {
            return candidate;
        }
    }
    None
}

/// A loaded cookie, or a panic naming the reason.
///
/// The panic message is libmagic's own `magic_error`, because "could not open
/// magic cookie" on its own has been the answer to every libmagic problem ever
/// and tells a reader nothing about which step failed.
#[cfg(feature = "libmagic")]
fn cookie(flags: std::os::raw::c_int, database: &Path) -> magic_sys::magic_t {
    // SAFETY: `magic_open` takes a flag word and returns a handle or null, which
    // the check below is the whole response to.
    let handle = unsafe { magic_sys::magic_open(flags) };
    assert!(!handle.is_null(), "magic_open returned null");

    // Already found loadable by `find_database`, so a failure here is a libmagic
    // that changed its mind, and the message names the file it was asked to load
    // rather than leaving it to be guessed.
    let c_path = std::ffi::CString::new(database.to_string_lossy().as_ref())
        .expect("a filesystem path with a NUL in it is not a path");
    // SAFETY: `handle` is non-null and uniquely owned from here, and `c_path` is
    // a live NUL-terminated string for the duration of the call.
    let status = unsafe { magic_sys::magic_load(handle, c_path.as_ptr()) };
    assert_eq!(
        status,
        0,
        "magic_load failed for {}: {}",
        database.display(),
        // SAFETY: `magic_error` on a non-null handle returns a pointer the handle
        // owns, or null, and `describe` handles both.
        unsafe { describe(magic_sys::magic_error(handle)) }
    );

    // SAFETY: `magic_setflags` on an open handle, with either one documented
    // constant or the disjunction of two.
    assert_ne!(unsafe { magic_sys::magic_setflags(handle, flags) }, -1);
    handle
}

/// A `*const c_char` as a `String`, or `"<null>"`.
///
/// # Safety
///
/// `text` must be null or a NUL-terminated string that outlives the returned
/// value, which for every call site here means "owned by a handle that is still
/// open".
#[cfg(feature = "libmagic")]
unsafe fn describe(text: *const std::os::raw::c_char) -> String {
    if text.is_null() {
        return "<null>".to_owned();
    }
    // SAFETY: the caller guarantees non-null and NUL-terminated.
    unsafe { std::ffi::CStr::from_ptr(text) }
        .to_string_lossy()
        .into_owned()
}

/// The database as a table cell: its size, and where it came from.
///
/// The size is what makes the comparison legible. A library with a 10 MB
/// compiled ruleset and a library with none are not two implementations of one
/// idea, and the reader should not have to go and look that up.
#[cfg(feature = "libmagic")]
fn describe_database(path: &Path) -> String {
    let size = std::fs::metadata(path).map_or_else(
        |_| "size unknown".to_owned(),
        |m| {
            // Not `m.len() as f64`: that is a `cast_precision_loss`, and a file
            // size is exactly where being off by a bit would be invisible in the
            // output and wrong in the arithmetic. Through `u32`, which is far
            // more than enough bytes and cannot round.
            let bytes = f64::from(u32::try_from(m.len()).unwrap_or(u32::MAX));
            format!("{:.1} MiB of rules", bytes / (1024.0 * 1024.0))
        },
    );
    // Only the file name. A path is a fact about the machine the benchmark ran
    // on, and the vcpkg hash inside it is noise in a pull request comment.
    let name = path.file_name().map_or_else(
        || path.display().to_string(),
        |n| n.to_string_lossy().into_owned(),
    );
    format!("{size}, from {name}")
}

/// libmagic's own version, for the row label.
///
/// `magic_version` returns a packed integer and the header defines it as
/// `MAGIC_VERSION 547` for 5.47 -- `major * 100 + minor`, with no patch field.
/// `magic-sys` exposes no string version function, so the integer is unpacked
/// here, and a value that does not fit that encoding prints as the raw number
/// rather than as a plausible-looking "0.5.47" that is a misreading.
#[cfg(feature = "libmagic")]
fn magic_version() -> String {
    // SAFETY: no arguments, and a version number has nothing to check.
    let packed = unsafe { magic_sys::magic_version() };
    if !(0..10_000).contains(&packed) {
        return format!("version {packed} (unrecognised encoding)");
    }
    format!("{}.{}", packed / 100, packed % 100)
}

/// Strips libmagic's `; charset=...` suffix and calls its own "I do not know"
/// answer what it is.
///
/// Both are normalisation of a *reported* string in the unmeasured pass, and
/// both are things libmagic says rather than things this crate believes.
/// `application/octet-stream` is left alone: the report prints it deliberately,
/// because "I do not know this" and "this is binary data" are different things
/// for a caller to have to handle.
#[cfg(feature = "libmagic")]
fn mime_of(text: &str) -> Option<String> {
    let mime = text.split(';').next().unwrap_or(text).trim();
    (!mime.is_empty()).then(|| mime.to_owned())
}

#[cfg(feature = "libmagic")]
impl Drop for LibMagic {
    fn drop(&mut self) {
        // SAFETY: all three handles came from `cookie`, which asserted them
        // non-null, and each is closed exactly once, here.
        unsafe {
            magic_sys::magic_close(self.mime);
            magic_sys::magic_close(self.extension);
            magic_sys::magic_close(self.path);
        }
    }
}

#[cfg(feature = "libmagic")]
impl Adapter for LibMagic {
    fn name(&self) -> &'static str {
        "libmagic"
    }

    fn read_size(&self) -> String {
        // What it will look at, and the version that decides how. The MIME
        // column for a libmagic that is compiled without a database is the
        // honest description of such a libmagic, which is why `open` refuses to
        // return one.
        format!("whole buffer passed in; libmagic {}", magic_version())
    }

    fn detect(&self, bytes: &[u8]) -> usize {
        // SAFETY: `self.mime` is a loaded handle owned by `self` and outlives
        // this call, and the pointer/length pair is exactly `bytes` for its
        // duration, which is what `magic_buffer` requires.
        let answer = unsafe {
            magic_sys::magic_buffer(
                self.mime,
                bytes.as_ptr().cast::<std::os::raw::c_void>(),
                bytes.len(),
            )
        };
        if answer.is_null() {
            return 0;
        }
        // The caller's own work: libmagic hands back a pointer into its own
        // storage, so reading the string out of it is part of using the answer
        // and is inside the measurement. Not copied.
        // SAFETY: non-null (checked above), NUL-terminated, owned by the still-open
        // cookie.
        unsafe { std::ffi::CStr::from_ptr(answer) }.to_bytes().len()
    }

    fn detect_path(&self, path: &Path) -> usize {
        let Ok(c_path) = std::ffi::CString::new(path.to_string_lossy().as_ref()) else {
            return 0;
        };
        // SAFETY: `self.path` outlives this call, and `c_path` is a live
        // NUL-terminated string for its duration.
        let answer = unsafe { magic_sys::magic_file(self.path, c_path.as_ptr()) };
        if answer.is_null() {
            return 0;
        }
        // SAFETY: as in `detect`.
        unsafe { std::ffi::CStr::from_ptr(answer) }.to_bytes().len()
    }

    fn answer(&self, bytes: &[u8]) -> Answer {
        // Two calls, because on libmagic 5.47 asking for `MAGIC_MIME |
        // MAGIC_EXTENSION` prints the extension and drops the MIME type: with
        // both flags set the output is `png; charset=binary`, where the MIME
        // alone is `image/png`. Combining them from one call is not a matter of
        // splitting on a tab, because there is no tab in it. Two cookies, two
        // calls, and it costs nothing that is measured -- this is the unmeasured
        // pass, and the measured one uses the MIME cookie alone.
        let mime = buffer_string(self.mime, bytes);
        let extension = buffer_string(self.extension, bytes);
        Answer {
            label: mime.clone().unwrap_or_else(|| "-".to_owned()),
            mime: mime
                .as_deref()
                .and_then(mime_of)
                .filter(|m| m != "application/octet-stream"),
            extension: extension
                .as_deref()
                .and_then(mime_of)
                .filter(|e| e != "???" && !e.is_empty()),
        }
    }

    fn answer_path(&self, path: &Path) -> Answer {
        let Ok(c_path) = std::ffi::CString::new(path.to_string_lossy().as_ref()) else {
            return Answer::none();
        };
        // SAFETY: `self.path` outlives this call and `c_path` is live for its
        // duration. A null return means libmagic could not open the file, which
        // is a real answer rather than a failure to be papered over.
        let answer = unsafe { magic_sys::magic_file(self.path, c_path.as_ptr()) };
        if answer.is_null() {
            return Answer::none();
        }
        // SAFETY: non-null, NUL-terminated, owned by the still-open cookie.
        let text = unsafe { std::ffi::CStr::from_ptr(answer) }
            .to_string_lossy()
            .into_owned();
        Answer {
            label: "-".to_owned(),
            mime: mime_of(&text).filter(|m| m != "application/octet-stream"),
            extension: None,
        }
    }

    fn startup(&self) -> Option<Startup> {
        Some(Startup {
            what: "magic_open + magic_load, three times: a cookie for MIME from a buffer, one for \
                   the extension, one for the path-based call",
            millis: self.load_millis,
            note: Some(self.database.clone()),
        })
    }
}

/// The string libmagic produced for a buffer, or [`None`] if it produced none.
///
/// A free function rather than a method because it uses no field of the adapter:
/// the cookie is passed in, and a method that took `&self` and ignored it would
/// read as though there were some other cookie it might prefer.
#[cfg(feature = "libmagic")]
fn buffer_string(cookie: magic_sys::magic_t, bytes: &[u8]) -> Option<String> {
    // SAFETY: `cookie` is one of `LibMagic`'s own loaded handles, which outlives
    // the call, and the pointer/length pair is exactly `bytes`.
    let answer = unsafe {
        magic_sys::magic_buffer(
            cookie,
            bytes.as_ptr().cast::<std::os::raw::c_void>(),
            bytes.len(),
        )
    };
    if answer.is_null() {
        return None;
    }
    // SAFETY: non-null, NUL-terminated, owned by the still-open cookie.
    Some(
        unsafe { std::ffi::CStr::from_ptr(answer) }
            .to_string_lossy()
            .into_owned(),
    )
}

/// Every adapter that is compiled into this build, in the order tables print them.
///
/// The order is the order of the list in this function and nothing is sorted by
/// time, because a table whose rows are ordered by the winner is a table that
/// has been arranged.
#[must_use]
pub fn all() -> Vec<Box<dyn Adapter>> {
    // Two `cfg` branches rather than a `mut` binding with a `#[cfg]` push, so a
    // build without the feature does not warn about a `mut` nothing needs.
    #[cfg(feature = "libmagic")]
    {
        vec![
            Box::new(Magical),
            Box::new(Infer),
            Box::new(LibMagic::open()),
        ]
    }
    #[cfg(not(feature = "libmagic"))]
    {
        vec![Box::new(Magical), Box::new(Infer)]
    }
}
