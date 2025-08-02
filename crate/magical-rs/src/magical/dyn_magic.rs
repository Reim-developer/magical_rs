#[cfg(feature = "magical_dyn")]
use std::any::Any;
#[cfg(feature = "magical_dyn")]
use std::boxed::Box;
#[cfg(feature = "magical_dyn")]
use std::vec::Vec;
#[cfg(feature = "magical_dyn")]
type MatcherBox = Box<dyn Fn(&[u8]) -> bool + Send + Sync>;

/// A dynamic magic rule that supports closures and arbitrary return types.
///
/// This struct is designed for use in runtime-defined file type detection,
/// such as plugin systems, configuration-driven analyzers, or interactive tools.
/// It allows:
/// - Closures as matchers (e.g., `|bytes| bytes.starts_with(...)`).
/// - Storing any type as `kind` (e.g., `&'static str`, `String`, enums, structs).
///
/// # Feature and Runtime Requirements
///
/// - Only available when the `dyn_magic` feature is enabled.
/// - Requires `std` (due to use of `Box<dyn Any>` and `Box<dyn Fn>`).
/// - Not compatible with `no_std` environments.
/// - Rules cannot be declared in `static` contexts if they contain non-`'static` closures.
///
/// # Fields
/// - `matcher`: A boxed closure that checks if input bytes match a format.
/// - `kind`: The associated value (type, tag, metadata) to return on match.
/// - `max_bytes_read`: Suggested maximum number of bytes to read (not enforced).
///
/// # Example
///
/// ```rust
/// # #[cfg(feature = "magical_dyn")]
/// # use magical_rs::magical::dyn_magic::DynMagicCustom;
///
/// # #[cfg(feature = "magical_dyn")]
/// # let rule = DynMagicCustom::new(
///     |bytes: &[u8]| bytes.starts_with(b"MAGIC"),
///     "MyFormat",
///     32,
/// # );
///
/// # #[cfg(feature = "magical_dyn")]
///
/// # assert!(rule.matches(b"MAGIC_DATA"));
/// # if let Some(&format) = rule.kind().downcast_ref::<&str>() {
///     assert_eq!(format, "MyFormat");
/// # }
/// ```
/// # Performance
///
/// This type uses dynamic dispatch and heap allocation `Box`,
/// so it is slower than `MagicCustom` for static rules.
/// Use it only when runtime flexibility is required.
#[cfg(feature = "magical_dyn")]
pub struct DynMagicCustom {
    matcher: MatcherBox,
    kind: Box<dyn Any + Send + Sync>,
    max_bytes_read: usize,
}

#[cfg(feature = "magical_dyn")]
impl DynMagicCustom {
    /// Creates a new dynamic rule with a matcher closure, associated kind, and max read limit.
    ///
    /// The closure must be `'static`, `Send`, and `Sync` to be boxed safely.
    /// The `kind` can be any type that is `'static`, `Send`, and `Sync`.
    #[must_use]
    pub fn new<F, K>(matcher: F, kind: K, max_bytes_read: usize) -> Self
    where
        F: Fn(&[u8]) -> bool + 'static + Send + Sync,
        K: 'static + Send + Sync,
    {
        Self {
            matcher: Box::new(matcher),
            kind: Box::new(kind),
            max_bytes_read,
        }
    }

    /// Check if the input bytes match this rules.
    #[must_use]
    pub fn matches(&self, bytes: &[u8]) -> bool {
        (self.matcher)(bytes)
    }

    /// Returns a reference to the stored `kind` as `&dyn Any`
    ///
    /// Use [`downcast_ref`] to recover the original type.
    ///
    /// [`downcast_ref`]: https://doc.rust-lang.org/std/any/trait.Any.html#method.downcast_ref
    #[must_use]
    pub fn kind(&self) -> &dyn Any {
        self.kind.as_ref()
    }

    /// Attempts to downcast the `kind` to reference of type `T`
    ///
    /// Returns `Some(&T)` if the type matches, [`None`] otherwise.
    #[must_use]
    pub fn kind_downcast_ref<T: 'static>(&self) -> Option<&T> {
        self.kind.as_ref().downcast_ref::<T>()
    }

    /// Returns the suggested maxium number of bytes to read for matching.
    ///
    /// This value is not enforced by the crate - for guidance only.
    #[must_use]
    pub const fn max_bytes_read(&self) -> usize {
        self.max_bytes_read
    }
}

#[cfg(feature = "magical_dyn")]
#[must_use]
/// Find the first matching rule and returns a reference to its `kind`
///
/// The returned `&dyn Any` has the same life time as the `rules` slice.
pub fn match_dyn_types<'a>(bytes: &[u8], rules: &'a [DynMagicCustom]) -> Option<&'a dyn Any> {
    rules
        .iter()
        .find(|rule| rule.matches(bytes))
        .map(DynMagicCustom::kind)
}

#[cfg(feature = "magical_dyn")]
/// Shorthand: returns the first matching `kind` downcasted to `&T`
#[must_use]
pub fn match_dyn_types_as<'a, T: 'static>(
    bytes: &[u8],
    rules: &'a [DynMagicCustom],
) -> Option<&'a T> {
    match_dyn_types(bytes, rules)?.downcast_ref::<T>()
}

#[cfg(feature = "magical_dyn")]
/// Returns all matching `kind`(s).
#[must_use]
pub fn match_dyn_types_all<'a>(bytes: &[u8], rules: &'a [DynMagicCustom]) -> Vec<&'a dyn Any> {
    rules
        .iter()
        .filter(|rule| rule.matches(bytes))
        .map(DynMagicCustom::kind)
        .collect()
}
