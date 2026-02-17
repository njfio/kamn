const CHECKLIST: &str = include_str!("../../../docs/foundation/release-gonogo-checklist.md");

#[test]
fn checklist_contains_preflight_gates() {
    assert!(CHECKLIST.contains("## Preflight Gates"));
    assert!(CHECKLIST.contains("Migration plan reviewed and signed"));
    assert!(CHECKLIST.contains("Compatibility matrix validated"));
    assert!(CHECKLIST.contains("CI fast gate and deferred deep lane both green"));
    assert!(CHECKLIST.contains("Rollback runbook version pinned"));
}

#[test]
fn checklist_contains_production_mode_live_provider_enforcement_gate() {
    assert!(CHECKLIST.contains("## Production-Mode Live Provider Enforcement Gate (Issue #4371)"));
    assert!(CHECKLIST.contains("test_run_local_kamn_live_runtime_integration_contract_lane.sh"));
    assert!(CHECKLIST.contains("test_run_local_kamn_live_runtime_integration_real_node_profile.sh"));
    assert!(CHECKLIST.contains("runtime_commit_in_memory_provider_reference_detected"));
    assert!(CHECKLIST.contains("runtime_commit_policy_check_in_memory_provider_reference_detected"));
    assert!(CHECKLIST.contains("InMemoryKolmeRuntimeCommitClient"));
    assert!(CHECKLIST.contains("Regression: #4371"));
}

#[test]
fn checklist_contains_dry_run_workflow() {
    assert!(CHECKLIST.contains("## Deterministic Dry-Run Workflow"));
    assert!(CHECKLIST.contains("1. Create release candidate tag"));
    assert!(CHECKLIST.contains("2. Rehearse migration on staging snapshot"));
    assert!(CHECKLIST.contains("3. Execute bounded smoke and invariant suites"));
    assert!(CHECKLIST.contains("4. Capture and sign dry-run evidence bundle"));
}

#[test]
fn checklist_contains_go_no_go_evidence_template() {
    assert!(CHECKLIST.contains("## Go/No-Go Evidence Template"));
    assert!(CHECKLIST.contains("Release candidate:"));
    assert!(CHECKLIST.contains("Schema target version:"));
    assert!(CHECKLIST.contains("Rollback trigger status:"));
    assert!(CHECKLIST.contains("Final decision: GO | NO-GO"));
}

#[test]
fn checklist_contains_message_anchoring_mismatch_tamper_gate() {
    assert!(CHECKLIST.contains("## Message Anchoring Mismatch/Tamper Gate (Issue #4419)"));
    assert!(CHECKLIST.contains("run_message_proof_anchoring_contract_lane.sh"));
    assert!(CHECKLIST.contains("validate_message_proof_anchoring_live.sh"));
    assert!(CHECKLIST.contains(
        "anchoring_gate_reason_taxonomy_version=kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "anchoring_gate_reason_codes_csv=message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required"
    ));
    assert!(CHECKLIST.contains("ci_smoke_local_heavy_boundary_status=verified"));
    assert!(CHECKLIST.contains("local_heavy_lane_execution_mode=opt_in"));
    assert!(CHECKLIST.contains("Regression: #4419"));
}

#[test]
fn checklist_contains_service_api_protocol_session_reason_mapping_gate() {
    assert!(CHECKLIST.contains("## Service API Protocol/Session Reason Mapping Gate (Issue #4318)"));
    assert!(CHECKLIST.contains(
        "service_api_protocol_session_reason_taxonomy_version=kamn.runtime.service-api.protocol-session-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "service_api_protocol_session_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing"
    ));
    assert!(CHECKLIST.contains("service_api_ws_upgrade_header_missing"));
    assert!(CHECKLIST.contains("service_api_ws_version_header_invalid"));
    assert!(CHECKLIST.contains("service_api_payload_json_syntax_invalid"));
    assert!(CHECKLIST.contains("service_api_auth_replay_nonce_detected"));
    assert!(CHECKLIST.contains("service_api_protocol_session_docs_marker_missing"));
    assert!(CHECKLIST.contains("Regression: #4318"));
}

#[test]
fn checklist_contains_panic_replacement_reason_taxonomy_and_runtime_evidence_gate() {
    assert!(CHECKLIST
        .contains("## Panic-Replacement Reason Taxonomy and Runtime Evidence Gate (Issue #4455)"));
    assert!(CHECKLIST.contains(
        "scripts/ci/check_no_production_expect.sh --root crates/kamn-node/src --output-json /tmp/no-production-expect-report.json"
    ));
    assert!(CHECKLIST.contains(
        "panic_replacement_reason_taxonomy_version=kamn.ci.production-panic-replacement-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "panic_replacement_reason_codes_csv=scan_root_not_found,production_expect_reachable,production_panic_macro_reachable,production_unreachable_macro_reachable,production_unsafe_env_fallback_default"
    ));
    assert!(CHECKLIST.contains("panic_replacement_reason_codes_value=none|<csv>"));
    assert!(CHECKLIST.contains(
        "panic_replacement_reason_class=stable|panic_reachability|unsafe_fallback|mixed|configuration"
    ));
    assert!(CHECKLIST.contains("runtime_panic_replacement_evidence_status=verified|violation"));
    assert!(CHECKLIST.contains("runtime_panic_replacement_evidence_violation_count=<n>"));
    assert!(CHECKLIST.contains("runtime_panic_replacement_evidence_files_csv=none|<csv>"));
    assert!(CHECKLIST.contains(
        "runtime_panic_replacement_evidence_outputs_csv=runtime_panic_replacement_evidence_status,runtime_panic_replacement_evidence_violation_count,runtime_panic_replacement_evidence_files_csv"
    ));
    assert!(CHECKLIST.contains("Regression: #4455"));
}

