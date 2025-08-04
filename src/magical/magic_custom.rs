/// Rules that determine how a `MagicCustom` instance matches against input bytes.
///
/// This enum allows defining flexible matching strategies for custom file type detection.
/// It supports both byte signature matching and custom logic via function pointers,
/// making it suitable for integration in `static` contexts and `no_std` environments.
///
/// # Variants
///
/// - [`CustomMatchRules::Default`]: Matches by checking if any of the defined `signatures`
///   appears at any of the specified `offsets` within the input data.
///   This is the standard method used for most file types (e.g., PNG, ZIP, ELF).
///
/// - [`CustomMatchRules::WithFn(fn(&[u8]) -> bool)`]: Uses a **custom function** to determine
///   whether the input bytes match a specific format.
///   The function must:
///   - Be a `fn` pointer (not a closure or `Box<dyn Fn>`).
///   - Have the exact signature `fn(&[u8]) -> bool`.
///   - Be usable in `const` or `static` contexts.
///
/// # Why Function Pointers?
///
/// - Zero-cost: No heap allocation or vtable overhead.
/// - static compatible: Can be used in `static MAGIC_RULE: MagicCustom<T> = ...`.
/// - `no_std` friendly: No dependency on `std`, `alloc`, or runtime features.
/// - No closure: You cannot use inline closures like `|b| b.starts_with(...)` directly.
///   Instead, define a separate `const fn` and pass it.
///
/// # Example
///
/// ```rust
/// use magical_rs::magical::magic_custom::{MagicCustom, CustomMatchRules};
///
/// #[derive(Debug, Clone, Copy, PartialEq)]
/// enum ShoujuFile {
///     MahouShouju,
///     Unknown,
/// }
///
/// fn is_magical_girl(bytes: &[u8]) -> bool {
///     bytes.starts_with(b"MagicalGirl")
/// }
///
/// static SHOUJO_FILE: MagicCustom<ShoujuFile> = MagicCustom {
///     signatures: &[],
///     offsets: &[],
///     max_bytes_read: 32,
///     kind: ShoujuFile::MahouShouju,
///     rules: CustomMatchRules::WithFn(is_magical_girl),
/// };
///
/// ```
///
/// # Note
///
/// This enum is designed to work seamlessly with `MagicCustom<K>` for extensible,
/// compile-time file type detection without sacrificing performance or portability.
///
#[derive(Clone, Copy)]
pub enum CustomMatchRules<'a> {
    /// Default. Let [`MagicCustom::matches_custom`] it handle for you.
    Default,
    /// With a single function.
    /// You can use it to check any condition as long as it
    /// returns a boolean. The result will be the basic for
    /// checking if the byte matches the rule.
    ///
    /// Macros with sugar syntax: [`with_fn_matches`]
    WithFn(fn(bytes: &[u8]) -> bool),
    /// For the purpose of using more than one function,
    /// with `OR` comparison type, this will be what you
    /// need.
    ///
    /// It's like [`CustomMatchRules::WithFn`], but can be
    /// used with multipe functions returning bool.
    /// Just one function that returns `true`, is enough.
    ///
    /// So, use it when you want to relax the file recognition rules.
    ///
    /// Macros with sugar syntax: [`any_matches`]
    AnyMatches(&'a [fn(bytes: &[u8]) -> bool]),
    /// This way you will be able to define multipe functions with rules.
    ///
    /// However, all functions must be passed. Otherwise false will be returned.
    ///
    /// Use when you want the strictest rule possible.
    ///
    /// Corresponds to the `AND` comparison type.
    ///
    /// Macros with sugar syntax: [`all_matches`]
    AllMatches(&'a [fn(bytes: &[u8]) -> bool]),
}

#[derive(Clone, Copy)]
pub struct MagicCustom<'a, K> {
    /// File signature.
    /// Multiple signatures can be added at once.
    pub signatures: &'static [&'static [u8]],
    /// File offset.
    /// Multiple offsets can be added at once.
    pub offsets: &'static [usize],
    /// Maximum number of bytes to read.
    pub max_bytes_read: usize,
    /// The file's identifier is determined by the enum
    /// if it matches.
    pub kind: K,
    /// File identification rules.
    ///
    /// If not used, leave this field as: [`CustomMatchRules::Default`].
    ///
    /// Otherwise, leave [`MagicCustom::signatures`] and [`MagicCustom::offsets`]
    /// as `&[]`.
    pub rules: CustomMatchRules<'a>,
}

impl<K: Clone> MagicCustom<'_, K> {
    #[must_use]
    #[inline]
    fn matches_custom(&self, bytes: &[u8]) -> bool {
        match &self.rules {
            CustomMatchRules::Default => self.signatures.iter().any(|&signature| {
                self.offsets.iter().any(|&offset| {
                    let offset_end = offset + signature.len();

                    bytes.len() >= offset_end && &bytes[offset..offset_end] == signature
                })
            }),

            CustomMatchRules::WithFn(func) => func(bytes),
            CustomMatchRules::AnyMatches(funcs) => funcs.iter().any(|&func| func(bytes)),
            CustomMatchRules::AllMatches(funcs) => funcs.iter().all(|&func| func(bytes)),
        }
    }
}

