use crate::support::{assert_not_contains_all, read_repo_file};

#[test]
fn main_module_extraction_contract_removes_inline_runtime_mode_profile_impls() {
    let main_rs = read_repo_file("src/main.rs");
    assert_not_contains_all(
        &main_rs,
        &[
            ("struct OutputMode {", "main.rs should not keep inline output mode struct"),
            ("enum OutputModeKind {", "main.rs should not keep inline output mode kind enum"),
            ("impl OutputMode {", "main.rs should not keep inline output mode impl"),
            ("struct RuntimeMode {", "main.rs should not keep inline runtime mode struct"),
            ("enum RuntimeModeKind {", "main.rs should not keep inline runtime mode kind enum"),
            ("impl RuntimeMode {", "main.rs should not keep inline runtime mode impl"),
            ("enum DiagnosticsMode {", "main.rs should not keep inline diagnostics mode enum"),
            ("impl DiagnosticsMode {", "main.rs should not keep inline diagnostics mode impl"),
            ("enum LocalProfile {", "main.rs should not keep inline local profile enum"),
            ("impl LocalProfile {", "main.rs should not keep inline local profile impl"),
        ],
    );
}