#[test]
fn checklist_contains_dependency_license_metadata_docs_mismatch_gate() {
    assert!(CHECKLIST.contains("## Dependency-License Metadata/Docs Mismatch Gate (Issue #4456)"));
    assert!(CHECKLIST.contains("scripts/ci/test_check_workspace_license_policy.sh"));
    assert!(CHECKLIST.contains("scripts/ci/test_check_kamn_core_live_https_dependency_posture.sh"));
    assert!(CHECKLIST.contains("license_mismatch"));
    assert!(CHECKLIST.contains("license_missing"));
    assert!(CHECKLIST.contains("manifest_invalid_toml"));
    assert!(CHECKLIST.contains("package_section_missing"));
    assert!(CHECKLIST.contains("readme_webpki_roots_reference_missing"));
    assert!(CHECKLIST.contains("readme_no_default_features_marker_missing"));
    assert!(CHECKLIST.contains("ci_strategy_no_default_features_check_missing"));
    assert!(CHECKLIST.contains("Regression: #4456"));
}

#[test]
fn checklist_contains_machine_readable_bundle_contract() {
    assert!(CHECKLIST.contains("## Machine-Readable Evidence Bundle Contract"));
    assert!(CHECKLIST.contains("gonogo_evidence_contract.py"));
    assert!(CHECKLIST.contains("generate_gonogo_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_gonogo_evidence_policy.sh"));
    assert!(CHECKLIST.contains("run_gonogo_evidence_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_gonogo_evidence_deep_lane.sh"));
}

#[test]
fn checklist_contains_tls_evidence_completeness_freshness_gate() {
    assert!(
        CHECKLIST.contains("## TLS Evidence Completeness/Freshness Convergence Gate (Issue #4477)")
    );
    assert!(CHECKLIST.contains(
        "--tls-evidence-report-file /tmp/kamn-core-live-https-dependency-posture-report.json"
    ));
    assert!(CHECKLIST.contains("--tls-evidence-max-age-seconds 1800"));
    assert!(CHECKLIST.contains(
        "tls_evidence_reason_taxonomy_version=kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "tls_evidence_reason_codes_csv=gonogo_tls_evidence_file_missing,gonogo_tls_evidence_invalid_json,gonogo_tls_evidence_schema_mismatch,gonogo_tls_evidence_status_not_pass,gonogo_tls_evidence_reason_taxonomy_version_mismatch,gonogo_tls_evidence_freshness_window_exceeded"
    ));
    assert!(CHECKLIST.contains("tls_evidence_gate_final_decision=GO|NO-GO"));
    assert!(CHECKLIST.contains("Regression: #4477"));
}

#[test]
fn checklist_contains_audit_integrity_convergence_gate() {
    assert!(CHECKLIST.contains("## Audit-Trail Integrity/Tamper Convergence Gate (Issue #4466)"));
    assert!(CHECKLIST.contains(
        "--audit-integrity-report-file /tmp/sqlite-crash-recovery-live-policy-report.json"
    ));
    assert!(CHECKLIST.contains("--audit-integrity-max-age-seconds 1800"));
    assert!(CHECKLIST.contains(
        "audit_integrity_reason_taxonomy_version=kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "audit_integrity_reason_codes_csv=gonogo_audit_integrity_file_missing,gonogo_audit_integrity_invalid_json,gonogo_audit_integrity_schema_mismatch,gonogo_audit_integrity_status_not_ok,gonogo_audit_integrity_final_decision_not_go,gonogo_audit_integrity_policy_status_not_verified,gonogo_audit_integrity_reason_taxonomy_version_mismatch,gonogo_audit_integrity_reason_codes_csv_mismatch,gonogo_audit_integrity_freshness_window_exceeded"
    ));
    assert!(CHECKLIST.contains("audit_integrity_gate_final_decision=GO|NO-GO"));
    assert!(CHECKLIST.contains("Regression: #4466"));
}

#[test]
fn checklist_contains_slo_threshold_policy_gate_convergence() {
    assert!(CHECKLIST.contains("## SLO Threshold/Policy Gate Convergence Gate (Issue #4468)"));
    assert!(CHECKLIST.contains("--slo-policy-report-file /tmp/deployment-slo-rollback-report.json"));
    assert!(CHECKLIST.contains("--slo-policy-max-age-seconds 1800"));
    assert!(CHECKLIST.contains(
        "slo_policy_reason_taxonomy_version=kamn.release.gonogo-slo-threshold-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "slo_policy_reason_codes_csv=gonogo_slo_policy_file_missing,gonogo_slo_policy_invalid_json,gonogo_slo_policy_schema_mismatch,gonogo_slo_policy_status_not_pass,gonogo_slo_policy_final_decision_not_go,gonogo_slo_policy_reason_key_mismatch,gonogo_slo_policy_reason_codes_not_empty,gonogo_slo_policy_freshness_window_exceeded"
    ));
    assert!(CHECKLIST.contains("slo_policy_gate_final_decision=GO|NO-GO"));
    assert!(CHECKLIST.contains("Regression: #4468"));
}