/// Detects the file type by matching against a custom list of `MagicCustom` rules.
///
/// This function iterates over the provided `rules` and returns the `kind` of the first rule
/// whose signature or custom logic matches the input `bytes`. If no rule matches,
/// it returns the provided `fallback` value.
///
/// ---
///
/// # Type Parameters
/// - `K`: The type representing the file kind (e.g., an enum, `&'static str`, or custom struct).
///   Must implement `Clone` to be returned by value.
///
/// ---
///
/// # Parameters
/// - `bytes`: The raw byte slice to analyze (e.g., file header or buffer).
/// - `rules`: A slice of `MagicCustom<K>` rules to check in order.
/// - `fallback`: The value to return if no rule matches.
///
/// ---
///
/// # Returns
/// - `K`: The `kind` of the first matching rule.
/// - `fallback`: If no rule matches.
///
/// ---
///
/// # Why This Matters
///
/// This function enables fully customizable file type detection:
/// - You can define your own `kind` type (e.g., `MyFileKind`, `&'static str`).
/// - You can create rules with custom signatures, offsets, or validation functions.
/// - It works in `no_std` environments — no heap allocation, no `std` dependency.
///
/// It is the core of the extensible detection system, allowing integration
/// into embedded systems, plugin architectures, or domain-specific formats.
///
/// ---
///
/// # Examples
///
/// ```rust
/// use magical_rs::magical::magic_custom::{MagicCustom, CustomMatchRules, match_types_custom};
///
/// #[derive(Debug, Clone, Copy, PartialEq)]
/// enum ShoujoKind {
///     Shoujo,
///     Unknown,
/// }
///
/// const SHOUJO: &[u8] = b"I_LOVE_MAGICAL_GIRL";
/// static SHOUJO_RULE: MagicCustom<ShoujoKind> = MagicCustom {
///     signatures: &[SHOUJO],
///     offsets: &[0],
///     max_bytes_read: 32,
///     kind: ShoujoKind::Shoujo,
///     rules: CustomMatchRules::Default,
/// };
///
/// let data = b"I_LOVE_MAGICAL_GIRL FOREVER";
/// let kind = match_types_custom(data, &[SHOUJO_RULE], ShoujoKind::Shoujo);
/// assert_eq!(kind, ShoujoKind::Shoujo);
/// ```
/// ---
///
/// # Note
/// - Rules are checked in order — the first match wins.
/// - The function uses `.find()` and `.map_or()`, so it short-circuits on the first match.
/// - It does not enforce `max_bytes_read` — that is the caller's responsibility.
///   Use `bytes.len()` or external checks if needed.
///
/// ---
///
/// # Performance
/// - Zero-cost abstraction: no allocation, no dynamic dispatch.
/// - `#[inline]`-friendly (you can add `#[inline]` if desired).
/// - Works in `no_std` and `const` contexts (with limitations).
#[inline]
#[must_use]
pub fn match_types_custom<K: Clone>(bytes: &[u8], rules: &[MagicCustom<K>], fallback: K) -> K {
    rules
        .iter()
        .find(|rule| rule.matches_custom(bytes))
        .map_or(fallback, |rule| rule.kind.clone())
}

/// Macros with sugar-coated syntax for [`CustomMatchRules::AnyMatches`]
#[macro_export]
macro_rules! any_matches {
    ($($func:expr),+ $(,)?) => {
        $crate::magical::magic_custom::CustomMatchRules::AnyMatches(&[$($func),+])
    };
}
/// Macros with sugar-coated syntax for [`CustomMatchRules::AllMatches`]
#[macro_export]
macro_rules! all_matches {
    ($($func:expr),+ $(,)?) => {
        $crate::magical::magic_custom::CustomMatchRules::AllMatches(&[$($func),+])
    };
}

#[macro_export]
macro_rules! with_fn_matches {
    ($func:expr) => {
        $crate::magical::magic_custom::CustomMatchRules::WithFn($func)
    };
}

/// Macros with sugar-coated syntax for [`MagicCustom`]
///
/// Makes [`MagicCustom`] initialization much easier and more readable.
///
/// ---
///
/// # Examples
///
/// ```rust
/// use magical_rs::any_matches;
/// use magical_rs::magic_custom;
///
/// #[derive(Clone, Copy, Debug)]
/// enum CuteGirlKind {
///     ShoujoFile,
///     UnknownFallback,
/// }
///
/// fn find_shoujo_girl(bytes: &[u8]) -> bool {
///     bytes.starts_with(b"MagicalGirl")
/// }
/// let rule = magic_custom! (
///     signatures: [b"MagicalGirl"],
///     offsets: [0],
///     max_bytes_read: 69,
///     kind: CuteGirlKind::ShoujoFile,
///     rules: any_matches!(find_shoujo_girl)
/// );
/// ```
#[macro_export]
macro_rules! magic_custom {
    (signatures: [$($sig:expr),+ $(,)?],
    offsets: [$($offsets:expr),+ $(,)?],
    max_bytes_read: $max_bytes_read:expr,
    kind: $kind:expr,
    rules: $rules:expr) => {
        $crate::magical::magic_custom::MagicCustom {
            signatures: &[$($sig),+],
            offsets: &[$($offsets),+],
            max_bytes_read: $max_bytes_read,
            kind: $kind,
            rules: $rules
        }
    };
}

#[macro_export]
macro_rules! match_custom {
    (bytes: $bytes:expr,
    rules: [$($rules:expr),+$(,)?],
    fallback: $fallback:expr) => {
     $crate::magical::magic_custom::match_types_custom($bytes, &[$($rules),+], $fallback)
    };
}
