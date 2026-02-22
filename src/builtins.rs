/// Builtin names expected by compatibility contract.
pub fn builtin_names() -> &'static [&'static str] {
    // TODO(step-5): implement builtin function bodies and registration table.
    &["len", "first", "last", "rest", "push", "puts"]
}