#[test]
fn checklist_contains_live_run_mode_rehearsal_lineage_gate() {
    assert!(CHECKLIST.contains("## Live Run-Mode Rehearsal Lineage Gate (Issue #3245)"));
    assert!(CHECKLIST.contains("run_local_live_node_validation_bundle_lane.sh"));
    assert!(CHECKLIST.contains("check_local_live_node_validation_bundle_policy.py"));
    assert!(CHECKLIST.contains("run_local_live_node_validation_bundle_contract_lane.sh"));
    assert!(CHECKLIST.contains("contracts.live_run_rehearsal_lineage_required=true"));
    assert!(CHECKLIST.contains("run_mode_check_status_mismatch"));
    assert!(CHECKLIST.contains("Regression: #3245"));
}

#[test]
fn checklist_contains_milestone_review_aggregate_lineage_gate() {
    assert!(CHECKLIST.contains("## Milestone Review Aggregate Lineage Gate (Issue #3247)"));
    assert!(CHECKLIST.contains("--deployment-preflight-summary-file"));
    assert!(CHECKLIST.contains("--deployment-preflight-policy-file"));
    assert!(CHECKLIST.contains("--live-node-validation-summary-file"));
    assert!(CHECKLIST.contains("--live-node-validation-policy-file"));
    assert!(CHECKLIST.contains("--go-no-go-gate-report-file"));
    assert!(CHECKLIST.contains("milestone_review_bundle"));
    assert!(CHECKLIST.contains("schema_version=kamn.release.milestone-review-bundle.v1"));
    assert!(CHECKLIST.contains("contracts.linked_artifact_lineage_required=true"));
    assert!(CHECKLIST.contains(
        "contracts.live_bundle_runtime_provider_client_required=KolmeRuntimeCommitLiveProvider"
    ));
    assert!(CHECKLIST.contains("contracts.go_no_go_gate_final_decision_required=GO"));
    assert!(CHECKLIST.contains("milestone_review_go_no_go_gate_report_missing"));
    assert!(CHECKLIST.contains("milestone_review_live_node_validation_runtime_provider_mismatch"));
    assert!(CHECKLIST.contains("milestone review bundle lineage mismatch"));
}

#[test]
fn checklist_contains_live_gonogo_convergence_boundary_governance_gate() {
    assert!(CHECKLIST.contains(
        "## Live Go/No-Go Evidence Convergence and Boundary Governance Gate (Issue #4434)"
    ));
    assert!(CHECKLIST.contains("run_gonogo_evidence_contract_lane.sh --max-seconds 120"));
    assert!(CHECKLIST.contains(
        "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900"
    ));
    assert!(CHECKLIST.contains(
        "live_gonogo_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "live_gonogo_reason_codes_csv=milestone_review_operator_runbook_missing,milestone_review_operator_runbook_markers_missing,milestone_review_deployment_preflight_summary_missing,milestone_review_deployment_preflight_summary_invalid_json,milestone_review_deployment_preflight_summary_schema_mismatch,milestone_review_deployment_preflight_summary_status_mismatch,milestone_review_deployment_preflight_scope_mismatch,milestone_review_deployment_preflight_policy_missing,milestone_review_deployment_preflight_policy_invalid_json,milestone_review_deployment_preflight_policy_schema_mismatch,milestone_review_deployment_preflight_policy_final_decision_mismatch,milestone_review_live_node_validation_summary_missing,milestone_review_live_node_validation_summary_invalid_json,milestone_review_live_node_validation_summary_schema_mismatch,milestone_review_live_node_validation_summary_status_mismatch,milestone_review_live_node_validation_scope_mismatch,milestone_review_live_node_validation_runtime_provider_mismatch,milestone_review_live_node_validation_lineage_contract_mismatch,milestone_review_live_node_validation_artifact_paths_missing,milestone_review_live_node_validation_rollback_lineage_missing,milestone_review_live_node_validation_recovery_lineage_missing,milestone_review_live_node_validation_policy_missing,milestone_review_live_node_validation_policy_invalid_json,milestone_review_live_node_validation_policy_schema_mismatch,milestone_review_live_node_validation_policy_final_decision_mismatch,milestone_review_go_no_go_gate_report_missing,milestone_review_go_no_go_gate_report_invalid_json,milestone_review_go_no_go_gate_schema_mismatch,milestone_review_go_no_go_gate_status_mismatch,milestone_review_go_no_go_gate_final_decision_mismatch,milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch,milestone_review_go_no_go_gate_combined_transport_reason_codes_mismatch,milestone_review_go_no_go_gate_combined_kolme_runtime_reason_code_mismatch,milestone_review_go_no_go_gate_kolme_runtime_commit_failure_taxonomy_version_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_version_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_status_mismatch,milestone_review_go_no_go_gate_combined_lane_marker_contract_status_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "live_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-live-boundary-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "live_gonogo_boundary_reason_codes_csv=live_gonogo_ci_smoke_seconds_exceeded,live_gonogo_local_heavy_seconds_exceeded,live_gonogo_local_heavy_opt_in_missing,live_gonogo_evidence_convergence_mismatch"
    ));
    assert!(CHECKLIST.contains("live_gonogo_ci_smoke_max_seconds=120"));
    assert!(CHECKLIST.contains("live_gonogo_local_heavy_max_seconds=900"));
    assert!(CHECKLIST.contains("Regression: #4441"));
    assert!(CHECKLIST.contains("Regression: #4442"));
}

