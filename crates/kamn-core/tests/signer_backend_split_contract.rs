use std::fs;

const SIGNER_EMULATOR_ROOT_MARKERS: [&str; 10] = [
    "#[path = \"signer_backend/signer_emulator_cases.rs\"]",
    "mod signer_emulator_cases;",
    "signer_emulator_cases::run_performance_signer_emulator_contract_lane_stays_within_budget();",
    "signer_emulator_cases::run_regression_signer_emulator_budget_comparator_allows_exact_boundary();",
    "signer_emulator_cases::run_regression_signer_emulator_budget_parser_rejects_invalid_override();",
    "signer_emulator_cases::run_regression_signer_emulator_budget_parser_uses_local_default_when_unset();",
    "signer_emulator_cases::run_regression_signer_emulator_budget_parser_uses_ci_default_when_ci_set();",
    "signer_emulator_cases::run_performance_signer_emulator_bulk_signing_deep_lane();",
    "fn performance_signer_emulator_contract_lane_stays_within_budget()",
    "fn performance_signer_emulator_bulk_signing_deep_lane()",
];

const SIGNER_EMULATOR_CASES_MARKERS: [&str; 6] = [
    "pub(super) fn run_performance_signer_emulator_contract_lane_stays_within_budget(",
    "pub(super) fn run_regression_signer_emulator_budget_comparator_allows_exact_boundary(",
    "pub(super) fn run_regression_signer_emulator_budget_parser_rejects_invalid_override(",
    "pub(super) fn run_regression_signer_emulator_budget_parser_uses_local_default_when_unset(",
    "pub(super) fn run_regression_signer_emulator_budget_parser_uses_ci_default_when_ci_set(",
    "pub(super) fn run_performance_signer_emulator_bulk_signing_deep_lane(",
];

const SIGNER_PROVIDER_ROOT_MARKERS: [&str; 13] = [
    "#[path = \"signer_backend/signer_provider_cases.rs\"]",
    "mod signer_provider_cases;",
    "signer_provider_cases::run_functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider();",
    "signer_provider_cases::run_functional_admin_role_key_signs_when_sender_role_matches();",
    "signer_provider_cases::run_regression_role_mismatch_signing_request_is_rejected();",
    "signer_provider_cases::run_regression_admin_key_does_not_fallback_when_secure_provider_unavailable();",
    "signer_provider_cases::run_functional_privileged_roles_deny_fallback_when_provider_unavailable();",
    "signer_provider_cases::run_regression_unknown_secure_provider_is_rejected_without_fallback();",
    "signer_provider_cases::run_regression_provider_handshake_policy_block_rejects_without_fallback();",
    "signer_provider_cases::run_regression_provider_client_backend_mismatch_is_rejected_without_fallback();",
    "signer_provider_cases::run_regression_secure_provider_backend_mismatch_is_rejected();",
    "fn functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider()",
    "fn regression_secure_provider_backend_mismatch_is_rejected()",
];

const SIGNER_PROVIDER_CASES_MARKERS: [&str; 9] = [
    "pub(super) fn run_functional_provider_handshake_matrix_routes_operator_fallback_for_unavailable_provider(",
    "pub(super) fn run_functional_admin_role_key_signs_when_sender_role_matches(",
    "pub(super) fn run_regression_role_mismatch_signing_request_is_rejected(",
    "pub(super) fn run_regression_admin_key_does_not_fallback_when_secure_provider_unavailable(",
    "pub(super) fn run_functional_privileged_roles_deny_fallback_when_provider_unavailable(",
    "pub(super) fn run_regression_unknown_secure_provider_is_rejected_without_fallback(",
    "pub(super) fn run_regression_provider_handshake_policy_block_rejects_without_fallback(",
    "pub(super) fn run_regression_provider_client_backend_mismatch_is_rejected_without_fallback(",
    "pub(super) fn run_regression_secure_provider_backend_mismatch_is_rejected(",
];

const SIGNER_SIGNATURE_ROOT_MARKERS: [&str; 9] = [
    "#[path = \"signer_backend/signer_signature_cases.rs\"]",
    "mod signer_signature_cases;",
    "signer_signature_cases::run_integration_signer_backend_accepts_baseline_v1_only_with_explicit_compatibility_switch();",
    "signer_signature_cases::run_regression_signer_backend_rejects_baseline_v1_signature_by_default();",
    "signer_signature_cases::run_regression_local_backend_rejects_tampered_signature();",
    "signer_signature_cases::run_regression_local_backend_rejects_signature_when_verifier_uses_wrong_key();",
    "signer_signature_cases::run_regression_local_backend_rejects_baseline_v1_signature_without_compat_switch();",
    "fn integration_signer_backend_accepts_baseline_v1_only_with_explicit_compatibility_switch()",
    "fn regression_local_backend_rejects_baseline_v1_signature_without_compat_switch()",
];

const SIGNER_SIGNATURE_CASES_MARKERS: [&str; 5] = [
    "pub(super) fn run_integration_signer_backend_accepts_baseline_v1_only_with_explicit_compatibility_switch(",
    "pub(super) fn run_regression_signer_backend_rejects_baseline_v1_signature_by_default(",
    "pub(super) fn run_regression_local_backend_rejects_tampered_signature(",
    "pub(super) fn run_regression_local_backend_rejects_signature_when_verifier_uses_wrong_key(",
    "pub(super) fn run_regression_local_backend_rejects_baseline_v1_signature_without_compat_switch(",
];

