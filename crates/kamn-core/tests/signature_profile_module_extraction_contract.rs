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

#[test]
fn signature_profile_root_is_extracted_into_bounded_modules() {
    let root = repo_root();
    let signature_root = root.join("crates/kamn-core/src/signature_profile.rs");
    let root_source = read(&signature_root);
    let module_dir = root.join("crates/kamn-core/src/signature_profile");

    for module_file in ["models.rs", "encoding.rs", "service_auth.rs", "fixtures.rs", "tests.rs"] {
        let path = module_dir.join(module_file);
        assert!(path.exists(), "expected extracted module {} to exist", path.display());
        assert!(count_lines(&path) <= 200, "expected {} to stay within 200 LOC", path.display());
    }

    for marker in [
        "mod models;",
        "mod encoding;",
        "mod service_auth;",
        "mod fixtures;",
        "#[cfg(test)] mod tests;",
    ] {
        assert!(root_source.contains(marker), "expected root shell to contain marker `{marker}`");
    }

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

    assert!(
        count_lines(&signature_root) <= 180,
        "expected signature_profile root shell to stay within 180 LOC"
    );
}
