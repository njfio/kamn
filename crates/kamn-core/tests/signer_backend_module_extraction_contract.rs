use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/signer_backend.rs";
const ROOT_BUDGET: usize = 180;
const MODULES: &[&str] = &[
    "src/signer_backend/request.rs",
    "src/signer_backend/env.rs",
    "src/signer_backend/provider_policy.rs",
    "src/signer_backend/backends.rs",
    "src/signer_backend/router.rs",
    "src/signer_backend/errors.rs",
    "src/signer_backend/tests.rs",
];

#[test]
fn signer_backend_root_is_extracted() {
    let root_path = manifest_path(ROOT);
    let root = fs::read_to_string(&root_path).expect("root module should exist");
    assert_root_budget(&root_path, &root);
    assert_modules_exist();
    assert_root_markers_present(&root);
    assert_legacy_markers_removed(&root);
}

fn manifest_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn assert_root_budget(root_path: &Path, root: &str) {
    let root_lines = root.lines().count();
    assert!(
        root_lines <= ROOT_BUDGET,
        "{} should be a thin shell after extraction; found {root_lines} lines",
        root_path.display()
    );
}

fn assert_modules_exist() {
    for module in MODULES {
        let module_path = manifest_path(module);
        assert!(
            Path::new(&module_path).exists(),
            "missing extracted module: {}",
            module_path.display()
        );
    }
}

fn assert_root_markers_present(root: &str) {
    for marker in marker_list() {
        assert!(
            root.contains(marker),
            "root module missing extraction marker `{marker}`"
        );
    }
}

fn assert_legacy_markers_removed(root: &str) {
    for legacy_marker in legacy_marker_list() {
        assert!(
            !root.contains(legacy_marker),
            "root module still contains legacy inline content marker `{legacy_marker}`"
        );
    }
}

fn marker_list() -> [&'static str; 8] {
    [
        "mod request;",
        "mod env;",
        "mod provider_policy;",
        "mod backends;",
        "mod router;",
        "mod errors;",
        "#[cfg(test)]",
        "mod tests;",
    ]
}

fn legacy_marker_list() -> [&'static str; 7] {
    [
        "pub struct SigningRequest",
        "pub enum SecureSignerProvider",
        "pub struct LocalSignerBackend",
        "pub struct SecureSignerBackend",
        "pub struct SignerBackendRouter",
        "pub enum SignerBackendError",
        "mod tests {",
    ]
}
