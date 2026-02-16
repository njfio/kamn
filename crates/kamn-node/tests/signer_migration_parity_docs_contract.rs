const SIGNER_LIFECYCLE_DOC: &str = include_str!("../../../docs/architecture/signer-lifecycle.md");

const REQUIRED_MIGRATION_PARITY_MARKERS: &[&str] = &[
    "signer_migration_parity_matrix_version=kamn.signer.migration-parity-matrix.v1",
    "matrix_case=primary_env_local_happy_path",
    "matrix_case=secondary_env_local_happy_path",
    "matrix_case=primary_managed_external_happy_path",
    "matrix_case=secondary_managed_external_disallowed",
    "legacy_behavior_diff_guard=enabled",
];

#[test]
fn docs_signer_lifecycle_declares_migration_parity_matrix() {
    assert!(
        SIGNER_LIFECYCLE_DOC.contains("### Migration Parity Matrix"),
        "signer lifecycle docs must declare migration parity matrix section"
    );
    for marker in REQUIRED_MIGRATION_PARITY_MARKERS {
        assert!(
            SIGNER_LIFECYCLE_DOC.contains(marker),
            "signer lifecycle docs missing migration parity marker: {marker}"
        );
    }
}

#[test]
fn docs_signer_lifecycle_declares_parity_guard_commands() {
    assert!(
        SIGNER_LIFECYCLE_DOC.contains(
            "cargo test -p kamn-node main_tests::signer_tests::functional_signer_migration_profile_key_source_parity_matrix -- --exact --nocapture"
        ),
        "signer lifecycle docs must declare functional parity matrix guard command"
    );
    assert!(
        SIGNER_LIFECYCLE_DOC.contains(
            "cargo test -p kamn-node --test signer_migration_parity_docs_contract -- --nocapture"
        ),
        "signer lifecycle docs must declare docs parity guard command"
    );
}
