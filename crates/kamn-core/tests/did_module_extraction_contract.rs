use std::{fs, path::PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).expect("source must exist")
}

fn assert_root_shell(root: &str) {
    let root_lines = root.lines().count();
    assert!(root_lines <= 180, "did.rs should be a thin shell after extraction, got {root_lines} lines");
    for marker in ["mod federated;", "mod document;", "#[cfg(test)]\nmod tests;"] {
        assert!(root.contains(marker), "root missing marker: {marker}");
    }
}

fn assert_root_exclusions(root: &str) {
    for marker in [
        "pub trait FederatedDidTrustStore",
        "pub enum DidDocumentError",
        "#[cfg(test)]\nmod tests {",
    ] {
        assert!(!root.contains(marker), "root still contains moved marker: {marker}");
    }
}

fn assert_extracted_modules() {
    let federated = read("crates/kamn-core/src/did/federated.rs");
    assert!(federated.contains("pub trait FederatedDidTrustStore"));

    let document = read("crates/kamn-core/src/did/document.rs");
    assert!(document.contains("pub enum DidDocumentError"));

    let tests = read("crates/kamn-core/src/did/tests.rs");
    assert!(tests.contains("regression_requires_constant_time_agent_did_key_binding_compare"));
}

#[test]
fn regression_did_root_is_extracted_to_bounded_modules() {
    let root = read("crates/kamn-core/src/did.rs");
    assert_root_shell(&root);
    assert_root_exclusions(&root);
    assert_extracted_modules();
}
