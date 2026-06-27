use std::path::{Path, PathBuf};

const GROUP_CHANNEL_SELECTOR_FILE: &str =
    "crates/kamn-core/src/group_channel_crypto/engine/sealing/encrypt.rs";
const LEGACY_GROUP_CHANNEL_SELECTOR_FILE: &str = "crates/kamn-core/src/group_channel_crypto.rs";
const NONCE_GUARD: &str = "if nonce == 0";

#[test]
fn critical_path_mutation_gate_uses_extracted_group_channel_selector() {
    let script = read_repo_file("scripts/ci/run_critical_path_mutation_gate.sh");
    let source = read_repo_file(GROUP_CHANNEL_SELECTOR_FILE);

    assert!(
        source.contains(NONCE_GUARD),
        "expected extracted group-channel encrypt module to own nonce guard",
    );
    assert!(
        script.contains(GROUP_CHANNEL_SELECTOR_FILE),
        "group-channel mutation selector must follow extracted encrypt module",
    );
    assert!(
        !script.contains(&format!("grep -n '{NONCE_GUARD}' {LEGACY_GROUP_CHANNEL_SELECTOR_FILE}")),
        "group-channel mutation selector must not drift back to the parent module",
    );
}

fn read_repo_file(relative_path: &str) -> String {
    std::fs::read_to_string(repo_root().join(relative_path))
        .unwrap_or_else(|error| panic!("failed to read {relative_path}: {error}"))
}

fn repo_root() -> PathBuf {
    manifest_dir()
        .parent()
        .and_then(Path::parent)
        .expect("kamn-core manifest should live under crates/kamn-core")
        .to_path_buf()
}

fn manifest_dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}
