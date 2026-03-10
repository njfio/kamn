use std::fs;

const ROOT_FILE: &str = "src/main_tests/signer_tests.rs";
const DIRECT_PROFILE_MODULE_FILE: &str =
    "src/main_tests/signer_tests/signer_direct_profile_contract_tests.rs";
const DIRECT_SIGNATURE_FILE: &str =
    "src/main_tests/signer_tests/signer_direct_profile_contract_tests/direct_signature_contract_tests.rs";
const PROFILE_SELECTION_FILE: &str =
    "src/main_tests/signer_tests/signer_direct_profile_contract_tests/profile_selection_contract_tests.rs";
const PREFLIGHT_NONCE_MODULE_FILE: &str =
    "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests.rs";
const PREFLIGHT_POLICY_FILE: &str =
    "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests/preflight_policy_contract_tests.rs";
const NONCE_RESOLVER_FILE: &str =
    "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests/nonce_resolver_contract_tests.rs";
const MANAGED_EXTERNAL_MODULE_FILE: &str =
    "src/main_tests/signer_tests/signer_managed_external_contract_tests.rs";
const MANAGED_EXTERNAL_BACKEND_FILE: &str =
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_contract_tests.rs";
const MANAGED_EXTERNAL_REASON_CODES_FILE: &str =
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_reason_code_contract_tests.rs";
const SHARED_SUPPORT_FILE: &str = "src/main_tests/signer_tests/support.rs";
const ROOT_MAX_LINES: usize = 200;
const EXTRACTED_MAX_LINES: usize = 200;

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn line_count(path: &str) -> usize {
    read_repo_file(path).lines().count()
}

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
        "fn unit_kolme_live_signer_adapter_signs_and_verifies_runtime_message()",
        "fn functional_signer_migration_profile_key_source_parity_matrix()",
        "fn integration_kolme_live_signer_preflight_rejects_non_failover_rotation_regression()",
        "fn integration_kolme_live_signer_preflight_quorum_profile_matrix_paths()",
        "fn integration_kolme_live_nonce_resolver_fetches_next_nonce()",
        "fn integration_kolme_live_nonce_resolver_retries_unavailable_then_succeeds()",
        "fn regression_kolme_live_nonce_resolver_rejects_malformed_response()",
        "fn regression_kolme_live_managed_external_requires_key_reference_env_marker()",
        "fn regression_kolme_live_managed_external_backend_response_requires_signer_public_key_marker()",
        "fn regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch()",
        "fn regression_kolme_live_managed_external_backend_timeout_maps_reason_code()",
        "fn regression_kolme_live_managed_external_backend_malformed_response_maps_reason_code()",
        "fn regression_kolme_live_managed_external_backend_unavailable_maps_reason_code()",
    ] {
        assert!(
            !source.contains(marker),
            "signer_tests.rs should not keep moved signer marker: {marker}"
        );
    }
}