#[test]
fn checklist_contains_staging_rehearsal_contract() {
    assert!(CHECKLIST.contains("## Staging Deploy + Rollback Rehearsal Contract"));
    assert!(CHECKLIST.contains("staging_rehearsal_contract.py"));
    assert!(CHECKLIST.contains("generate_staging_rehearsal_bundle.sh"));
    assert!(CHECKLIST.contains("check_staging_rehearsal_policy.sh"));
    assert!(CHECKLIST.contains("run_staging_rehearsal_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_staging_rehearsal_deep_lane.sh"));
    assert!(CHECKLIST.contains("kamn.release.staged-rehearsal-signoff.v1"));
    assert!(CHECKLIST.contains("staged_rehearsal_signoff_status=verified|fail-closed"));
    assert!(CHECKLIST.contains("--recovery-time-seconds"));
    assert!(CHECKLIST.contains("--max-allowed-recovery-time-seconds"));
    assert!(CHECKLIST.contains("mttr-threshold-exceeded"));
    assert!(CHECKLIST.contains("mttr_within_bound"));
    assert!(CHECKLIST.contains("Regression: #2337"));
}

#[test]
fn checklist_contains_durable_guard_recovery_evidence() {
    assert!(CHECKLIST.contains("## Durable Guard Migration + Recovery Matrix Evidence"));
    assert!(CHECKLIST.contains("run_durable_guard_recovery_contract_lane.sh"));
    assert!(CHECKLIST.contains("durable_guard_recovery_contract_lane_contract.py"));
    assert!(CHECKLIST.contains("run_durable_guard_recovery_deep_lane.sh"));
    assert!(CHECKLIST.contains("performance_durable_guard_recovery_contract_lane_budget"));
    assert!(CHECKLIST.contains("performance_durable_guard_recovery_matrix_deep_lane"));
    assert!(CHECKLIST.contains("performance_bundle_contract_lane_budget"));
    assert!(CHECKLIST.contains("performance_bundle_store_deep_lane_stress"));
}

#[test]
fn checklist_contains_persistence_evidence_tamper_freshness_gate() {
    assert!(CHECKLIST.contains("## Persistence Evidence Tamper/Freshness Gate (Issue #4389)"));
    assert!(CHECKLIST.contains("validate_persistence_adapters_live.sh"));
    assert!(CHECKLIST.contains(
        "persistence_gate_reason_taxonomy_version=kamn.runtime.persistence-gate-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "persistence_gate_reason_codes_csv=content_storage_corrupt_payload_rejected,did_registry_corrupt_payload_rejected,task_operation_snapshot_schema_mismatch_rejected,durable_guard_snapshot_schema_mismatch_rejected,channel_snapshot_corrupt_payload_rejected,channel_snapshot_schema_mismatch_rejected,message_lifecycle_snapshot_corrupt_payload_rejected,message_lifecycle_snapshot_schema_mismatch_rejected,runtime_snapshot_corrupt_payload_rejected,runtime_snapshot_state_version_regression_rejected,persistence_evidence_tamper_detected,persistence_evidence_freshness_window_exceeded,persistence_evidence_incomplete,persistence_ci_smoke_local_heavy_boundary_violation"
    ));
    assert!(CHECKLIST.contains("persistence_tamper_freshness_drift_fail_closed_status=verified"));
    assert!(CHECKLIST.contains("persistence_evidence_completeness_status=verified"));
    assert!(CHECKLIST.contains("persistence_ci_smoke_local_heavy_boundary_status=verified"));
    assert!(CHECKLIST.contains("persistence_ci_smoke_lane_cost_profile=low"));
    assert!(CHECKLIST.contains("persistence_local_heavy_execution_mode=opt_in"));
    assert!(CHECKLIST.contains("Regression: #4389"));
}

#[test]
fn checklist_contains_signer_incident_recovery_contract_and_cadence() {
    assert!(CHECKLIST
        .contains("## Signer Incident Recovery Contract and Deep-Lane Cadence (Issue #989)"));
    assert!(CHECKLIST.contains("run_signer_incident_recovery_lane.sh"));
    assert!(CHECKLIST.contains("check_signer_incident_recovery_policy.sh"));
    assert!(CHECKLIST.contains("run_signer_incident_recovery_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_signer_incident_recovery_deep_lane.sh"));
    assert!(CHECKLIST.contains("kamn.signer.incident-recovery-report.v1"));
    assert!(CHECKLIST.contains("kamn.signer.incident-recovery-deep-summary.v1"));
    assert!(CHECKLIST.contains("signer_incident_recovery_reason_codes:GO:v1"));
    assert!(CHECKLIST.contains("KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE"));
}

#[test]
fn checklist_contains_settlement_reconciliation_evidence_contract() {
    assert!(CHECKLIST.contains("## Settlement Reconciliation Evidence Contract"));
    assert!(CHECKLIST.contains("generate_settlement_reconciliation_evidence_bundle.sh",));
    assert!(CHECKLIST.contains("check_settlement_reconciliation_evidence_policy.sh",));
    assert!(CHECKLIST.contains("run_settlement_reconciliation_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_settlement_reconciliation_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_settlement_reconciliation_race_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/escrow_reconciliation/finality_race_cases.json"));
    assert!(CHECKLIST.contains("--ledger-reference-id"));
}

#[test]
fn checklist_contains_soc2_control_evidence_contract() {
    assert!(CHECKLIST.contains("## SOC2 Control Evidence Contract"));
    assert!(CHECKLIST.contains("generate_soc2_control_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_soc2_control_evidence_policy.sh"));
    assert!(CHECKLIST.contains("run_soc2_control_evidence_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_soc2_control_evidence_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_soc2_control_evidence_replay_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/compliance_soc2/control_evidence_replay_cases.json"));
}

