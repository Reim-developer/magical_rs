#[cfg(feature = "magical_async_dyn")]
use {core::pin::Pin, std::any::Any, std::boxed::Box, std::future::Future, std::sync::Arc};

#[cfg(feature = "magical_async_dyn")]
type AsyncMatcher =
    Box<dyn Fn(&[u8]) -> Pin<Box<dyn Future<Output = bool> + Send + 'static>> + Send + Sync>;

/// Same as `DynMagic` but with async support.
///
/// Be careful and use it with extreme caution because
/// it is very powerful feature but use it when you
/// know what you are doing.
///
/// An asynchronouse runtime is required to use this feature.
///
/// However, you can use any asynchronous run-time like Tokio.
#[cfg(feature = "magical_async_dyn")]
pub struct AsyncDynMagic {
    matcher: AsyncMatcher,
    kind: Arc<dyn Any + Send + Sync>,
    max_bytes_read: usize,
}

#[cfg(feature = "magical_async_dyn")]
impl AsyncDynMagic {
    #[must_use]
    pub fn new<F, Fut, K>(matcher: F, kind: K, max_bytes_read: usize) -> Self
    where
        F: Fn(&[u8]) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = bool> + Send + 'static,
        K: 'static + Send + Sync,
    {
        Self {
            matcher: Box::new(move |bytes| Box::pin(matcher(bytes))),
            kind: Arc::new(kind),
            max_bytes_read,
        }
    }

    /// Check if the input bytes match this rules.
    #[must_use]
    pub async fn matches(&self, bytes: &[u8]) -> bool {
        (self.matcher)(bytes).await
    }

    #[must_use]
    /// Returns a reference to the stored [`AsyncDynMagic::kind`], as `Arc<dyn Any + Send + Sync>`
    pub fn kind_arc(&self) -> Arc<dyn Any + Send + Sync> {
        self.kind.clone()
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

#[cfg(feature = "magical_async_dyn")]
#[must_use]
pub async fn match_dyn_types<'a>(bytes: &[u8], rules: &'a [AsyncDynMagic]) -> Option<&'a dyn Any> {
    for rule in rules {
        if rule.matches(bytes).await {
            return Some(rule.kind());
        }
    }

    None
}

#[cfg(feature = "magical_async_dyn")]
#[must_use]
/// Shorthand: returns the first matching [`AsyncDynMagic::kind`] downcasted to `&T`
///
/// This is the asynchronous version of function `match_dyn_types_as` in `DynMagicCustom`
pub async fn match_dyn_types_as<'a, T: 'static>(
    bytes: &[u8],
    rules: &'a [AsyncDynMagic],
) -> Option<&'a T> {
    match_dyn_types(bytes, rules).await?.downcast_ref::<T>()
}

#[cfg(feature = "magical_async_dyn")]
#[must_use]
/// Returns all matching [`AsyncDynMagic::kind`]
///
/// This is the asynchronous version of function `match_dyn_types_all` in `DynMagicCustom`
pub async fn match_dyn_types_all(
    bytes: &[u8],
    rules: &[AsyncDynMagic],
) -> Vec<Arc<dyn Any + Send + Sync>> {
    let mut results = Vec::new();

    for rule in rules {
        if rule.matches(bytes).await {
            results.push(rule.kind_arc());
        }
    }

    results
}
