use super::support::read_repo_file;

const EXTRACTED_LAYOUT: &[(&str, &[&str])] = &[
    (
        "src/main_tests/signer_tests/signer_direct_profile_contract_tests.rs",
        &[
            "mod direct_signature_contract_tests;",
            "mod profile_parity_matrix_contract_tests;",
            "mod profile_selection_contract_tests;",
            "mod runtime_selection_contract_tests;",
            "mod signer_source_policy_contract_tests;",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests.rs",
        &[
            "mod nonce_resolver_contract_tests;",
            "mod preflight_quorum_contract_tests;",
            "mod signer_preflight_policy_contract_tests;",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests.rs",
        &[
            "mod backend_command_contract_tests;",
            "mod backend_provenance_contract_tests;",
            "mod backend_provenance_parity_contract_tests;",
            "mod backend_reason_code_contract_tests;",
            "mod backend_required_marker_contract_tests;",
            "mod backend_response_pubkey_contract_tests;",
            "mod key_source_contract_tests;",
            "mod runtime_pubkey_contract_tests;",
            "mod selection_contract_tests;",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_direct_profile_contract_tests/direct_signature_contract_tests.rs",
        &[
            "fn unit_kolme_live_signer_builds_direct_signed_wire_payload()",
            "fn unit_kolme_live_signer_adapter_signs_and_verifies_runtime_message()",
            "fn integration_kolme_live_signer_vector_probe_contract()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_direct_profile_contract_tests/profile_parity_matrix_contract_tests.rs",
        &["fn functional_signer_migration_profile_key_source_parity_matrix()"],
    ),
    (
        "src/main_tests/signer_tests/signer_direct_profile_contract_tests/profile_selection_contract_tests.rs",
        &[
            "fn unit_kolme_live_signer_profile_defaults_to_primary_key_env()",
            "fn regression_kolme_live_signer_profile_rejects_unsupported_value()",
            "fn integration_kolme_live_signer_profile_secondary_uses_secondary_key_env()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_direct_profile_contract_tests/runtime_selection_contract_tests.rs",
        &[
            "fn integration_runtime_kolme_live_renders_secondary_signer_selection_markers()",
            "fn integration_runtime_kolme_live_renders_managed_external_signer_selection_markers()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_direct_profile_contract_tests/signer_source_policy_contract_tests.rs",
        &[
            "fn regression_kolme_live_signer_adapter_rejects_malformed_signature_hex()",
            "fn regression_signer_private_key_parse_path_requires_zeroize_markers()",
            "fn regression_live_signer_vector_probe_must_not_be_ignored()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests/signer_preflight_policy_contract_tests.rs",
        &[
            "fn integration_kolme_live_signer_preflight_rejects_non_failover_rotation_regression()",
            "fn regression_issue_2279_kolme_live_signer_rejects_fallback_private_key_env_path()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests/preflight_quorum_contract_tests.rs",
        &["fn integration_kolme_live_signer_preflight_quorum_profile_matrix_paths()"],
    ),
    (
        "src/main_tests/signer_tests/signer_preflight_nonce_contract_tests/nonce_resolver_contract_tests.rs",
        &[
            "fn integration_kolme_live_nonce_resolver_fetches_next_nonce()",
            "fn regression_kolme_live_nonce_resolver_rejects_malformed_response()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests/key_source_contract_tests.rs",
        &[
            "fn regression_kolme_live_managed_external_requires_key_reference_env_marker()",
            "fn regression_kolme_live_managed_external_rejects_raw_private_key_env_path()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_command_contract_tests.rs",
        &[
            "fn regression_kolme_live_managed_external_strict_contracts_require_backend_command_marker()",
            "fn regression_kolme_live_managed_external_requires_backend_command_without_required_marker()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_required_marker_contract_tests.rs",
        &[
            "fn regression_kolme_live_managed_external_required_marker_rejects_invalid_boolean()",
            "fn regression_kolme_live_managed_external_required_marker_forces_backend_command()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests/selection_contract_tests.rs",
        &["fn integration_kolme_live_managed_external_adapter_provenance_consumed_by_signer_selection()"],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_response_pubkey_contract_tests.rs",
        &["fn regression_kolme_live_managed_external_backend_response_requires_signer_public_key_marker()"],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests/runtime_pubkey_contract_tests.rs",
        &[
            "fn regression_kolme_live_managed_external_requires_runtime_signer_public_key_marker()",
            "fn regression_kolme_live_managed_external_rejects_invalid_runtime_signer_public_key_marker()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_provenance_contract_tests.rs",
        &["fn regression_kolme_live_managed_external_backend_response_rejects_signer_public_key_mismatch()"],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_provenance_parity_contract_tests.rs",
        &[
            "fn regression_kolme_live_managed_external_backend_response_accepts_case_variant_signer_public_key()",
            "fn regression_kolme_live_managed_external_backend_response_rejects_malformed_signer_public_key()",
        ],
    ),
    (
        "src/main_tests/signer_tests/signer_managed_external_contract_tests/backend_reason_code_contract_tests.rs",
        &[
            "fn regression_kolme_live_managed_external_maps_provider_unavailable_reason_code()",
            "fn regression_kolme_live_managed_external_backend_unavailable_maps_reason_code()",
        ],
    ),
    (
        "src/main_tests/signer_tests/support.rs",
        &["fn managed_external_core_signer_env_guards() -> (EnvVarGuard, EnvVarGuard)"],
    ),
];

#[test]
fn spec_c03_signer_extracted_modules_exist_and_own_coverage() {
    for (path, markers) in EXTRACTED_LAYOUT {
        let source = read_repo_file(path);
        for marker in *markers {
            assert!(
                source.contains(marker),
                "{path} should include moved marker: {marker}"
            );
        }
    }
}
