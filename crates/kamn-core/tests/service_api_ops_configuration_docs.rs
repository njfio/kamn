const DOC: &str = include_str!("../../../docs/ops/configuration.md");

#[test]
fn service_api_ops_configuration_contains_async_backpressure_failure_modes() {
    assert!(DOC.contains("## Async API Backpressure Failure Modes (Issue #4315)"));
    assert!(DOC.contains(
        "service_api_backpressure_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("service_api_ingress_concurrency_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_rate_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_sender_rate_limit_exceeded"));
    assert!(DOC.contains("fail-closed response contract"));
    assert!(DOC.contains("Regression: #4315"));
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
fn service_api_ops_configuration_contains_in_memory_provider_rejection_controls() {
    assert!(DOC.contains("## Production-Mode In-Memory Provider Rejection Controls (Issue #4371)"));
    assert!(DOC.contains("runtime_commit_in_memory_provider_reference_detected"));
    assert!(DOC.contains("runtime_commit_policy_check_in_memory_provider_reference_detected"));
    assert!(DOC.contains("InMemoryKolmeRuntimeCommitClient"));
    assert!(DOC.contains("test_run_local_kamn_live_runtime_integration_contract_lane.sh"));
    assert!(DOC.contains("Regression: #4371"));
}

#[test]
fn service_api_ops_configuration_contains_multi_signer_quorum_signature_decision_controls() {
    assert!(DOC
        .contains("## Multi-Signer Profile and Quorum Signature-Decision Controls (Issue #4357)"));
    assert!(DOC.contains(
        "signature_decision_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-signature-decision-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "signature_decision_reason_codes_csv=runtime_signer_profile_missing,runtime_signer_profile_invalid,runtime_signer_previous_profile_missing,runtime_signer_previous_profile_invalid,runtime_signer_failover_profile_unchanged,runtime_signer_profile_changed_without_failover,runtime_signer_rotation_epoch_stale,runtime_signer_attestation_schema_invalid,runtime_signer_attestation_required_approvals_invalid,runtime_signer_attestation_approved_signers_invalid,runtime_signer_attestation_approved_signers_not_unique,runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_quorum_linkage_contract_version_invalid,runtime_signer_quorum_linkage_contract_version_mismatch,runtime_signer_quorum_required_approvals_invalid,runtime_signer_quorum_required_approvals_mismatch,runtime_signer_quorum_approved_signers_count_invalid,runtime_signer_quorum_approved_signers_count_mismatch,runtime_signer_quorum_profile_linked_invalid,runtime_signer_quorum_profile_linked_mismatch,runtime_signer_quorum_satisfied_invalid,runtime_signer_quorum_satisfied_mismatch,runtime_signer_quorum_linked_invalid,runtime_signer_quorum_linkage_drift,runtime_signer_quorum_linkage_violation,runtime_signer_failover_attestation_required_approvals_insufficient,runtime_signer_failover_attestation_previous_profile_not_approved"
    ));
    assert!(DOC.contains("signature_decision_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("runtime_signer_attestation_quorum_shortfall"));
    assert!(DOC.contains("runtime_signer_quorum_linkage_drift"));
    assert!(DOC.contains("Regression: #4357"));
}

#[test]
fn service_api_ops_configuration_contains_retry_envelope_exhaustion_reconnect_bound_governance() {
    assert!(
        DOC.contains("### Retry Envelope Exhaustion and Reconnect Bound Governance (Issue #4296)")
    );
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.runtime.local-retry-diagnostics-reason-taxonomy.v2"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=local_retry_readiness_progress_stalled,local_retry_backoff_jitter_parity_bypass_detected,local_retry_envelope_exhaustion_fail_closed_missing,local_retry_reconnect_attempt_bound_drift,local_retry_reconnect_backoff_bound_drift,ci_local_network_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("retry_envelope_exhaustion_fail_closed_status=verified"));
    assert!(DOC.contains("reconnect_attempt_bound_status=verified"));
    assert!(DOC.contains("reconnect_backoff_bound_status=verified"));
    assert!(DOC.contains("retry_envelope_max_attempts=3"));
    assert!(DOC.contains("retry_envelope_max_backoff_seconds=8"));
    assert!(DOC.contains("local_retry_envelope_exhaustion_fail_closed_missing"));
    assert!(DOC.contains("local_retry_reconnect_attempt_bound_drift"));
    assert!(DOC.contains("local_retry_reconnect_backoff_bound_drift"));
    assert!(DOC.contains("Regression: #4300"));
    assert!(DOC.contains("Regression: #4301"));
}

#[test]
fn service_api_ops_configuration_contains_shutdown_checkpoint_reconciliation_failure_modes() {
    assert!(DOC.contains("Shutdown signal failure matrix"));
    assert!(DOC.contains("full_supervisor_stop_graceful_drain_timeout_contract_mismatch"));
    assert!(DOC.contains(
        "shutdown_checkpoint_reconciliation_reason_taxonomy_version=kamn.runtime.shutdown-checkpoint-reconciliation-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("shutdown_checkpoint_reconciliation_timeout_reason_code_mismatch"));
    assert!(DOC.contains("shutdown_checkpoint_reconciliation_not_signaled_checkpoint_mismatch"));
    assert!(DOC.contains("runtime_shutdown_invariant_violation:<reason_code>"));
    assert!(DOC.contains("Regression: #4332"));
    assert!(DOC.contains("Regression: #4333"));
}