const SIGNER_REQUEST_ROOT_MARKERS: [&str; 7] = [
    "#[path = \"signer_backend/signer_request_cases.rs\"]",
    "mod signer_request_cases;",
    "signer_request_cases::run_for_transaction_rejects_empty_transaction_id();",
    "signer_request_cases::run_regression_signing_request_matches_canonical_signature_profile();",
    "signer_request_cases::run_regression_signatures_include_profile_identifier_segment();",
    "fn for_transaction_rejects_empty_transaction_id()",
    "fn regression_signatures_include_profile_identifier_segment()",
];

const SIGNER_REQUEST_CASES_MARKERS: [&str; 3] = [
    "pub(super) fn run_for_transaction_rejects_empty_transaction_id(",
    "pub(super) fn run_regression_signing_request_matches_canonical_signature_profile(",
    "pub(super) fn run_regression_signatures_include_profile_identifier_segment(",
];

const SIGNER_CORE_ROOT_MARKERS: [&str; 12] = [
    "#[path = \"signer_backend/signer_core_cases.rs\"]",
    "mod signer_core_cases;",
    "signer_core_cases::run_functional_secure_backend_signs_and_verifies_when_available();",
    "signer_core_cases::run_functional_aws_kms_provider_routes_to_production_adapter_backend();",
    "signer_core_cases::run_functional_router_uses_custom_provider_client_mapping_for_secure_provider();",
    "signer_core_cases::run_functional_secure_unavailable_falls_back_to_local_backend();",
    "signer_core_cases::run_regression_local_backend_signing_requires_explicit_key_material();",
    "signer_core_cases::run_integration_router_signed_transaction_passes_transaction_guards();",
    "signer_core_cases::run_integration_aws_kms_signed_transaction_passes_transaction_guards();",
    "signer_core_cases::run_regression_unsupported_secure_key_reference_does_not_fallback();",
    "fn functional_secure_backend_signs_and_verifies_when_available()",
    "fn regression_unsupported_secure_key_reference_does_not_fallback()",
];

const SIGNER_CORE_CASES_MARKERS: [&str; 8] = [
    "pub(super) fn run_functional_secure_backend_signs_and_verifies_when_available(",
    "pub(super) fn run_functional_aws_kms_provider_routes_to_production_adapter_backend(",
    "pub(super) fn run_functional_router_uses_custom_provider_client_mapping_for_secure_provider(",
    "pub(super) fn run_functional_secure_unavailable_falls_back_to_local_backend(",
    "pub(super) fn run_regression_local_backend_signing_requires_explicit_key_material(",
    "pub(super) fn run_integration_router_signed_transaction_passes_transaction_guards(",
    "pub(super) fn run_integration_aws_kms_signed_transaction_passes_transaction_guards(",
    "pub(super) fn run_regression_unsupported_secure_key_reference_does_not_fallback(",
];

fn read_repo_file(path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let full_path = format!("{root}/{path}");
    fs::read_to_string(&full_path).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn spec_c01_signer_emulator_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/signer_backend.rs");
    let cases = read_repo_file("tests/signer_backend/signer_emulator_cases.rs");

    for marker in SIGNER_EMULATOR_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root signer-backend contract should contain signer-emulator delegation marker: {marker}"
        );
    }

    for marker in SIGNER_EMULATOR_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "signer-emulator cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c02_signer_provider_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/signer_backend.rs");
    let cases = read_repo_file("tests/signer_backend/signer_provider_cases.rs");

    for marker in SIGNER_PROVIDER_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root signer-backend contract should contain signer-provider delegation marker: {marker}"
        );
    }

    for marker in SIGNER_PROVIDER_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "signer-provider cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c03_signer_signature_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/signer_backend.rs");
    let cases = read_repo_file("tests/signer_backend/signer_signature_cases.rs");

    for marker in SIGNER_SIGNATURE_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root signer-backend contract should contain signer-signature delegation marker: {marker}"
        );
    }

    for marker in SIGNER_SIGNATURE_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "signer-signature cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c04_signer_request_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/signer_backend.rs");
    let cases = read_repo_file("tests/signer_backend/signer_request_cases.rs");

    for marker in SIGNER_REQUEST_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root signer-backend contract should contain signer-request delegation marker: {marker}"
        );
    }

    for marker in SIGNER_REQUEST_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "signer-request cases module should define marker: {marker}"
        );
    }
}

#[test]
fn spec_c05_signer_core_tests_delegate_to_cases_module() {
    let root = read_repo_file("tests/signer_backend.rs");
    let cases = read_repo_file("tests/signer_backend/signer_core_cases.rs");

    for marker in SIGNER_CORE_ROOT_MARKERS {
        assert!(
            root.contains(marker),
            "root signer-backend contract should contain signer-core delegation marker: {marker}"
        );
    }

    for marker in SIGNER_CORE_CASES_MARKERS {
        assert!(
            cases.contains(marker),
            "signer-core cases module should define marker: {marker}"
        );
    }
}
