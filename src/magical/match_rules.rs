pub enum MatchRules {
    Default,
    WithFn(fn(bytes: &[u8]) -> bool),
}
