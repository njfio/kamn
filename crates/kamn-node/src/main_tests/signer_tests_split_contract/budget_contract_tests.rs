use super::support::line_count;

const ROOT_MAX_LINES: usize = 200;
const EXTRACTED_MAX_LINES: usize = 200;
const EXTRACTED_FILES: &[&str] = &[
    "src/main_tests/signer_tests/signer_direct_profile_contract_tests.rs",
    "src/main_tests/signer_tests/signer_direct_profile_contract_tests/direct_signature_contract_tests.rs",
    "src/main_tests/signer_tests/signer_direct_profile_contract_tests/profile_parity_matrix_contract_tests.rs",
    "src/main_tests/signer_tests/signer_direct_profile_contract_tests/profile_selection_contract_tests.rs",
    "src/main_tests/signer_tests/signer_direct_profile_contract_tests/runtime_selection_contract_tests.rs",
    "src/main_tests/signer_tests/signer_direct_profile_contract_tests/signer_source_policy_contract_tests.rs",
    "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests.rs",
    "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests/signer_preflight_policy_contract_tests.rs",
    "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests/preflight_quorum_contract_tests.rs",
    "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests/nonce_resolver_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/key_source_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_command_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_required_marker_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/selection_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_response_pubkey_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/runtime_pubkey_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_provenance_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_provenance_parity_contract_tests.rs",
    "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_reason_code_contract_tests.rs",
    "src/main_tests/signer_tests/support.rs",
];

#[test]
fn spec_c04_signer_root_and_extracted_files_respect_staged_budgets() {
    assert!(
        line_count("src/main_tests/signer_tests.rs") <= ROOT_MAX_LINES,
        "signer_tests.rs should stay within staged root cap of {ROOT_MAX_LINES} lines"
    );
    for path in EXTRACTED_FILES {
        assert!(
            line_count(path) <= EXTRACTED_MAX_LINES,
            "{path} should stay within extracted file cap of {EXTRACTED_MAX_LINES} lines"
        );
    }
}
