use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate dir should have workspace root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn source_path(relative: &str) -> PathBuf {
    repo_root().join(relative)
}

fn assert_root_shell_markers(root_text: &str) {
    for marker in [
        "mod entrypoints;",
        "mod layout;",
        "mod validation;",
        "mod error_mapping;",
        "mod tests;",
    ] {
        assert!(root_text.contains(marker), "expected root shell marker `{marker}`");
    }
}

fn assert_root_shell_budget(root_path: &Path, root_text: &str) {
    let line_count = root_text.lines().count();
    assert!(
        line_count <= 180,
        "expected {} to be <= 180 lines after extraction, got {line_count}",
        root_path.display()
    );
}

fn assert_root_shell_excludes_inline_markers(root_text: &str) {
    for marker in [
        "fn bootstrap_from_state_version_with_transport_profile(",
        "fn resolve_runtime_persistence_layout(",
        "fn validate_runtime_persistence_layout(",
        "fn map_content_store_validation_error(",
        "mod tests {",
    ] {
        assert!(
            !root_text.contains(marker),
            "expected root shell to exclude inline marker `{marker}`"
        );
    }
}

fn assert_expected_modules(root: &Path) {
    for relative in [
        "entrypoints.rs",
        "layout.rs",
        "validation.rs",
        "error_mapping.rs",
        "tests.rs",
    ] {
        let module_path = root.join(relative);
        assert!(
            module_path.exists(),
            "expected extracted module {}",
            module_path.display()
        );
        let module_text = read(&module_path);
        assert!(
            module_text.lines().count() <= 200,
            "expected {} to stay within 200 lines, got {}",
            module_path.display(),
            module_text.lines().count()
        );
    }
}

#[test]
fn bootstrap_root_is_extracted() {
    let root_path = source_path("crates/kamn-core/src/bootstrap.rs");
    let root_text = read(&root_path);

    assert_root_shell_markers(&root_text);
    assert_root_shell_budget(&root_path, &root_text);
    assert_root_shell_excludes_inline_markers(&root_text);
    assert_expected_modules(&source_path("crates/kamn-core/src/bootstrap"));
}
