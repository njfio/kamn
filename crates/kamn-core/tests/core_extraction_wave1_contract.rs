use std::fs;
use std::path::PathBuf;

fn repo_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");
    fs::read_to_string(root.join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn spec_c01_workspace_registers_wave1_extraction_crates() {
    let workspace = repo_file("Cargo.toml");
    for member in [
        "\"crates/kamn-crypto\"",
        "\"crates/kamn-bridges\"",
        "\"crates/kamn-data-layer\"",
    ] {
        assert!(
            workspace.contains(member),
            "expected workspace members to include {member}"
        );
    }
}

#[test]
fn spec_c02_kamn_core_facade_modules_reexport_extracted_crates() {
    let direct_message_facade = repo_file("crates/kamn-core/src/direct_message_crypto.rs");
    assert!(
        direct_message_facade.contains("kamn_crypto::direct_message_crypto"),
        "expected direct-message crypto module to be re-exported from kamn-crypto"
    );

    let bridge_receipt_facade = repo_file("crates/kamn-core/src/cross_chain_receipt.rs");
    assert!(
        bridge_receipt_facade.contains("kamn_bridges::cross_chain_receipt"),
        "expected cross-chain receipt module to be re-exported from kamn-bridges"
    );

    let data_layer_hashing_facade = repo_file("crates/kamn-core/src/data_layer_hashing.rs");
    assert!(
        data_layer_hashing_facade.contains("kamn_data_layer::data_layer_hashing"),
        "expected data-layer hashing module to be re-exported from kamn-data-layer"
    );
}

#[test]
fn spec_c03_extracted_module_files_exist_in_focused_crates() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..");

    for path in [
        "crates/kamn-crypto/src/direct_message_crypto.rs",
        "crates/kamn-bridges/src/cross_chain_receipt.rs",
        "crates/kamn-data-layer/src/data_layer_hashing.rs",
    ] {
        assert!(
            root.join(path).is_file(),
            "expected extracted module file to exist: {path}"
        );
    }
}
