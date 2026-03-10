use super::*;

#[test]
fn service_api_ops_configuration_contains_protocol_mismatch_reason_mapping_controls() {
    assert!(DOC.contains(
        "## API Protocol Compliance Mismatch Reason Mapping (Issues #4266, #4270, #4271)"
    ));
    assert!(DOC.contains("service_api_axum_protocol_mismatch_reason_mapping_status=verified"));
    assert!(DOC.contains(
        "service_api_axum_protocol_mismatch_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "service_api_axum_protocol_mismatch_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation"
    ));
    assert!(DOC.contains("service_api_axum_protocol_mismatch_reason_code=none|<reason>"));
    assert!(DOC.contains("service_api_axum_policy_protocol_taxonomy_mismatch"));
    assert!(DOC.contains("service_api_axum_policy_limit_contract_mismatch"));
    assert!(DOC.contains("Regression: #4270"));
    assert!(DOC.contains("Regression: #4271"));
}

#[test]
fn service_api_ops_configuration_contains_audit_integrity_tamper_controls() {
    assert!(DOC.contains("## Audit Integrity Go/No-Go Policy Controls (Issue #4465)"));
    assert!(DOC.contains(
        "audit_integrity_reason_taxonomy_version=kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("gonogo_audit_integrity_reason_taxonomy_version_mismatch"));
    assert!(DOC.contains("gonogo_audit_integrity_reason_codes_csv_mismatch"));
    assert!(DOC.contains("audit integrity gate convergence mismatch"));
    assert!(DOC.contains("Regression: #4465"));
}

#[test]
fn service_api_ops_configuration_contains_journal_append_checkpoint_integrity_controls() {
    assert!(DOC
        .contains("## Journal Append/Checkpoint Integrity Controls (Issues #4236, #4240, #4241)"));
    assert!(DOC.contains("append_checkpoint_integrity_status=verified"));
    assert!(DOC.contains(
        "append_checkpoint_reason_taxonomy_version=kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "append_checkpoint_reason_codes_csv=wal_append_marker_missing,wal_checkpoint_marker_missing,append_checkpoint_marker_parity_mismatch"
    ));
    assert!(DOC.contains("sqlite_crash_recovery_policy_wal_append_status_mismatch"));
    assert!(DOC.contains("sqlite_crash_recovery_policy_wal_checkpoint_status_mismatch"));
    assert!(DOC.contains("sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch"));
    assert!(DOC.contains("Regression: #4240"));
    assert!(DOC.contains("Regression: #4241"));
}

#[test]
fn service_api_ops_configuration_contains_in_memory_provider_rejection_controls() {
    assert!(DOC.contains("## Production-Mode In-Memory Provider Rejection Controls (Issue #4371)"));
    assert!(DOC.contains("runtime_commit_in_memory_provider_reference_detected"));
    assert!(DOC.contains("runtime_commit_policy_check_in_memory_provider_reference_detected"));
    assert!(DOC.contains("InMemoryKolmeRuntimeCommitClient"));
    assert!(DOC.contains("test_run_local_kamn_live_runtime_integration_contract_lane.sh"));
    assert!(DOC.contains("Regression: #4371"));
}

#[test]
fn service_api_ops_configuration_contains_signer_material_validation_and_fallback_prohibition_contracts(
) {
    assert_doc_contains_all(&["## Signer Material Validation and Fallback Prohibition Contracts (Issues #4167, #4168)", "signer_config_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-signer-config-reason-taxonomy.v1", "signer_config_reason_codes_csv=signer_secret_missing,signer_secret_invalid_hex,fallback_signer_secret_present_violation,fallback_signer_secret_checkpoint_reason_mismatch,fallback_signer_secret_remediation_missing", "signer_config_reason_codes_value=none|<csv>", "signer_secret_missing", "signer_secret_invalid_hex", "fallback_signer_secret_present_violation", "fallback_signer_secret_checkpoint_reason_mismatch", "fallback_signer_secret_remediation_missing", "runtime_signer_key_source_policy_reason_codes_csv=production_signer_key_source_env_local_forbidden,fallback_signer_secret_present_violation", "managed_signer_provenance_reason_codes_csv=managed_signer_backend_response_provenance_missing,managed_signer_backend_response_provenance_malformed,managed_signer_backend_response_provenance_mismatch", "managed_signer_backend_response_provenance_missing", "managed_signer_backend_response_provenance_malformed", "managed_signer_backend_response_provenance_mismatch", "signer secret env is required for selected profile", "fallback signer secret env must not be set", "remediation: unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK", "test_run_local_kolme_live_deployment_preflight_lane.sh", "test_check_local_kolme_live_deployment_preflight_policy.sh", "cargo test -p kamn-node --test signer_provenance_fallback_policy_contract -- --nocapture", "check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-deployment-preflight-policy.json", "Regression: #4167", "Regression: #4168"]);
}
#[test]
fn service_api_ops_configuration_contains_managed_key_source_adapter_provenance_mapping() {
    assert!(DOC.contains("managed_key_source_adapter_provenance_status=verified"));
    assert!(DOC.contains(
        "managed_key_source_adapter_provenance_fields_csv=profile,key_source,key_reference_env,signer_public_key_hex"
    ));
    assert!(DOC.contains(
        "managed_key_source_adapter_provenance_reason_codes_csv=managed_signer_provenance_marker_profile_mismatch,managed_signer_provenance_marker_key_source_mismatch,managed_signer_provenance_marker_key_reference_env_mismatch,managed_signer_provenance_marker_public_key_missing"
    ));
    assert!(DOC.contains("managed_signer_provenance_marker_profile_mismatch"));
    assert!(DOC.contains("managed_signer_provenance_marker_key_source_mismatch"));
    assert!(DOC.contains("managed_signer_provenance_marker_key_reference_env_mismatch"));
    assert!(DOC.contains("managed_signer_provenance_marker_public_key_missing"));
    assert!(DOC.contains(
        "cargo test -p kamn-node signer::managed_backend::tests::unit_managed_key_source_adapter_emits_deterministic_provenance_marker -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node signer::tests::regression_managed_key_source_provenance_marker_profile_mismatch_fails_closed -- --exact"
    ));
    assert!(DOC.contains("Regression: #3955"));
}

#[test]
fn service_api_ops_configuration_contains_multi_signer_quorum_signature_decision_controls() {
    assert!(DOC
        .contains("## Multi-Signer Profile and Quorum Signature-Decision Controls (Issue #4357)"));
    assert!(DOC.contains(
        "signature_decision_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-signature-decision-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "signature_decision_reason_codes_csv=runtime_signer_profile_missing,runtime_signer_profile_invalid,runtime_signer_previous_profile_missing,runtime_signer_previous_profile_invalid,runtime_signer_failover_profile_unchanged,runtime_signer_profile_changed_without_failover,runtime_signer_rotation_epoch_stale,runtime_signer_rotation_epoch_regressed,runtime_signer_attestation_schema_invalid,runtime_signer_attestation_required_approvals_invalid,runtime_signer_attestation_approved_signers_invalid,runtime_signer_attestation_approved_signers_not_unique,runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_quorum_linkage_contract_version_invalid,runtime_signer_quorum_linkage_contract_version_mismatch,runtime_signer_quorum_required_approvals_invalid,runtime_signer_quorum_required_approvals_mismatch,runtime_signer_quorum_approved_signers_count_invalid,runtime_signer_quorum_approved_signers_count_mismatch,runtime_signer_quorum_profile_linked_invalid,runtime_signer_quorum_profile_linked_mismatch,runtime_signer_quorum_satisfied_invalid,runtime_signer_quorum_satisfied_mismatch,runtime_signer_quorum_linked_invalid,runtime_signer_quorum_linkage_drift,runtime_signer_quorum_linkage_violation,runtime_signer_failover_attestation_required_approvals_insufficient,runtime_signer_failover_attestation_previous_profile_not_approved"
    ));
    assert!(DOC.contains("signature_decision_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("runtime_signer_attestation_quorum_shortfall"));
    assert!(DOC.contains("runtime_signer_quorum_linkage_drift"));
    assert!(DOC.contains("Regression: #4357"));
}

#[test]
fn service_api_ops_configuration_contains_signer_quorum_profile_matrix_controls() {
    assert!(DOC.contains("signer_quorum_profile_matrix_fixture_status=verified"));
    assert!(DOC.contains(
        "signer_quorum_profile_matrix_case_labels_csv=linked_non_failover_primary,profile_not_approved_non_failover,quorum_shortfall_non_failover,failover_previous_profile_not_approved,linked_failover_dual_approved"
    ));
    assert!(DOC.contains(
        "signer_quorum_profile_matrix_fail_closed_reason_codes_csv=runtime_signer_quorum_linkage_violation,runtime_signer_attestation_quorum_shortfall,runtime_signer_failover_attestation_previous_profile_not_approved"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node signer::signer_policy::tests::unit_signer_quorum_decision_path_matrix -- --exact --nocapture"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node main_tests::signer_tests::integration_kolme_live_signer_preflight_quorum_profile_matrix_paths -- --exact --nocapture"
    ));
    assert!(DOC.contains("Regression: #3957"));
}