#[test]
fn checklist_contains_dsar_legal_hold_evidence_contract() {
    assert!(CHECKLIST.contains("## DSAR Legal-Hold Evidence Contract"));
    assert!(CHECKLIST.contains("generate_dsar_legal_hold_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_dsar_legal_hold_policy.sh"));
    assert!(CHECKLIST.contains("run_dsar_legal_hold_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_dsar_legal_hold_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_dsar_legal_hold_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/compliance_dsar/legal_hold_precedence_cases.json"));
}

#[test]
fn checklist_contains_federated_did_handshake_evidence_contract() {
    assert!(CHECKLIST.contains("## Federated DID Handshake Evidence Contract"));
    assert!(CHECKLIST.contains("federated_did_handshake_contract.py"));
    assert!(CHECKLIST.contains("generate_federated_did_handshake_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_federated_did_handshake_policy.sh"));
    assert!(CHECKLIST.contains("run_federated_did_handshake_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_federated_did_handshake_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_federated_did_handshake_matrix.py"));
    assert!(CHECKLIST.contains("check_federated_did_handshake_deep_policy.sh"));
    assert!(CHECKLIST.contains("federated_did_handshake_deep_policy_contract.py"));
    assert!(CHECKLIST.contains("run_federated_did_handshake_deep_policy_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/federated_did_handshake/partition_replay_cases.json"));
    assert!(CHECKLIST.contains("cargo test -p kamn-core --test federated_did_handshake_runtime"));
}

#[test]
fn checklist_contains_federated_delegation_settlement_evidence_contract() {
    assert!(CHECKLIST.contains("## Federated Delegation Settlement Evidence Contract"));
    assert!(CHECKLIST.contains("generate_federated_delegation_settlement_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_federated_delegation_settlement_policy.sh"));
    assert!(CHECKLIST.contains("run_federated_delegation_settlement_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_federated_delegation_settlement_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_federated_delegation_settlement_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/federated_task_delegation/partition_replay_cases.json"));
}

#[test]
fn checklist_contains_kolme_version_compatibility_replay_evidence_contract() {
    assert!(CHECKLIST.contains("## Kolme Version Compatibility Replay Evidence Contract"));
    assert!(CHECKLIST.contains("validate_version_compatibility.py"));
    assert!(CHECKLIST.contains("generate_fork_compatibility_evidence.py"));
    assert!(CHECKLIST.contains("check_fork_compatibility_policy.py"));
    assert!(CHECKLIST.contains("run_version_compatibility_replay.py"));
    assert!(CHECKLIST.contains("check_runtime_commit_replay_policy.py"));
    assert!(CHECKLIST.contains("run_runtime_commit_replay_tamper_matrix.py"));
    assert!(CHECKLIST.contains("run_runtime_commit_adapter_contract_lane.sh"));
    assert!(CHECKLIST
        .contains("cargo test -p kamn-kolme --test runtime_commit_module_boundary_contracts"));
    assert!(
        CHECKLIST.contains("cargo test -p kamn-core --test kolme_runtime_commit_import_boundary")
    );
    assert!(CHECKLIST.contains("receipt_provider_mismatch"));
    assert!(CHECKLIST.contains("receipt_not_final"));
    assert!(CHECKLIST.contains("run_version_compatibility_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_runtime_commit_replay_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_version_compatibility_replay_deep_lane.sh"));
    assert!(CHECKLIST.contains("fixtures/kolme_compatibility/version_compatibility_cases.json"));
    assert!(CHECKLIST.contains("fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json"));
}

#[test]
fn checklist_contains_failover_sync_drill_evidence_contract() {
    assert!(CHECKLIST.contains("## Failover + Sync Drill Evidence Contract"));
    assert!(CHECKLIST.contains("select_failover_sync_drill_lane.sh"));
    assert!(CHECKLIST.contains("run_failover_sync_drill_preflight_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_failover_sync_drill_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_failover_sync_drill_suite.sh"));
}

