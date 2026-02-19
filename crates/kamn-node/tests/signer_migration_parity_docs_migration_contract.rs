const SHARED_DOC_HARNESS_SOURCE: &str = include_str!("./node_runtime_cli_docs.rs");

#[test]
fn regression_signer_migration_parity_docs_migrated_into_shared_harness_matrix() {
    assert!(
        SHARED_DOC_HARNESS_SOURCE
            .contains("migration_signer_lifecycle_docs_declares_migration_parity_matrix"),
        "shared harness must retain signer lifecycle migration parity markers"
    );
    assert!(
        SHARED_DOC_HARNESS_SOURCE
            .contains("migration_signer_lifecycle_docs_declares_parity_guard_commands"),
        "shared harness must retain signer lifecycle parity guard command markers"
    );
    assert!(
        SHARED_DOC_HARNESS_SOURCE.contains(
            "cargo test -p kamn-node --test node_runtime_cli_docs migration_signer_lifecycle_docs_declares_parity_guard_commands -- --exact --nocapture"
        ),
        "shared harness must enforce updated signer lifecycle guard command reference"
    );
}
