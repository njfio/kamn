use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_phase1_required_paths_exist() {
    let root = repo_root();
    let required_paths = [
        "crates/kamn-agent-lib/Cargo.toml",
        "crates/kamn-agent-lib/src/lib.rs",
        "crates/kamn-agent-lib/src/identity.rs",
        "crates/kamn-agent-lib/src/auth.rs",
        "crates/kamn-agent-lib/src/envelope.rs",
        "crates/kamn-agent-lib/src/client.rs",
        "crates/kamn-agent-lib/src/kolme.rs",
        "crates/kamn-agent-lib/src/nonce.rs",
        "crates/kamn-agent-lib/src/errors.rs",
        "crates/kamn-agent-lib/tests/auth_roundtrip.rs",
        "crates/kamn-agent-lib/tests/envelope_construction.rs",
        "crates/kamn-agent-lib/tests/kolme_verification.rs",
    ];

    for path in required_paths {
        assert!(root.join(path).is_file(), "required path missing: {path}");
    }
}

#[test]
fn spec_c02_workspace_registers_kamn_agent_lib_member() {
    let cargo_toml = std::fs::read_to_string(repo_root().join("Cargo.toml"))
        .expect("workspace Cargo.toml should be readable");
    assert!(cargo_toml.contains("\"crates/kamn-agent-lib\""));
}
