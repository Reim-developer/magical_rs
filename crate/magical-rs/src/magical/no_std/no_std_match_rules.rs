pub enum NoStdMatchRules {
    Default,
    WithFn(fn(bytes: &[u8]) -> bool),
}
