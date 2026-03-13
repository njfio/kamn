use std::{env, fs, path::PathBuf};

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

#[test]
fn regression_did_root_is_extracted_to_bounded_modules() {
    let root = read("crates/kamn-core/src/did.rs");
    let root_lines = root.lines().count();
    assert!(
        root_lines <= 180,
        "did.rs should be a thin shell after extraction, got {root_lines} lines"
    );
    assert!(root.contains("mod federated;"), "root should declare federated module");
    assert!(root.contains("mod document;"), "root should declare document module");
    assert!(root.contains("#[cfg(test)]\nmod tests;"), "root should route tests into sibling module");
    assert!(
        !root.contains("pub trait FederatedDidTrustStore"),
        "trust-store types should move out of the root shell"
    );
    assert!(
        !root.contains("pub enum DidDocumentError"),
        "did document errors should move out of the root shell"
    );
    assert!(
        !root.contains("#[cfg(test)]\nmod tests {"),
        "inline tests should move out of the root shell"
    );

    let federated = read("crates/kamn-core/src/did/federated.rs");
    assert!(
        federated.contains("pub trait FederatedDidTrustStore"),
        "federated module should own trust-store contracts"
    );

    let document = read("crates/kamn-core/src/did/document.rs");
    assert!(
        document.contains("pub enum DidDocumentError"),
        "document module should own DID document helpers"
    );

    let tests = read("crates/kamn-core/src/did/tests.rs");
    assert!(
        tests.contains("regression_requires_constant_time_agent_did_key_binding_compare"),
        "tests module should preserve the existing key-binding regression"
    );
}