#[test]
fn checklist_contains_peer_adapter_reason_projection_multi_process_gate() {
    assert!(CHECKLIST.contains(
        "## Peer Adapter Reason Projection and Multi-Process Validation Hooks (Issue #4320)"
    ));
    assert!(CHECKLIST.contains("cargo test -p kamn-core --test p2p_peer_adapter_reason_projection"));
    assert!(CHECKLIST.contains("validate_libp2p_convergence_process_isolated_live.sh"));
    assert!(CHECKLIST.contains("check_libp2p_convergence_process_isolated_live_policy.sh"));
    assert!(
        CHECKLIST.contains("validate_libp2p_convergence_process_isolated_live_contract_lane.sh")
    );
    assert!(CHECKLIST.contains(
        "peer_adapter_reason_taxonomy_version=kamn.runtime.peer-adapter-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST
        .contains("peer_integrity_fail_closed_reason_code=p2p_transport_unknown_sender_peer"));
    assert!(CHECKLIST.contains(
        "peer_adapter_reason_projection_timeout_code=p2p_live_reconnect_retry_dial_timeout"
    ));
    assert!(CHECKLIST.contains(
        "peer_adapter_reason_projection_budget_exhausted_code=p2p_live_reconnect_retry_budget_exhausted"
    ));
    assert!(CHECKLIST.contains("peer_adapter_multi_process_validation_local_heavy_status=required"));
    assert!(CHECKLIST.contains("Regression: #4320"));
}

#[test]
fn checklist_contains_live_network_pilot_launch_and_rollback_evidence_gates() {
    assert!(CHECKLIST.contains("## Live-Network Pilot Launch and Rollback Evidence Gates"));
    assert!(CHECKLIST.contains("run_live_network_smoke_lane.sh"));
    assert!(CHECKLIST.contains("run_live_network_pilot_deep_lane.sh"));
    assert!(CHECKLIST.contains("check_live_network_pilot_artifact_summary_policy.sh"));
    assert!(CHECKLIST.contains("run_live_network_pilot_deep_contract_lane.sh"));
    assert!(CHECKLIST.contains("select_live_network_partition_reconnect_lane.sh"));
    assert!(CHECKLIST.contains("run_live_network_partition_reconnect_smoke_lane.sh"));
    assert!(CHECKLIST.contains("run_live_network_partition_reconnect_deep_lane.sh"));
    assert!(CHECKLIST.contains("check_live_network_partition_reconnect_policy.sh"));
    assert!(CHECKLIST.contains("run_live_network_partition_reconnect_contract_lane.sh"));
    assert!(
        CHECKLIST.contains("fixtures/runtime/live_network_partition_reconnect_matrix_cases.json")
    );
}

#[test]
fn checklist_contains_watchdog_proof_consensus_evidence_contract() {
    assert!(CHECKLIST.contains("## Validator/Watchdog Proof Consensus Evidence Contract"));
    assert!(CHECKLIST.contains("run_watchdog_proof_consensus_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_watchdog_proof_consensus_deep_lane.sh"));
    assert!(CHECKLIST.contains("generate_watchdog_proof_consensus_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_watchdog_proof_consensus_policy.sh"));
    assert!(CHECKLIST.contains("KAMN_WATCHDOG_PROOF_CONSENSUS_DEEP_CADENCE"));
}

#[test]
fn checklist_contains_governance_simulation_and_human_veto_evidence_contract() {
    assert!(CHECKLIST.contains("## Governance Simulation and Human-Veto Evidence Contract"));
    assert!(CHECKLIST.contains("generate_governance_simulation_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_governance_simulation_policy.sh"));
    assert!(CHECKLIST.contains("governance_simulation_contract_lane_contract.py"));
    assert!(CHECKLIST.contains("framework.contract_lane_helpers"));
    assert!(CHECKLIST.contains("run_governance_simulation_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_governance_simulation_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_governance_simulation_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/governance_simulation/veto_timelock_cases.json"));
}

#[test]
fn checklist_contains_governance_stake_slash_risk_threshold_contract() {
    assert!(CHECKLIST.contains("## Governance Stake/Slash Risk Threshold Contract"));
    assert!(CHECKLIST.contains("generate_stake_slash_risk_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_stake_slash_risk_policy.sh"));
    assert!(CHECKLIST.contains("stake_slash_risk_contract_lane_contract.py"));
    assert!(CHECKLIST.contains("framework.contract_lane_helpers"));
    assert!(CHECKLIST.contains("run_stake_slash_risk_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_stake_slash_risk_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_stake_slash_risk_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/governance_stake_slash/risk_threshold_cases.json"));
}

#[test]
fn checklist_contains_reputation_dispute_evidence_contract() {
    assert!(CHECKLIST.contains("## Reputation Dispute Evidence Contract"));
    assert!(CHECKLIST.contains("reputation_dispute_contract_lane_contract.py"));
    assert!(CHECKLIST.contains("framework.contract_lane_helpers"));
    assert!(CHECKLIST.contains("generate_reputation_dispute_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_reputation_dispute_policy.sh"));
    assert!(CHECKLIST.contains("run_reputation_dispute_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_reputation_dispute_deep_lane.sh"));
    assert!(CHECKLIST.contains("run_reputation_dispute_matrix.py"));
    assert!(CHECKLIST.contains("fixtures/reputation_dispute/replay_cases.json"));
}

#[test]
fn checklist_contains_token_launch_handoff_evidence_contract() {
    assert!(CHECKLIST.contains("## Token Launch Handoff Evidence Contract"));
    assert!(CHECKLIST.contains("generate_token_launch_handoff_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_token_launch_handoff_policy.sh"));
    assert!(CHECKLIST.contains("run_token_launch_handoff_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_token_launch_handoff_deep_lane.sh"));
}

#[test]
fn checklist_contains_treasury_disbursement_approval_evidence_contract() {
    assert!(CHECKLIST.contains("## Treasury Disbursement Approval Evidence Contract"));
    assert!(CHECKLIST.contains("generate_treasury_disbursement_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_treasury_disbursement_policy.sh"));
    assert!(CHECKLIST.contains("treasury_disbursement_contract_lane_contract.py"));
    assert!(CHECKLIST.contains("run_treasury_disbursement_contract_lane.sh"));
}

#[test]
fn checklist_contains_mainnet_cutover_manifest_contract() {
    assert!(CHECKLIST.contains("## Mainnet Cutover Manifest Validation Contract"));
    assert!(CHECKLIST.contains("fixtures/mainnet_cutover/mainnet_cutover_manifest.schema.json"));
    assert!(CHECKLIST.contains("validate_mainnet_cutover_manifest.py"));
    assert!(CHECKLIST.contains("run_mainnet_cutover_contract_lane.sh"));
}

#[test]
fn checklist_contains_cutover_rollback_evidence_contract() {
    assert!(CHECKLIST.contains("## Cutover Rollback Evidence Contract"));
    assert!(CHECKLIST.contains("generate_cutover_rollback_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_cutover_rollback_evidence_policy.sh"));
    assert!(CHECKLIST.contains("run_cutover_rollback_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_cutover_rollback_deep_lane.sh"));
}

#[test]
fn checklist_contains_launch_canary_critical_path_contract() {
    assert!(CHECKLIST.contains("## Launch Canary Critical-Path Contract"));
    assert!(CHECKLIST.contains("fixtures/launch_canary/critical_path_probe_cases.json"));
    assert!(CHECKLIST.contains("run_launch_canary_matrix.py"));
    assert!(CHECKLIST.contains("launch_canary_contract_lane_contract.py"));
    assert!(CHECKLIST.contains("run_launch_canary_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_launch_canary_deep_lane.sh"));
}

#[test]
fn checklist_contains_post_cutover_slo_evidence_contract() {
    assert!(CHECKLIST.contains("## Post-Cutover SLO Gate Evidence Contract"));
    assert!(CHECKLIST.contains("generate_post_cutover_slo_evidence_bundle.sh"));
    assert!(CHECKLIST.contains("check_post_cutover_slo_policy.sh"));
    assert!(CHECKLIST.contains("post_cutover_slo_contract_lane_contract.py"));
    assert!(CHECKLIST.contains("run_post_cutover_slo_contract_lane.sh"));
    assert!(CHECKLIST.contains("run_post_cutover_slo_deep_lane.sh"));
    assert!(CHECKLIST.contains("alert_rule_promotion_gate_status=verified"));
    assert!(CHECKLIST.contains("burn_rate_parity_status=verified"));
    assert!(CHECKLIST.contains("ci_local_promotion_budget_boundary_status=verified"));
    assert!(CHECKLIST.contains(
        "alert_governance_reason_taxonomy_version=kamn.runtime.alert-governance-reason-taxonomy.v1",
    ));
    assert!(CHECKLIST.contains(
        "alert_governance_reason_codes_csv=alert_rule_promotion_stalled,burn_rate_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded",
    ));
    assert!(CHECKLIST.contains("KAMN_POST_CUTOVER_SLO_CI_LOCAL_PROMOTION_MAX_SECONDS"));
    assert!(CHECKLIST.contains("KAMN_POST_CUTOVER_SLO_DEEP_LOCAL_ONLY"));
}

#[test]
fn regression_requires_rollback_precheck_in_checklist() {
    // Regression: #173
    assert!(CHECKLIST.contains("Rollback precheck result: PASS"));
}

#[test]
fn regression_requires_staging_rehearsal_mismatch_guard() {
    // Regression: #623
    assert!(CHECKLIST.contains(
        "rollback target hash mismatch and incomplete rehearsal evidence force `NO-GO` (`Regression: #623`)."
    ));
}

#[test]
fn regression_requires_chain_receipt_evidence_guard_marker() {
    // Regression: #678
    assert!(CHECKLIST.contains(
        "missing or invalid chain receipt evidence forces `NO-GO` (`Regression: #678`)."
    ));
    assert!(CHECKLIST.contains(
        "timeout-before-finality pending receipts and failed receipts force `NO-GO` (`Regression: #678`)."
    ));
}

#[test]
fn regression_requires_ledger_reference_evidence_guard_marker() {
    // Regression: #717
    assert!(CHECKLIST.contains(
        "missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`)."
    ));
}

#[test]
fn regression_requires_durable_guard_shared_contract_marker() {
    // Regression: #1242
    assert!(CHECKLIST.contains(
        "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1242`)."
    ));
}

#[test]
fn regression_requires_failover_sync_budget_and_cadence_guard_markers() {
    // Regression: #788
    assert!(CHECKLIST
        .contains("preflight runtime budget overruns force lane failure (`Regression: #788`)."));
    assert!(CHECKLIST.contains(
        "unscheduled deep-lane execution force-fails via scheduled-only cadence guard (`Regression: #788`)."
    ));
}

#[test]
fn regression_requires_live_network_pilot_launch_and_rollback_guard_marker() {
    // Regression: #830
    assert!(CHECKLIST.contains(
        "missing smoke/deep pilot evidence or non-`GO` pilot decisions force launch `NO-GO` and trigger rollback review (`Regression: #830`)."
    ));
}

#[test]
fn regression_requires_live_network_partition_reconnect_guard_marker() {
    // Regression: #982
    assert!(CHECKLIST.contains(
        "stale/tampered partition/reconnect matrix artifacts and replay anomalies force `NO-GO` (`Regression: #982`)."
    ));
}

#[test]
fn regression_requires_watchdog_proof_consensus_budget_and_cadence_guard_marker() {
    // Regression: #996
    assert!(CHECKLIST.contains(
        "proof-consensus deep-lane budget overruns and unscheduled cadence execution force `NO-GO` (`Regression: #996`)."
    ));
}

#[test]
fn regression_requires_soc2_control_evidence_guard_marker() {
    // Regression: #732
    assert!(CHECKLIST.contains(
        "tampered final decisions and incomplete/tampered control evidence force `NO-GO` (`Regression: #732`)."
    ));
}

#[test]
fn regression_requires_dsar_legal_hold_evidence_guard_marker() {
    // Regression: #732
    assert!(CHECKLIST.contains(
        "legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`)."
    ));
}

#[test]
fn regression_requires_federated_did_handshake_evidence_guard_marker() {
    // Regression: #734
    assert!(CHECKLIST.contains(
        "replay/downgrade attempts, quorum shortfalls, and tampered final decisions force `NO-GO` (`Regression: #734`)."
    ));
}

#[test]
fn regression_requires_federated_runtime_trust_store_guard_marker() {
    // Regression: #1002
    assert!(CHECKLIST.contains(
        "runtime trust-store misses and quorum shortfalls must remain fail-closed with deterministic reason codes (`Regression: #1002`)."
    ));
}

#[test]
fn regression_requires_federated_deep_lane_tamper_guard_marker() {
    // Regression: #1003
    assert!(CHECKLIST.contains(
        "stale/tampered federated handshake deep-lane summary artifacts must remain `NO-GO` (`Regression: #1003`)."
    ));
}

#[test]
fn regression_requires_federated_delegation_settlement_evidence_guard_marker() {
    // Regression: #734
    assert!(CHECKLIST.contains(
        "settlement reference drift, replay attempts, quorum shortfalls, and tampered final decisions force `NO-GO` (`Regression: #734`)."
    ));
}

#[test]
fn regression_requires_kolme_incompatible_upgrade_signature_guard_marker() {
    // Regression: #775
    assert!(CHECKLIST.contains(
        "incompatible upgrade signature (`kamn 1.2.x` + `kolme 0.14.x`) remains blocked (`Regression: #775`)."
    ));
}

#[test]
fn regression_requires_kolme_runtime_commit_replay_guard_marker() {
    // Regression: #827
    assert!(CHECKLIST.contains(
        "runtime commit replay/tamper mismatches and non-final receipts force `NO-GO` (`Regression: #827`)."
    ));
}

#[test]
fn regression_requires_adapter_runtime_commit_replay_guard_marker() {
    // Regression: #980
    assert!(CHECKLIST.contains(
        "adapter transport/provider mismatch and non-final receipt reason-code checks remain fail-closed (`Regression: #980`)."
    ));
}

#[test]
fn regression_requires_kolme_fork_release_drift_guard_marker() {
    // Regression: #1401
    assert!(CHECKLIST.contains("fork release-tag drift remains blocked (`Regression: #1401`)."));
}

#[test]
fn regression_requires_kolme_fork_policy_checker_guard_marker() {
    // Regression: #1402
    assert!(CHECKLIST.contains(
        "fork policy checker rejects malformed schema, tuple mismatch, and missing required reason codes (`Regression: #1402`)."
    ));
}

#[test]
fn regression_requires_governance_simulation_and_veto_guard_marker() {
    // Regression: #733
    assert!(CHECKLIST.contains(
        "simulation/veto bypass attempts and tampered evidence bundles force `NO-GO` (`Regression: #733`)."
    ));
}

#[test]
fn regression_requires_governance_stake_slash_risk_guard_marker() {
    // Regression: #733
    assert!(CHECKLIST.contains(
        "unsafe threshold bypass attempts and tampered risk evidence force `NO-GO` (`Regression: #733`)."
    ));
}

#[test]
fn regression_requires_reputation_dispute_evidence_guard_marker() {
    // Regression: #730
    assert!(CHECKLIST.contains(
        "tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`)."
    ));
}

#[test]
fn regression_requires_token_launch_handoff_evidence_guard_marker() {
    // Regression: #714
    assert!(CHECKLIST.contains(
        "supply/allocation invariant drift and insufficient approvals force `NO-GO` (`Regression: #714`)."
    ));
}

#[test]
fn regression_requires_treasury_disbursement_approval_guard_marker() {
    // Regression: #716
    assert!(CHECKLIST.contains(
        "insufficient approvals, approval-window closure, and daily-limit overruns force `NO-GO` (`Regression: #716`)."
    ));
}

#[test]
fn regression_requires_treasury_shared_contract_lane_marker() {
    // Regression: #1278
    assert!(CHECKLIST.contains(
        "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1278`)."
    ));
}

#[test]
fn regression_requires_mainnet_cutover_dependency_and_approval_guards() {
    // Regression: #705
    assert!(CHECKLIST.contains(
        "unresolved/non-prior dependencies and insufficient approvals force `NO-GO` (`Regression: #705`)."
    ));
}

#[test]
fn regression_requires_cutover_rollback_evidence_guard_marker() {
    // Regression: #708
    assert!(CHECKLIST.contains(
        "missing failed-checkpoint evidence and rollback-target hash mismatch force `NO-GO` (`Regression: #708`)."
    ));
}

#[test]
fn regression_requires_launch_canary_evidence_guard_marker() {
    // Regression: #710
    assert!(CHECKLIST.contains(
        "missing probe evidence and failing critical-path probes force `NO-GO` (`Regression: #710`)."
    ));
}

#[test]
fn regression_requires_launch_canary_shared_contract_lane_marker() {
    // Regression: #1286
    assert!(CHECKLIST.contains(
        "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1286`)."
    ));
}

#[test]
fn regression_requires_post_cutover_slo_evidence_guard_marker() {
    // Regression: #711
    assert!(CHECKLIST.contains(
        "stale snapshots and incomplete SLO evidence force `NO-GO` (`Regression: #711`)."
    ));
}

#[test]
fn regression_requires_post_cutover_slo_shared_contract_lane_marker() {
    // Regression: #1282
    assert!(CHECKLIST.contains(
        "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1282`)."
    ));
}

#[test]
fn regression_requires_signer_incident_recovery_stale_artifact_guard_marker() {
    // Regression: #989
    assert!(CHECKLIST.contains(
        "stale deep-lane artifacts, unscheduled deep-lane execution, and incident recovery policy drift force `NO-GO` (`Regression: #989`)."
    ));
}
