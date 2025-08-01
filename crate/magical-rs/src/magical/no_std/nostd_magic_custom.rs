#[derive(Clone, Copy)]
pub enum CustomMatchRules {
    Default,
    WithFn(fn(bytes: &[u8]) -> bool),
}

#[derive(Clone, Copy)]
pub struct MagicCustom<K> {
    pub signatures: &'static [&'static [u8]],
    pub offsets: &'static [usize],
    pub max_bytes_read: usize,
    pub kind: K,
    pub rules: CustomMatchRules,
}

impl<K: Clone> MagicCustom<K> {
    #[inline]
    #[must_use]
    fn matches_custom(&self, bytes: &[u8]) -> bool {
        match &self.rules {
            CustomMatchRules::Default => self.signatures.iter().any(|&signature| {
                self.offsets.iter().any(|&offset| {
                    let offset_end = offset + signature.len();

                    bytes.len() >= offset_end && &bytes[offset..offset_end] == signature
                })
            }),

            CustomMatchRules::WithFn(func) => func(bytes),
        }
    }
}

#[inline]
#[must_use]
pub fn match_types_custom<K: Clone>(bytes: &[u8], rules: &[MagicCustom<K>], fallback: K) -> K {
    rules
        .iter()
        .find(|rule| rule.matches_custom(bytes))
        .map_or(fallback, |rule| rule.kind.clone())
}
