/// Rules that determine how a `MagicCustom` instance matches against input bytes.
///
/// This enum allows defining flexible matching strategies for custom file type detection.
/// It supports both byte signature matching and custom logic via function pointers,
/// making it suitable for integration in `static` contexts and `no_std` environments.
///
/// ---
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
/// static SHOUJO_FILE: MagicCustom<ShoujuFile> = MagicCustom {
///     signatures: &[b"Magic"],
///     offsets: &[0],
///     max_bytes_read: 32,
///     kind: ShoujuFile::MahouShouju,
///     rules: CustomMatchRules::Default,
/// };
///
/// ```
#[derive(Clone, Copy)]
pub enum CustomMatchRules<'a> {
    /// Default. Let [`match_types_custom`] it handle for you.
    ///
    /// # Examples
    /// ```
    /// use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom, match_types_custom};
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// enum FileKind {
    ///     Png,
    ///     Unknown,
    /// }
    ///
    /// const PNG_SIGNATURE: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    /// static PNG_RULE: MagicCustom<FileKind> = MagicCustom {
    ///     signatures: &[PNG_SIGNATURE],
    ///     offsets: &[0],
    ///     max_bytes_read: 2048,
    ///     kind: FileKind::Png,
    ///     rules: CustomMatchRules::Default,
    /// };
    ///
    /// const PNG_BYTES: &[u8] = &[
    ///     0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
    /// ];
    ///
    /// let result = match_types_custom(PNG_BYTES, &[PNG_RULE], FileKind::Unknown);
    ///
    /// assert_eq!(result, FileKind::Png);
    /// assert_ne!(result, FileKind::Unknown);
    /// ```
    ///
    /// ---
    ///
    /// # No Standard Library Context:
    /// Basically, [`CustomMatchRules::Default`] supports `no_std` context.
    ///
    /// It only requires at least Rust's [`core`].
    Default,

    /// # Safety:
    /// With a single function.
    /// You can use it to check any condition as long as it
    /// returns a boolean. The result will be the basic for
    /// checking if the byte matches the rule.
    ///
    /// However, it is limited to a single function and the logic is `OR` by default.
    ///
    /// Therefore, it is not suitable for strict rules or many combinations. It
    /// should not be used if the rule is too strict or many conditions are interwined.
    ///
    /// ---
    ///
    /// # Examples:
    /// ```rust
    /// use magical_rs::magical::magic_custom::{MagicCustom, match_types_custom};
    /// use magical_rs::with_fn_matches;
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq)]
    /// enum ShoujuFile {
    ///     MahouShouju,
    ///     Unknown,
    /// }
    ///
    /// fn is_shoujo_girl(bytes: &[u8]) -> bool {
    ///     bytes.starts_with(b"MagicalGirl")
    /// }
    ///
    /// static SHOUJO_RULE: MagicCustom<ShoujuFile> = MagicCustom {
    ///     signatures: &[],
    ///     offsets: &[],
    ///     max_bytes_read: 2048,
    ///     kind: ShoujuFile::MahouShouju,
    ///     rules: with_fn_matches!(is_shoujo_girl),
    /// };
    ///
    /// let magical_girl = b"MagicalGirl";
    /// let result = match_types_custom(magical_girl, &[SHOUJO_RULE], ShoujuFile::Unknown);
    ///
    /// assert_eq!(result, ShoujuFile::MahouShouju);
    /// assert_ne!(result, ShoujuFile::Unknown);
    /// ```
    ///
    /// ---
    ///
    /// # Macros:
    /// Macros with sugar syntax: [`with_fn_matches`]
    ///
    /// [`with_fn_matches`]: https://docs.rs/magical_rs/0.3.1/magical_rs/macro.with_fn_matches.html
    ///
    /// ---
    ///
    /// # No Standard Library Context:
    /// Basically, [`CustomMatchRules::WithFn`] supports `no_std` context.
    ///
    /// It only requires at least Rust's [`core`].
    WithFn(fn(bytes: &[u8]) -> bool),

    /// # Safety:
    /// For the purpose of using more than one function,
    /// with `OR` comparison type, this will be what you
    /// need.
    ///
    /// It's like [`CustomMatchRules::WithFn`], but can be
    /// used with multipe functions returning bool.
    /// Just one function that returns `true`, is enough.
    /// So, use it when you want to relax the file recognition rules.
    ///
    /// However, remember that even if you add `1000` rule functions, as long
    /// as just one of them is passed. Implicitly returning true, then of course
    /// the file will be recognized as the `Kind` you defined. It can create security
    /// risks in systems that require very high levels of identification and security.
    /// So, if you require a stricter rule, use [`CustomMatchRules::AllMatches`].
    ///
    /// ---
    ///     
    /// # Examples
    /// ```
    /// use magical_rs::any_matches;
    /// use magical_rs::magic_custom;
    /// use magical_rs::match_custom;
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// enum CuteGirlKind {
    ///     ShoujoFile,
    ///     UnknownFallback,
    /// }
    ///
    /// fn find_shoujo_girl(bytes: &[u8]) -> bool {
    ///     bytes.starts_with(b"MagicalGirl")
    /// }
    ///
    /// fn wrong_shoujo_girl(bytes: &[u8]) -> bool {
    ///     !bytes.starts_with(b"MagicalGirl")
    /// }
    ///
    /// let rule = magic_custom! (
    ///     signatures: [b"MagicalGirl"],
    ///     offsets: [0],
    ///     max_bytes_read: 69,
    ///     kind: CuteGirlKind::ShoujoFile,
    ///     rules: any_matches!(find_shoujo_girl, wrong_shoujo_girl)
    /// );
    ///
    /// let result = match_custom! {
    ///     bytes: b"MagicalGirl",
    ///     rules: [rule],
    ///     fallback: CuteGirlKind::UnknownFallback
    /// };
    ///
    /// assert_eq!(result, CuteGirlKind::ShoujoFile);
    /// assert_ne!(result, CuteGirlKind::UnknownFallback);
    /// ```
    /// ---
    ///
    /// # Macros:
    /// Macros with sugar syntax: [`any_matches`]
    ///
    /// [`any_matches`]: https://docs.rs/magical_rs/latest/magical_rs/macro.any_matches.html
    ///
    /// ---
    /// # No Standard Library Context:
    /// Basically, [`CustomMatchRules::AnyMatches`] supports `no_std` context.
    ///
    /// It only requires at least Rust's [`core`].
    AnyMatches(&'a [fn(bytes: &[u8]) -> bool]),

    /// # Safety:
    /// This way you will be able to define multipe functions with rules.
    ///
    /// However, all functions must be passed. Otherwise false will be returned.
    ///
    /// Use when you want the strictest rule possible.
    ///
    /// Corresponds to the `AND` comparison type.
    ///
    /// That is, if you need a very strict rule that guarantees that everything must pass,
    /// implicitly returning true, then use it. It doesn't make sense if you only need to check
    /// simple conditions, and may cause undefined beavior if used in the wrong context.
    ///
    /// ---
    ///
    /// # Examples:
    /// ```
    /// use magical_rs::all_matches;
    /// use magical_rs::magic_custom;
    /// use magical_rs::match_custom;
    ///
    /// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    /// enum CuteGirlKind {
    ///     ShoujoFile,
    ///     UnknownFallback,
    /// }
    ///
    /// fn find_shoujo_girl(bytes: &[u8]) -> bool {
    ///     bytes.starts_with(b"MagicalGirl")
    /// }
    ///
    /// fn wrong_shoujo_girl(bytes: &[u8]) -> bool {
    ///     !bytes.starts_with(b"MagicalGirl")
    /// }
    ///
    /// let rule = magic_custom! (
    ///     signatures: [],
    ///     offsets: [],
    ///     max_bytes_read: 69,
    ///     kind: CuteGirlKind::ShoujoFile,
    ///     rules: all_matches!(find_shoujo_girl, wrong_shoujo_girl)
    /// );
    ///
    /// let result = match_custom! {
    ///     bytes: b"MagicalGirl",
    ///     rules: [rule],
    ///     fallback: CuteGirlKind::UnknownFallback
    /// };
    ///
    /// assert_eq!(result, CuteGirlKind::UnknownFallback);
    /// assert_ne!(result, CuteGirlKind::ShoujoFile);
    ///
    /// ```
    ///
    /// ---
    ///
    /// # Macros:
    /// Macros with sugar syntax: [`all_matches`]
    ///
    /// [`all_matches`]: https://docs.rs/magical_rs/latest/magical_rs/macro.all_matches.html
    ///
    /// ---
    ///
    /// # No Standard Library Context:
    /// Basically, [`CustomMatchRules::AllMatches`] supports `no_std` context.
    ///
    /// It only requires at least Rust's [`core`].
    AllMatches(&'a [fn(bytes: &[u8]) -> bool]),

    /// # Safety:
    /// Here, you can use raw pointers. There are no restrictions.
    ///
    /// You will need to make sure the length of data is correct.
    ///
    /// Too much will cause overhead, to litle will cause undefined
    /// beavior.
    ///
    /// Worse, it can cause your program to crash irreversibly, or
    /// cause serious erros such as segmentation faults, buffer overflows,
    /// or memory overwrites.
    ///
    /// Only use this if you case requires very high
    /// performance, or in environments like kernel, embeded, or you want
    /// handle very specific file types, maybe very large file over 100 GiB,
    /// then this will be a very good choice.
    ///
    /// However, you will also have trade off the inherent memory safety of Rust.
    ///
    /// So, use it with caution.
    ///
    /// ---
    ///
    /// # Usable Version:
    /// You can use [`CustomMatchRules::WithFnUnsafe`] from version `0.4.0` onwards.
    ///
    /// ---
    ///
    /// # Feature Flag:
    /// To be safe and make sure you know what you are doing, you need to use the feature flag:
    ///
    /// ```bash
    /// cargo add magical_rs --features unsafe_context
    /// ```
    /// By default, this feature is disabled and not compiled.
    ///
    /// ---
    ///
    /// # Examples:
    ///```rust
    /// use core::slice;
    /// use magical_rs::magical::magic_custom::match_types_custom;
    /// use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};
    ///
    /// fn is_magic_file() {
    ///     #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    ///     enum MagicKind {
    ///         MoeMoe,
    ///         UnknownFallback,
    ///     }
    ///
    ///     fn is_shoujo_girl(data: *const ()) -> bool {
    ///         unsafe {
    ///             let slice_ptr = data.cast::<u8>();
    ///             let slice = slice::from_raw_parts(slice_ptr, 100);
    ///
    ///             slice.starts_with(b"MagicalGirl")
    ///         }
    ///     }
    ///
    ///     let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
    ///         signatures: &[],
    ///         offsets: &[],
    ///         max_bytes_read: 200,
    ///         kind: MagicKind::MoeMoe,
    ///         rules: CustomMatchRules::WithFnUnsafe {
    ///             func: is_shoujo_girl,
    ///         },
    ///     }];
    ///
    ///     let result = match_types_custom(b"MagicalGirl", rules, MagicKind::UnknownFallback);
    ///
    ///     assert_eq!(result, MagicKind::MoeMoe);
    ///     assert_ne!(result, MagicKind::UnknownFallback);
    /// }
    /// ```
    ///
    /// ---
    ///
    /// # No Standard Library Context:
    /// Basically, [`CustomMatchRules::WithFnUnsafe`] supports `no_std` context.
    ///
    /// It only requires at least Rust's [`core`].
    ///
    /// You can also skip using [`core`] if you don't use [`core::slice`].
    ///
    /// The above sample uses [`core`] and [`core::slice`] as an assumption of minimal
    /// environments like kernel, embedded.
    ///
    /// ---
    ///
    /// # Macros:
    /// We will not and do not intend to create macros for this context.
    ///
    /// Macros would undermine the reliability of the type system, which is
    /// already fragile in `unsafe`. So not using macros, even if supported, is
    /// always encouraged.
    #[cfg(feature = "unsafe_context")]
    WithFnUnsafe {
        func: unsafe fn(ptr_data: *const ()) -> bool,
    },

    /// # Safety:
    ///
    /// Similar to [`CustomMatchRules::AnyMatches`] but supports `unsafe`
    ///
    /// Here, you can pass in multiple unsafe function pointers.
    ///
    /// However, these functions will work on `OR` logic, so only one
    /// function returning `true` is needed.
    ///
    /// Since it's operating in an `unsafe` context, you need to control
    /// everything and there are no constraints or protections from the
    /// Rust compiler.
    ///
    /// It can cause segmentation faults, memory leaks, or worse
    /// undefined beavior.
    ///
    /// Becareful and use them when you need to combine multiple unsafe function
    /// pointers.
    ///
    /// Note that the rules for using `OR` logic can be very loose if you're not
    /// careful. If stricter rules are needed, use [`CustomMatchRules::AllMatchesUnsafe`],
    /// or use [`CustomMatchRules::AllMatches`] if you don't need unsafe.
    ///
    /// ---
    ///
    /// # Feature Flag:
    /// To be safe and make sure you know what you are doing, you need to use the feature flag:
    ///
    /// ```bash
    /// cargo add magical_rs --features unsafe_context
    /// ```
    /// By default, this feature is disabled and not compiled.
    ///
    /// ---
    ///
    /// # Examples:
    /// ```rust
    /// fn any_unsafe_fn() {
    ///     use core::slice;
    ///     use magical_rs::magical::magic_custom::match_types_custom;
    ///     use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};
    ///
    ///     #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    ///     enum MagicKind {
    ///         MoeMoe,
    ///         UnknownFallback,
    ///     }
    ///
    ///     fn is_shoujo_girl(data: *const ()) -> bool {
    ///         unsafe {
    ///             let slice_ptr = data.cast::<u8>();
    ///             let slice = slice::from_raw_parts(slice_ptr, 100);
    ///
    ///             slice.starts_with(b"MagicalGirl")
    ///         }
    ///     }
    ///
    ///     fn is_not_shoujo_girl(data: *const ()) -> bool {
    ///         unsafe {
    ///             let slice_ptr = data.cast::<u8>();
    ///             let slice = slice::from_raw_parts(slice_ptr, 100);
    ///
    ///             !slice.starts_with(b"MagicalGirl")
    ///         }
    ///     }
    ///
    ///     let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
    ///         signatures: &[],
    ///         offsets: &[],
    ///         max_bytes_read: 200,
    ///         kind: MagicKind::MoeMoe,
    ///         rules: CustomMatchRules::AnyMatchesUnsafe(
    ///                 &[is_shoujo_girl, is_not_shoujo_girl]),
    ///     }];
    ///
    ///     let result = match_types_custom(b"MagicalGirl", rules, MagicKind::UnknownFallback);
    ///
    ///     assert_eq!(result, MagicKind::MoeMoe);
    ///     assert_ne!(result, MagicKind::UnknownFallback);
    /// }
    /// ```
    ///
    /// ---
    ///
    /// # Unsafe Context:
    /// Basically, [`CustomMatchRules::AnyMatchesUnsafe`] supports `no_std` context.
    ///
    /// It only requires at least Rust's [`core`].
    ///
    /// You can also skip using [`core`] if you don't use [`core::slice`].
    ///
    /// The above sample uses [`core`] and [`core::slice`] as an assumption of minimal
    /// environments like kernel, embedded.
    #[cfg(feature = "unsafe_context")]
    AnyMatchesUnsafe(&'a [unsafe fn(ptr_data: *const ()) -> bool]),

    /// # Safety:
    ///
    /// Similar to [`CustomMatchRules::AllMatches`] but supports `unsafe`
    ///
    /// Here, you can pass in multiple unsafe function pointers.
    ///
    /// However, these functions will work on `AND` logic, so all
    /// functions returning `true` is needed.
    ///
    /// Since it's operating in an `unsafe` context, you need to control
    /// everything and there are no constraints or protections from the
    /// Rust compiler.
    ///
    /// It can cause segmentation faults, memory leaks, or worse
    /// undefined beavior.
    ///
    /// Becareful and use them when you need to combine multiple unsafe function
    /// pointers.
    ///
    /// ---
    ///
    /// # Feature Flag:
    /// To be safe and make sure you know what you are doing, you need to use the feature flag:
    ///
    /// ```bash
    /// cargo add magical_rs --features unsafe_context
    /// ```
    /// By default, this feature is disabled and not compiled.
    ///
    /// ---
    ///
    /// # Examples:
    /// ```rust
    /// fn any_unsafe_fn() {
    ///     use core::slice;
    ///     use magical_rs::magical::magic_custom::match_types_custom;
    ///     use magical_rs::magical::magic_custom::{CustomMatchRules, MagicCustom};
    ///
    ///     #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    ///     enum MagicKind {
    ///         MoeMoe,
    ///         UnknownFallback,
    ///     }
    ///
    ///     fn is_shoujo_girl(data: *const ()) -> bool {
    ///         unsafe {
    ///             let slice_ptr = data.cast::<u8>();
    ///             let slice = slice::from_raw_parts(slice_ptr, 100);
    ///
    ///             slice.starts_with(b"MagicalGirl")
    ///         }
    ///     }
    ///
    ///     fn is_not_shoujo_girl(data: *const ()) -> bool {
    ///         unsafe {
    ///             let slice_ptr = data.cast::<u8>();
    ///             let slice = slice::from_raw_parts(slice_ptr, 100);
    ///
    ///             !slice.starts_with(b"MagicalGirl")
    ///         }
    ///     }
    ///
    ///     let rules: &[MagicCustom<MagicKind>] = &[MagicCustom {
    ///         signatures: &[],
    ///         offsets: &[],
    ///         max_bytes_read: 200,
    ///         kind: MagicKind::MoeMoe,
    ///         rules: CustomMatchRules::AllMatchesUnsafe(
    ///                 &[is_shoujo_girl, is_not_shoujo_girl]),
    ///     }];
    ///
    ///     let result = match_types_custom(b"MagicalGirl", rules, MagicKind::UnknownFallback);
    ///
    ///     assert_ne!(result, MagicKind::MoeMoe);
    ///     assert_eq!(result, MagicKind::UnknownFallback);
    /// }
    /// ```
    ///
    /// ---
    ///
    /// # Unsafe Context:
    /// Basically, [`CustomMatchRules::AllMatchesUnsafe`] supports `no_std` context.
    ///
    /// It only requires at least Rust's [`core`].
    ///
    /// You can also skip using [`core`] if you don't use [`core::slice`].
    ///
    /// The above sample uses [`core`] and [`core::slice`] as an assumption of minimal
    /// environments like kernel, embedded.
    #[cfg(feature = "unsafe_context")]
    AllMatchesUnsafe(&'a [unsafe fn(ptr_data: *const ()) -> bool]),
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

#[must_use]
#[inline]
#[doc(hidden)]
#[cfg(feature = "unsafe_context")]
const fn __from_ref<T: ?Sized>(r: &T) -> *const T {
    r
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

            #[cfg(feature = "unsafe_context")]
            CustomMatchRules::WithFnUnsafe { func, .. } => {
                /*
                 * This is a minimal type conversion.
                 * You have full control over every byte.
                 * Be careful.
                 */
                let raw = __from_ref::<[u8]>(bytes).cast::<()>();

                unsafe { func(raw) }
            }
            #[cfg(feature = "unsafe_context")]
            CustomMatchRules::AnyMatchesUnsafe(funcs) => funcs.iter().any(|&func| {
                /*
                 * This is a minimal type conversion.
                 * You have full control over every byte.
                 * Be careful.
                 */
                let raw = __from_ref::<[u8]>(bytes).cast::<()>();

                unsafe { func(raw) }
            }),
            #[cfg(feature = "unsafe_context")]
            CustomMatchRules::AllMatchesUnsafe(funcs) => funcs.iter().all(|&func| {
                /*
                 * This is a minimal type conversion.
                 * You have full control over every byte.
                 * Be careful.
                 */
                let raw = __from_ref::<[u8]>(bytes).cast::<()>();

                unsafe { func(raw) }
            }),
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
    (signatures: [$($sig:expr),* $(,)?],
    offsets: [$($offsets:expr),* $(,)?],
    max_bytes_read: $max_bytes_read:expr,
    kind: $kind:expr,
    rules: $rules:expr) => {
        $crate::magical::magic_custom::MagicCustom {
            signatures: &[$($sig),*],
            offsets: &[$($offsets),*],
            max_bytes_read: $max_bytes_read,
            kind: $kind,
            rules: $rules
        }
    };
}

/// Macros with sugar-coated syntax for [`match_types_custom`]
#[macro_export]
macro_rules! match_custom {
    (bytes: $bytes:expr,
    rules: [$($rules:expr),+$(,)?],
    fallback: $fallback:expr) => {
     $crate::magical::magic_custom::match_types_custom($bytes, &[$($rules),+], $fallback)
    };
}
