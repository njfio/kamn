const DOC: &str = include_str!("../../../docs/ops/configuration.md");

#[test]
fn service_api_ops_configuration_contains_signer_secret_zeroization_controls() {
    assert!(
        DOC.contains("## Signer Secret Decode Buffer Zeroization Controls (Issues #4165, #4166)")
    );
    assert!(DOC.contains("signer_secret_source_precedence_zeroization_status=verified"));
    assert!(DOC.contains("signer_private_key_parse_zeroization_status=verified"));
    assert!(DOC.contains("signer_transient_key_material_zeroization_status=verified"));
    assert!(DOC.contains("signer_secret_source_precedence_violation"));
    assert!(DOC.contains("managed_signer_private_key_adapter_unsupported"));
    assert!(DOC.contains(
        "signer::tests::regression_signer_secret_source_precedence_failure_zeroizes_env_secret_buffer"
    ));
    assert!(DOC.contains(
        "signer::tests::unit_build_kolme_live_managed_signing_key_zeroizes_transient_key_material"
    ));
    assert!(DOC.contains("Regression: #4165"));
    assert!(DOC.contains("Regression: #4166"));
}

#[test]
fn service_api_ops_configuration_contains_async_backpressure_failure_modes() {
    assert!(DOC.contains("## Async API Backpressure Failure Modes (Issue #4315)"));
    assert!(DOC.contains(
        "service_api_backpressure_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("service_api_ingress_concurrency_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_rate_limit_exceeded"));
    assert!(DOC.contains("service_api_ingress_sender_rate_limit_exceeded"));
    assert!(DOC.contains("admission_inflight_budget_status=verified"));
    assert!(DOC.contains("admission_queue_budget_status=verified"));
    assert!(DOC.contains("admission_inflight_budget_limit=32"));
    assert!(DOC.contains("admission_queue_budget_limit=1"));
    assert!(DOC.contains(
        "admission_budget_reason_taxonomy_version=kamn.runtime.service-api-admission-budget-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "admission_budget_reason_codes_csv=admission_inflight_budget_mismatch,admission_queue_budget_mismatch"
    ));
    assert!(DOC.contains("admission_decision_taxonomy_status=verified"));
    assert!(DOC.contains("admission_decision_accept_status=verified"));
    assert!(DOC.contains("admission_decision_defer_status=verified"));
    assert!(DOC.contains("admission_decision_reject_status=verified"));
    assert!(DOC.contains(
        "admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject"
    ));
    assert!(DOC.contains("service_api_axum_policy_admission_inflight_budget_limit_mismatch"));
    assert!(DOC.contains("service_api_axum_policy_admission_queue_budget_limit_mismatch"));
    assert!(
        DOC.contains("service_api_axum_policy_admission_decision_reason_taxonomy_version_mismatch")
    );
    assert!(DOC.contains("service_api_axum_policy_admission_decision_reason_codes_csv_mismatch"));
    assert!(DOC.contains("fail-closed response contract"));
    assert!(DOC.contains("Regression: #4315"));
}

#[test]
fn service_api_ops_configuration_contains_quota_policy_fixture_matrix_controls() {
    assert!(
        DOC.contains("## Quota Policy Fixture Matrix and Parser Helper Contracts (Issue #4090)")
    );
    assert!(DOC.contains(
        "quota_policy_fixture_matrix_path=fixtures/runtime/quota_policy_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "quota_policy_fixture_matrix_schema_version=kamn.runtime.quota-policy-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "quota_policy_reason_taxonomy_version=kamn.runtime.quota-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "quota_policy_reason_codes_csv=quota_scope_unknown,quota_window_non_positive,quota_limit_non_positive"
    ));
    assert!(DOC.contains(
        "quota_policy_fixture_columns=case_id|scope|window_seconds|limit|expected_status|expected_reason_code"
    ));
    assert!(DOC.contains("quota_scope_unknown"));
    assert!(DOC.contains("quota_window_non_positive"));
    assert!(DOC.contains("quota_limit_non_positive"));
    assert!(DOC.contains("cargo test -p kamn-core --test quota_policy_fixture_parser_contract"));
    assert!(DOC.contains("Regression: #4090"));
}

#[test]
fn service_api_ops_configuration_contains_fairness_starvation_fixture_controls() {
    assert!(DOC.contains("## Fairness Starvation Fixture and Checker Contracts (Issue #4092)"));
    assert!(DOC.contains(
        "fairness_fixture_matrix_path=fixtures/runtime/starvation_fairness_fixture_matrix.txt"
    ));
    assert!(DOC.contains(
        "fairness_fixture_matrix_schema_version=kamn.runtime.fairness-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "fairness_reason_taxonomy_version=kamn.runtime.fairness-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "fairness_reason_codes_csv=fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap"
    ));
    assert!(DOC.contains(
        "fairness_fixture_columns=case_id|scope|window_seconds|active_weighted_share|max_weighted_share_gap|expected_status|expected_reason_code"
    ));
    assert!(DOC.contains("fairness_scope_unknown"));
    assert!(DOC.contains("fairness_window_non_positive"));
    assert!(DOC.contains("fairness_max_gap_non_positive"));
    assert!(DOC.contains("fairness_weighted_share_exceeds_gap"));
    assert!(DOC.contains("cargo test -p kamn-core --test fairness_policy_checker_contract"));
    assert!(DOC.contains("Regression: #4092"));
}

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
    assert!(DOC.contains(
        "## Signer Material Validation and Fallback Prohibition Contracts (Issues #4167, #4168)"
    ));
    assert!(DOC.contains(
        "signer_config_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-signer-config-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "signer_config_reason_codes_csv=signer_secret_missing,signer_secret_invalid_hex,fallback_signer_secret_present_violation,fallback_signer_secret_checkpoint_reason_mismatch,fallback_signer_secret_remediation_missing"
    ));
    assert!(DOC.contains("signer_config_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("signer_secret_missing"));
    assert!(DOC.contains("signer_secret_invalid_hex"));
    assert!(DOC.contains("fallback_signer_secret_present_violation"));
    assert!(DOC.contains("fallback_signer_secret_checkpoint_reason_mismatch"));
    assert!(DOC.contains("fallback_signer_secret_remediation_missing"));
    assert!(DOC.contains(
        "runtime_signer_key_source_policy_reason_codes_csv=production_signer_key_source_env_local_forbidden,fallback_signer_secret_present_violation"
    ));
    assert!(DOC.contains(
        "managed_signer_provenance_reason_codes_csv=managed_signer_backend_response_provenance_missing,managed_signer_backend_response_provenance_malformed,managed_signer_backend_response_provenance_mismatch"
    ));
    assert!(DOC.contains("managed_signer_backend_response_provenance_missing"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_malformed"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_mismatch"));
    assert!(DOC.contains("signer secret env is required for selected profile"));
    assert!(DOC.contains("fallback signer secret env must not be set"));
    assert!(DOC.contains("remediation: unset KAMN_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_FALLBACK"));
    assert!(DOC.contains("test_run_local_kolme_live_deployment_preflight_lane.sh"));
    assert!(DOC.contains("test_check_local_kolme_live_deployment_preflight_policy.sh"));
    assert!(DOC.contains(
        "cargo test -p kamn-node --test signer_provenance_fallback_policy_contract -- --nocapture"
    ));
    assert!(DOC.contains(
        "check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-deployment-preflight-policy.json"
    ));
    assert!(DOC.contains("Regression: #4167"));
    assert!(DOC.contains("Regression: #4168"));
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
fn service_api_ops_configuration_contains_live_node_drift_marker_mismatch_policy_contracts() {
    assert!(DOC.contains("### Live-Node Drift Marker Mismatch Policy Contracts (Issue #4281)"));
    assert!(DOC.contains("failover_promotion_gate_status=verified"));
    assert!(DOC.contains("live_node_drift_parity_status=verified"));
    assert!(DOC.contains("ci_local_promotion_budget_boundary_status=verified"));
    assert!(DOC.contains(
        "failover_readiness_reason_taxonomy_version=kamn.runtime.failover-readiness-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "failover_readiness_reason_codes_csv=failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("failover_sync_drift_policy_status=verified"));
    assert!(DOC.contains(
        "bash scripts/runtime/failover_sync_drill_preflight_contract_lane_contract.sh check-policy"
    ));
    assert!(DOC.contains("live_node_drift_marker_parity_mismatch"));
    assert!(DOC.contains("failover_readiness_progress_stalled"));
    assert!(DOC.contains("ci_local_promotion_budget_boundary_exceeded"));
    assert!(DOC.contains("failover_sync_drift_policy_required_field_missing:<field>"));
    assert!(DOC.contains("failover_sync_drift_policy_reason_taxonomy_version_mismatch"));
    assert!(DOC.contains("failover_sync_drift_policy_reason_codes_csv_mismatch"));
    assert!(DOC.contains("Regression: #4285"));
    assert!(DOC.contains("Regression: #4286"));
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

#[test]
fn service_api_ops_configuration_contains_full_stack_harness_marker_mismatch_controls() {
    assert!(DOC.contains(
        "## Full-Stack Harness Marker Completeness and Parity Mismatch Controls (Issue #4195)"
    ));
    assert!(DOC.contains("full_io_harness_marker_completeness_status=verified"));
    assert!(DOC.contains("full_io_harness_marker_parity_status=verified"));
    assert!(DOC.contains(
        "full_io_harness_policy_reason_taxonomy_version=kamn.runtime.full-io-scenario-matrix-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "full_io_harness_policy_reason_codes_csv=full_io_scenario_matrix_policy_process_harness_mismatch,full_io_scenario_matrix_policy_api_route_matrix_mismatch,full_io_scenario_matrix_policy_auth_failure_matrix_mismatch,full_io_scenario_matrix_policy_websocket_matrix_mismatch,full_io_scenario_matrix_policy_multinode_propagation_mismatch,full_io_scenario_matrix_policy_dry_run_command_count_mismatch,full_io_scenario_matrix_policy_dry_run_command_status_mismatch"
    ));
    assert!(DOC.contains("full_io_scenario_matrix_policy_process_harness_mismatch"));
    assert!(DOC.contains("full_io_scenario_matrix_policy_dry_run_command_count_mismatch"));
    assert!(DOC.contains("full_io_scenario_matrix_policy_dry_run_command_status_mismatch"));
    assert!(DOC.contains("Regression: #4195"));
}

#[test]
fn service_api_ops_configuration_contains_upgrade_compatibility_marker_matrix_controls() {
    assert!(DOC.contains("## Upgrade Compatibility Marker Matrix Controls (Issue #4181)"));
    assert!(DOC.contains(
        "check_upgrade_compatibility_marker_matrix_policy.py --version-report-file /tmp/kolme-version-report.json --fork-policy-report-file /tmp/kolme-fork-compatibility-policy-report.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-upgrade-compatibility-marker-matrix-policy-report.json"
    ));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=version_report_missing,fork_policy_report_missing,version_report_schema_mismatch,version_report_reason_taxonomy_mismatch,version_report_reason_codes_csv_mismatch,version_report_rehearsal_bypass_guard_status_mismatch,version_report_rehearsal_output_normalization_status_mismatch,fork_policy_report_schema_mismatch,fork_policy_report_reason_taxonomy_mismatch,fork_policy_report_reason_codes_csv_mismatch,fork_policy_report_rehearsal_bypass_guard_status_mismatch,fork_policy_report_rehearsal_output_normalization_status_mismatch,expected_final_decision_mismatch,ci_fast_gate_failed"
    ));
    assert!(DOC.contains("version_report_schema_mismatch"));
    assert!(DOC.contains("fork_policy_report_reason_codes_csv_mismatch"));
    assert!(DOC.contains("fork_policy_report_rehearsal_bypass_guard_status_mismatch"));
    assert!(DOC.contains("Regression: #4180"));
    assert!(DOC.contains("Regression: #4181"));
}

#[test]
fn service_api_ops_configuration_contains_partition_healing_mismatch_mapping_controls() {
    assert!(DOC.contains(
        "### Block Reconciliation Partition-Healing Mismatch Mapping Contracts (Issues #4251, #4255, #4256)"
    ));
    assert!(DOC.contains("partition_healing_mismatch_reason_mapping_status=verified"));
    assert!(DOC.contains(
        "partition_healing_mismatch_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-healing-mismatch-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "partition_healing_mismatch_reason_codes_csv=block_reconciliation_partition_rejoin_policy_required_field_missing,block_reconciliation_partition_rejoin_policy_marker_mismatch,block_reconciliation_partition_rejoin_policy_transport_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_taxonomy_mismatch,block_reconciliation_partition_rejoin_policy_recovery_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid,block_reconciliation_partition_rejoin_policy_lane_mode_contract_mismatch,block_reconciliation_partition_rejoin_policy_ci_fast_gate_failed,block_reconciliation_partition_rejoin_policy_expected_decision_mismatch,block_reconciliation_partition_rejoin_policy_violation"
    ));
    assert!(DOC.contains("partition_healing_mismatch_reason_code=none|<reason>"));
    assert!(
        DOC.contains("block_reconciliation_partition_rejoin_policy_required_field_missing:<field>")
    );
    assert!(DOC.contains(
        "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid"
    ));
    assert!(DOC.contains("Regression: #4255"));
    assert!(DOC.contains("Regression: #4256"));
}
