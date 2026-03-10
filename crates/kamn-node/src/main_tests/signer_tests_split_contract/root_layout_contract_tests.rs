use super::support::read_repo_file;

const ROOT_FILE: &str = "src/main_tests/signer_tests.rs";

#[test]
fn spec_c01_signer_root_declares_extracted_modules() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "mod signer_direct_profile_contract_tests;",
        "mod signer_preflight_nonce_contract_tests;",
        "mod signer_managed_external_contract_tests;",
        "mod support;",
    ] {
        assert!(
            source.contains(marker),
            "signer_tests.rs should declare extracted module marker: {marker}"
        );
    }
}

#[test]
fn spec_c02_signer_root_removes_moved_markers() {
    let source = read_repo_file(ROOT_FILE);
    for marker in [
        "fn unit_kolme_live_signer_builds_direct_signed_wire_payload()",
        "fn functional_signer_migration_profile_key_source_parity_matrix()",
        "fn integration_runtime_kolme_live_renders_managed_external_signer_selection_markers()",
        "fn integration_kolme_live_signer_preflight_quorum_profile_matrix_paths()",
        "fn integration_kolme_live_nonce_resolver_retries_unavailable_then_succeeds()",
        "fn regression_kolme_live_managed_external_backend_response_requires_signer_public_key_marker()",
        "fn regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch()",
        "fn regression_kolme_live_managed_external_backend_unavailable_maps_reason_code()",
    ] {
        assert!(
            !source.contains(marker),
            "signer_tests.rs should not keep moved signer marker: {marker}"
        );
    }
}
