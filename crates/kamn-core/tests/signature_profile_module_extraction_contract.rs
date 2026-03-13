use std::{
    fs,
    path::{Path, PathBuf},
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
}

fn count_lines(path: &Path) -> usize {
    read(path).lines().count()
}

fn required_module_paths(root: &Path) -> [PathBuf; 5] {
    let module_dir = root.join("crates/kamn-core/src/signature_profile");
    [
        module_dir.join("models.rs"),
        module_dir.join("encoding.rs"),
        module_dir.join("service_auth.rs"),
        module_dir.join("fixtures.rs"),
        module_dir.join("tests.rs"),
    ]
}

fn assert_required_modules_exist(paths: &[PathBuf]) {
    for path in paths {
        assert!(path.exists(), "expected extracted module {} to exist", path.display());
        assert!(count_lines(path) <= 200, "expected {} to stay within 200 LOC", path.display());
    }
}

fn assert_root_markers(root_source: &str) {
    for marker in [
        "mod models;",
        "mod encoding;",
        "mod service_auth;",
        "mod fixtures;",
        "#[cfg(test)] mod tests;",
    ] {
        assert!(root_source.contains(marker), "expected root shell to contain marker `{marker}`");
    }
}

fn assert_moved_markers_absent(root_source: &str) {
    for moved_marker in [
        "pub enum ServiceAuthSignatureError",
        "pub struct SignatureProfileCompatibilityFixture",
        "pub fn service_auth_sign_with_private_key_hex",
        "pub fn signature_profile_compatibility_fixtures_for_fields",
        "mod tests {",
    ] {
        assert!(
            !root_source.contains(moved_marker),
            "expected root shell to move `{moved_marker}` into extracted modules"
        );
    }
}

#[test]
fn signature_profile_root_is_extracted_into_bounded_modules() {
    let root = repo_root();
    let signature_root = root.join("crates/kamn-core/src/signature_profile.rs");
    let root_source = read(&signature_root);
    let required = required_module_paths(&root);

    assert_required_modules_exist(&required);
    assert_root_markers(&root_source);
    assert_moved_markers_absent(&root_source);
    assert!(count_lines(&signature_root) <= 180, "expected signature_profile root shell to stay within 180 LOC");
}
