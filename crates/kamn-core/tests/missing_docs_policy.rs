const CORE_LIB: &str = include_str!("../src/lib.rs");

#[test]
fn kamn_core_declares_missing_docs_warning_policy() {
    assert!(CORE_LIB.contains("#![warn(missing_docs)]"));
}
