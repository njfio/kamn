use super::support::assert_doc_contains_all;

const TOUCHED_SHELL_STRICT_MODE_MARKERS: &[&str] = &[
    "test_check_touched_shell_strict_mode.sh",
    "fixtures/ci/touched_shell_strict_mode_exceptions.txt",
    "check_touched_shell_strict_mode.sh --output-json /tmp/touched-shell-strict-mode-report.json",
    "reason_codes=touched_shell_strict_mode_missing_strict_mode",
    "reason_codes=touched_shell_strict_mode_git_base_unavailable",
    "reason_codes=touched_shell_strict_mode_exception_file_invalid",
];

const SIGNER_PROVENANCE_MARKERS: &[&str] = &[
    "### Signer Provenance and Fallback-Prohibition Docs/Config Parity Contract",
    "signer_provenance_fallback_policy_contract_status=active",
    "signer_provenance_fallback_policy_contract_version=v1",
    "signer_provenance_fallback_policy_required_markers_csv=runtime_signer_key_source_policy_reason_codes_csv,managed_signer_backend_response_provenance_missing,managed_signer_backend_response_provenance_malformed,managed_signer_backend_response_provenance_mismatch",
    "cargo test -p kamn-node --test signer_provenance_fallback_policy_contract -- --nocapture",
    "production_signer_key_source_env_local_forbidden",
    "fallback_signer_secret_present_violation",
    "managed_signer_backend_response_provenance_missing",
    "managed_signer_backend_response_provenance_malformed",
    "managed_signer_backend_response_provenance_mismatch",
];

const STARTUP_NEGATIVE_MATRIX_MARKERS: &[&str] = &[
    "## Node Runtime Startup Negative-Matrix Fast Lane",
    "cargo test -p kamn-node main_tests::cli_contract_tests::regression_3599_startup_signer_mode_negative_matrix_corpus -- --exact",
    "cargo test -p kamn-node cli_tests::regression_3598_startup_paths_have_no_panic_control_flow -- --exact",
    "startup_negative_matrix_policy_marker_missing",
    "must fail before network dispatch",
];

#[test]
fn doc_contains_touched_shell_strict_mode_markers() {
    assert_doc_contains_all(
        TOUCHED_SHELL_STRICT_MODE_MARKERS,
        "touched shell strict mode",
    );
}

#[test]
fn doc_contains_signer_provenance_fallback_policy_contract_markers() {
    assert_doc_contains_all(SIGNER_PROVENANCE_MARKERS, "signer provenance fallback");
}

#[test]
fn doc_contains_node_runtime_startup_negative_matrix_fast_lane_contract_markers() {
    assert_doc_contains_all(STARTUP_NEGATIVE_MATRIX_MARKERS, "startup negative matrix");
}