#[test]
fn spec_c03_signer_extracted_modules_exist_and_own_coverage() {
    let direct_profile = read_repo_file(DIRECT_PROFILE_MODULE_FILE);
    let direct_signature = read_repo_file(DIRECT_SIGNATURE_FILE);
    let profile_selection = read_repo_file(PROFILE_SELECTION_FILE);
    let preflight_nonce = read_repo_file(PREFLIGHT_NONCE_MODULE_FILE);
    let preflight_policy = read_repo_file(PREFLIGHT_POLICY_FILE);
    let nonce_resolver = read_repo_file(NONCE_RESOLVER_FILE);
    let managed_external = read_repo_file(MANAGED_EXTERNAL_MODULE_FILE);
    let backend = read_repo_file(MANAGED_EXTERNAL_BACKEND_FILE);
    let reason_codes = read_repo_file(MANAGED_EXTERNAL_REASON_CODES_FILE);
    let support = read_repo_file(SHARED_SUPPORT_FILE);

    for marker in [
        "mod direct_signature_contract_tests;",
        "mod profile_selection_contract_tests;",
    ] {
        assert!(
            direct_profile.contains(marker),
            "signer_direct_profile_contract_tests.rs should declare {marker}"
        );
    }
    for marker in [
        "mod preflight_policy_contract_tests;",
        "mod nonce_resolver_contract_tests;",
    ] {
        assert!(
            preflight_nonce.contains(marker),
            "signer_preflight_nonce_contract_tests.rs should declare {marker}"
        );
    }
    for marker in [
        "mod backend_contract_tests;",
        "mod backend_reason_code_contract_tests;",
    ] {
        assert!(
            managed_external.contains(marker),
            "signer_managed_external_contract_tests.rs should declare {marker}"
        );
    }

    for marker in [
        "fn unit_kolme_live_signer_builds_direct_signed_wire_payload()",
        "fn unit_kolme_live_signer_adapter_signs_and_verifies_runtime_message()",
        "fn integration_kolme_live_signer_vector_probe_contract()",
    ] {
        assert!(
            direct_signature.contains(marker),
            "direct signer file should include moved marker: {marker}"
        );
    }
    for marker in [
        "fn functional_signer_migration_profile_key_source_parity_matrix()",
        "fn unit_kolme_live_signer_profile_defaults_to_primary_key_env()",
        "fn integration_runtime_kolme_live_renders_secondary_signer_selection_markers()",
        "fn integration_runtime_kolme_live_renders_managed_external_signer_selection_markers()",
    ] {
        assert!(
            profile_selection.contains(marker),
            "profile selection file should include moved marker: {marker}"
        );
    }
    for marker in [
        "fn integration_kolme_live_signer_preflight_rejects_non_failover_rotation_regression()",
        "fn integration_kolme_live_signer_preflight_quorum_profile_matrix_paths()",
        "fn regression_kolme_live_signer_requires_primary_key_env_value()",
        "fn regression_issue_2279_kolme_live_signer_rejects_fallback_private_key_env_path()",
    ] {
        assert!(
            preflight_policy.contains(marker),
            "preflight policy file should include moved marker: {marker}"
        );
    }
    for marker in [
        "fn integration_kolme_live_nonce_resolver_fetches_next_nonce()",
        "fn integration_kolme_live_nonce_resolver_retries_unavailable_then_succeeds()",
        "fn regression_kolme_live_nonce_resolver_rejects_malformed_response()",
    ] {
        assert!(
            nonce_resolver.contains(marker),
            "nonce resolver file should include moved marker: {marker}"
        );
    }
    for marker in [
        "fn regression_kolme_live_managed_external_requires_key_reference_env_marker()",
        "fn regression_kolme_live_managed_external_requires_runtime_signer_public_key_marker()",
        "fn regression_kolme_live_managed_external_backend_response_requires_signer_public_key_marker()",
        "fn regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch()",
    ] {
        assert!(
            backend.contains(marker),
            "managed-external backend file should include moved marker: {marker}"
        );
    }
    for marker in [
        "fn regression_kolme_live_managed_external_maps_provider_unavailable_reason_code()",
        "fn regression_kolme_live_managed_external_backend_timeout_maps_reason_code()",
        "fn regression_kolme_live_managed_external_backend_malformed_response_maps_reason_code()",
        "fn regression_kolme_live_managed_external_backend_unavailable_maps_reason_code()",
        "fn regression_kolme_live_managed_external_adapter_retired_not_integrated_marker()",
    ] {
        assert!(
            reason_codes.contains(marker),
            "managed-external reason-code file should include moved marker: {marker}"
        );
    }

    for marker in [
        "struct EnvVarGuard",
        "fn spawn_kolme_live_mock_server(replies: Vec<MockHttpReply>) -> (String, Arc<Mutex<Vec<String>>>)",
        "fn extract_json_string_field(body: &str, field: &str) -> Option<String>",
    ] {
        assert!(
            support.contains(marker),
            "signer support file should include moved marker: {marker}"
        );
    }
}

#[test]
fn spec_c04_signer_root_and_extracted_files_respect_staged_budgets() {
    assert!(
        line_count(ROOT_FILE) <= ROOT_MAX_LINES,
        "signer_tests.rs should stay within staged root cap of {ROOT_MAX_LINES} lines"
    );
    for path in [
        DIRECT_PROFILE_MODULE_FILE,
        DIRECT_SIGNATURE_FILE,
        PROFILE_SELECTION_FILE,
        PREFLIGHT_NONCE_MODULE_FILE,
        PREFLIGHT_POLICY_FILE,
        NONCE_RESOLVER_FILE,
        MANAGED_EXTERNAL_MODULE_FILE,
        MANAGED_EXTERNAL_BACKEND_FILE,
        MANAGED_EXTERNAL_REASON_CODES_FILE,
        SHARED_SUPPORT_FILE,
    ] {
        assert!(
            line_count(path) <= EXTRACTED_MAX_LINES,
            "{path} should stay within extracted file cap of {EXTRACTED_MAX_LINES} lines"
        );
    }
}
