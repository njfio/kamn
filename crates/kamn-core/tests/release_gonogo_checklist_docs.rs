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
fn checklist_contains_full_stack_harness_marker_checker_reason_mapping_gate() {
    assert!(CHECKLIST
        .contains("## Full-Stack Harness Marker Checker Reason Mapping Gate (Issue #4196)"));
    assert!(CHECKLIST.contains("test_check_full_io_scenario_matrix_live_policy.sh"));
    assert!(CHECKLIST.contains("test_validate_full_io_scenario_matrix_live_contract_lane.sh"));
    assert!(CHECKLIST.contains(
        "full_io_harness_policy_reason_taxonomy_version=kamn.runtime.full-io-scenario-matrix-policy-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "full_io_harness_policy_reason_codes_csv=full_io_scenario_matrix_policy_schema_mismatch,full_io_scenario_matrix_policy_status_mismatch,full_io_scenario_matrix_policy_final_decision_mismatch,full_io_scenario_matrix_policy_ci_fast_gate_mismatch,full_io_scenario_matrix_policy_process_harness_mismatch,full_io_scenario_matrix_policy_api_route_matrix_mismatch,full_io_scenario_matrix_policy_auth_failure_matrix_mismatch,full_io_scenario_matrix_policy_websocket_matrix_mismatch,full_io_scenario_matrix_policy_multinode_propagation_mismatch,full_io_scenario_matrix_policy_fast_gate_exclusion_mismatch,full_io_scenario_matrix_policy_fast_gate_reason_mismatch,full_io_scenario_matrix_policy_lane_mode_invalid,full_io_scenario_matrix_policy_command_count_invalid,full_io_scenario_matrix_policy_artifact_paths_invalid,full_io_scenario_matrix_policy_dry_run_eligibility_mismatch,full_io_scenario_matrix_policy_dry_run_command_count_mismatch,full_io_scenario_matrix_policy_dry_run_command_status_mismatch,full_io_scenario_matrix_policy_dry_run_reason_code_mismatch,full_io_scenario_matrix_policy_run_mode_exclusion_mismatch,full_io_scenario_matrix_policy_run_mode_command_count_mismatch,full_io_scenario_matrix_policy_run_mode_command_status_mismatch,full_io_scenario_matrix_policy_run_mode_reason_code_mismatch,full_io_scenario_matrix_policy_expected_decision_mismatch"
    ));
    assert!(CHECKLIST.contains("full_io_harness_policy_reason_codes_value=none|<csv>"));
    assert!(CHECKLIST.contains("full_io_scenario_matrix_policy_status=verified|failed"));
    assert!(CHECKLIST.contains("full_io_scenario_matrix_policy_process_harness_mismatch"));
    assert!(CHECKLIST.contains("full_io_scenario_matrix_policy_dry_run_command_count_mismatch"));
    assert!(CHECKLIST.contains("full_io_scenario_matrix_policy_dry_run_command_status_mismatch"));
    assert!(CHECKLIST.contains("full_io_scenario_matrix_policy_expected_decision_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4196"));
}

#[test]
fn checklist_contains_runtime_signer_key_source_reason_mapping_gate() {
    assert!(CHECKLIST
        .contains("## Runtime Signer Key-Source/Fallback Reason Mapping Gate (Issue #4356)"));
    assert!(CHECKLIST.contains(
        "key_source_reason_taxonomy_version=kamn.kolme.local-kamn-live-runtime-key-source-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "key_source_reason_codes_csv=runtime_signer_key_source_contract_version_missing,runtime_signer_key_source_contract_version_mismatch,runtime_signer_key_source_contract_version_contract_mismatch,runtime_signer_key_source_missing,runtime_signer_key_source_invalid,runtime_signer_key_source_profile_pair_disallowed,runtime_signer_key_source_contract_mismatch,runtime_commit_signer_key_source_marker_missing,runtime_commit_fallback_private_key_command_marker_detected,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation"
    ));
    assert!(CHECKLIST.contains("key_source_reason_codes_value=none|<csv>"));
    assert!(CHECKLIST.contains("runtime_commit_signer_key_source_marker_missing"));
    assert!(CHECKLIST.contains("Regression: #4356"));
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
fn checklist_contains_service_api_axum_protocol_mismatch_reason_mapping_gate() {
    assert!(CHECKLIST.contains(
        "## Service API Axum Protocol Mismatch Reason Mapping Gate (Issues #4266, #4270, #4271)"
    ));
    assert!(CHECKLIST.contains("test_check_service_api_axum_ingress_live_policy.sh"));
    assert!(CHECKLIST.contains("test_validate_service_api_axum_ingress_live_contract_lane.sh"));
    assert!(CHECKLIST.contains("service_api_axum_protocol_mismatch_reason_mapping_status=verified"));
    assert!(CHECKLIST.contains(
        "service_api_axum_protocol_mismatch_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "service_api_axum_protocol_mismatch_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation"
    ));
    assert!(CHECKLIST.contains("service_api_axum_protocol_mismatch_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("admission_inflight_budget_status=verified"));
    assert!(CHECKLIST.contains("admission_queue_budget_status=verified"));
    assert!(CHECKLIST.contains("admission_inflight_budget_limit=32"));
    assert!(CHECKLIST.contains("admission_queue_budget_limit=1"));
    assert!(CHECKLIST.contains(
        "admission_budget_reason_taxonomy_version=kamn.runtime.service-api-admission-budget-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "admission_budget_reason_codes_csv=admission_inflight_budget_mismatch,admission_queue_budget_mismatch"
    ));
    assert!(CHECKLIST.contains("service_api_axum_policy_marker_missing:<field>"));
    assert!(CHECKLIST
        .contains("service_api_axum_policy_protocol_compliance_reason_taxonomy_version_mismatch"));
    assert!(CHECKLIST
        .contains("service_api_axum_policy_admission_budget_reason_taxonomy_version_mismatch"));
    assert!(CHECKLIST.contains("service_api_axum_policy_admission_inflight_budget_limit_mismatch"));
    assert!(CHECKLIST.contains("service_api_axum_policy_admission_queue_budget_limit_mismatch"));
    assert!(CHECKLIST.contains("service_api_axum_policy_body_size_limit_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4270"));
    assert!(CHECKLIST.contains("Regression: #4271"));
}

#[test]
fn checklist_contains_service_api_axum_protocol_taxonomy_runbook_parity_gate() {
    assert!(CHECKLIST
        .contains("## Service API Axum Protocol Taxonomy/Runbook Parity Gate (Issue #4267)"));
    assert!(CHECKLIST.contains("test_validate_service_api_axum_ingress_live_contract_lane.sh"));
    assert!(CHECKLIST.contains("protocol_taxonomy_mapping_status=verified"));
    assert!(CHECKLIST.contains("runbook_marker_parity_status=verified"));
    assert!(CHECKLIST.contains(
        "protocol_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-taxonomy-runbook-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "protocol_taxonomy_runbook_reason_codes_csv=protocol_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "protocol_compliance_reason_taxonomy_version=kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "request_validation_reason_taxonomy_version=kamn.runtime.service-api-request-validation-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "error_envelope_reason_taxonomy_version=kamn.runtime.service-api-error-envelope-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "service_api_axum_protocol_mismatch_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains("protocol_taxonomy_mapping_drift_detected"));
    assert!(CHECKLIST.contains("runbook_marker_parity_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4272"));
    assert!(CHECKLIST.contains("Regression: #4273"));
}

#[test]
fn checklist_contains_service_api_axum_admission_decision_taxonomy_runbook_parity_gate() {
    assert!(CHECKLIST.contains(
        "## Service API Axum Admission Decision Taxonomy/Runbook Parity Gate (Issues #4222, #4227, #4228)"
    ));
    assert!(CHECKLIST.contains("test_validate_service_api_axum_ingress_live_contract_lane.sh"));
    assert!(CHECKLIST.contains("test_check_service_api_axum_ingress_live_policy.sh"));
    assert!(CHECKLIST.contains("admission_decision_taxonomy_status=verified"));
    assert!(CHECKLIST.contains("admission_decision_accept_status=verified"));
    assert!(CHECKLIST.contains("admission_decision_defer_status=verified"));
    assert!(CHECKLIST.contains("admission_decision_reject_status=verified"));
    assert!(CHECKLIST.contains(
        "admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject"
    ));
    assert!(CHECKLIST.contains("admission_decision_taxonomy_mapping_status=verified"));
    assert!(CHECKLIST.contains("admission_decision_runbook_marker_parity_status=verified"));
    assert!(CHECKLIST.contains(
        "admission_decision_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.service-api-axum-admission-decision-runbook-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "admission_decision_taxonomy_runbook_reason_codes_csv=admission_decision_taxonomy_mapping_drift_detected,admission_runbook_marker_parity_mismatch"
    ));
    assert!(CHECKLIST
        .contains("service_api_axum_policy_admission_decision_reason_taxonomy_version_mismatch"));
    assert!(
        CHECKLIST.contains("service_api_axum_policy_admission_decision_reason_codes_csv_mismatch")
    );
    assert!(CHECKLIST
        .contains("service_api_axum_policy_marker_missing:admission_decision_defer_status"));
    assert!(CHECKLIST.contains("protocol_taxonomy_mapping_drift_detected"));
    assert!(CHECKLIST.contains("runbook_marker_parity_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4227"));
    assert!(CHECKLIST.contains("Regression: #4228"));
}

#[test]
fn checklist_contains_service_api_websocket_session_evidence_convergence_gate() {
    assert!(CHECKLIST
        .contains("## Service API Websocket Session Evidence Convergence Gate (Issue #4268)"));
    assert!(CHECKLIST.contains(
        "check_service_api_websocket_live_evidence_convergence.sh --report-file /tmp/service-api-websocket-live-contract-lane-report.json --policy-file /tmp/service-api-websocket-live-policy-report.json --output-json /tmp/service-api-websocket-live-convergence-report.json"
    ));
    assert!(CHECKLIST.contains("service_api_websocket_evidence_convergence_status=verified"));
    assert!(CHECKLIST.contains("promotion_decision_reason_mapping_status=verified"));
    assert!(CHECKLIST.contains(
        "service_api_websocket_evidence_reason_taxonomy_version=kamn.runtime.service-api-websocket-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "service_api_websocket_evidence_reason_codes_csv=service_api_websocket_evidence_link_missing,service_api_websocket_evidence_payload_tamper_detected,service_api_websocket_promotion_decision_reason_mapping_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_taxonomy_version=kamn.runtime.service-api-websocket-promotion-decision-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_codes_csv=service_api_websocket_policy_required_field_missing,service_api_websocket_policy_marker_missing,service_api_websocket_policy_reason_taxonomy_mismatch,service_api_websocket_policy_idle_timeout_contract_mismatch,ci_fast_gate_failed,service_api_websocket_policy_expected_decision_mismatch,service_api_websocket_policy_violation"
    ));
    assert!(CHECKLIST.contains("promotion_decision_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("service_api_websocket_evidence_link_missing:source_report_file"));
    assert!(CHECKLIST.contains("service_api_websocket_promotion_decision_reason_mapping_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4274"));
    assert!(CHECKLIST.contains("Regression: #4275"));
}

#[test]
fn checklist_contains_service_api_axum_admission_backpressure_evidence_convergence_gate() {
    assert!(CHECKLIST.contains(
        "## Service API Axum Admission/Backpressure Evidence Convergence Gate (Issues #4223, #4229, #4230)"
    ));
    assert!(CHECKLIST.contains(
        "check_service_api_axum_ingress_live_evidence_convergence.sh --report-file /tmp/service-api-axum-ingress-contract-lane-report.json --policy-file /tmp/service-api-axum-ingress-policy-report.json --output-json /tmp/service-api-axum-ingress-convergence-report.json"
    ));
    assert!(CHECKLIST.contains("test_check_service_api_axum_ingress_live_evidence_convergence.sh"));
    assert!(CHECKLIST.contains("service_api_axum_evidence_convergence_status=verified"));
    assert!(CHECKLIST.contains("promotion_decision_reason_mapping_status=verified"));
    assert!(CHECKLIST.contains(
        "service_api_axum_evidence_reason_taxonomy_version=kamn.runtime.service-api-axum-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "service_api_axum_evidence_reason_codes_csv=service_api_axum_evidence_link_missing,service_api_axum_evidence_payload_tamper_detected,service_api_axum_promotion_decision_reason_mapping_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation"
    ));
    assert!(CHECKLIST.contains("promotion_decision_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("service_api_axum_evidence_link_missing:source_report_file"));
    assert!(CHECKLIST.contains("service_api_axum_promotion_decision_reason_mapping_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4229"));
    assert!(CHECKLIST.contains("Regression: #4230"));
}

#[test]
fn checklist_contains_block_reconciliation_partition_healing_mismatch_mapping_gate() {
    assert!(CHECKLIST.contains(
        "## Block Reconciliation Partition-Healing Mismatch Mapping Gate (Issues #4251, #4255, #4256)"
    ));
    assert!(CHECKLIST.contains("test_check_block_reconciliation_partition_rejoin_live_policy.sh"));
    assert!(CHECKLIST
        .contains("test_validate_block_reconciliation_partition_rejoin_live_contract_lane.sh"));
    assert!(CHECKLIST.contains("partition_healing_mismatch_reason_mapping_status=verified"));
    assert!(CHECKLIST.contains(
        "partition_healing_mismatch_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-healing-mismatch-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "partition_healing_mismatch_reason_codes_csv=block_reconciliation_partition_rejoin_policy_required_field_missing,block_reconciliation_partition_rejoin_policy_marker_mismatch,block_reconciliation_partition_rejoin_policy_transport_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_taxonomy_mismatch,block_reconciliation_partition_rejoin_policy_recovery_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid,block_reconciliation_partition_rejoin_policy_lane_mode_contract_mismatch,block_reconciliation_partition_rejoin_policy_ci_fast_gate_failed,block_reconciliation_partition_rejoin_policy_expected_decision_mismatch,block_reconciliation_partition_rejoin_policy_violation"
    ));
    assert!(CHECKLIST.contains("partition_healing_mismatch_reason_code=none|<reason>"));
    assert!(CHECKLIST
        .contains("block_reconciliation_partition_rejoin_policy_required_field_missing:<field>"));
    assert!(CHECKLIST.contains(
        "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid"
    ));
    assert!(CHECKLIST.contains(
        "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_csv_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "block_reconciliation_partition_rejoin_policy_reconciliation_consistency_reason_taxonomy_version_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "block_reconciliation_partition_rejoin_policy_consistency_classification_status_mismatch"
    ));
    assert!(CHECKLIST.contains("Regression: #4255"));
    assert!(CHECKLIST.contains("Regression: #4256"));
}

#[test]
fn checklist_contains_fork_choice_finality_evidence_convergence_gate() {
    assert!(CHECKLIST.contains(
        "## Fork-Choice Finality Evidence Convergence Gate (Issues #4253, #4259, #4260)"
    ));
    assert!(CHECKLIST.contains(
        "check_libp2p_convergence_process_isolated_live_evidence_convergence.sh --report-file /tmp/libp2p-convergence-process-isolated-live-contract-lane-report.json --policy-file /tmp/libp2p-convergence-process-isolated-live-policy-report.json --output-json /tmp/libp2p-convergence-process-isolated-live-convergence-report.json"
    ));
    assert!(CHECKLIST
        .contains("test_check_libp2p_convergence_process_isolated_live_evidence_convergence.sh"));
    assert!(CHECKLIST.contains("libp2p_finality_evidence_convergence_status=verified"));
    assert!(CHECKLIST.contains("promotion_decision_reason_mapping_status=verified"));
    assert!(CHECKLIST.contains(
        "libp2p_finality_evidence_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "libp2p_finality_evidence_reason_codes_csv=libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_codes_csv=libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation"
    ));
    assert!(CHECKLIST.contains("promotion_decision_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("libp2p_finality_evidence_link_missing:source_report_file"));
    assert!(CHECKLIST.contains("libp2p_finality_evidence_payload_tamper_detected:<field>"));
    assert!(CHECKLIST.contains("libp2p_finality_promotion_decision_reason_mapping_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4259"));
    assert!(CHECKLIST.contains("Regression: #4260"));
}

#[test]
fn checklist_contains_shutdown_signal_lifecycle_reason_mapping_gate() {
    assert!(CHECKLIST.contains("## Shutdown Signal Lifecycle Reason Mapping Gate (Issue #4331)"));
    assert!(CHECKLIST.contains(
        "main_tests::runtime_tests::regression_full_supervisor_stop_contract_classifier_rejects_empty_or_non_numeric_signal_tick -- --exact"
    ));
    assert!(CHECKLIST.contains(
        "main_tests::runtime_tests::regression_shutdown_policy_rejects_os_signal_hooks_for_non_daemon_modes -- --exact"
    ));
    assert!(CHECKLIST.contains(
        "shutdown_signal_reason_taxonomy_version=kamn.runtime.shutdown-signal-lifecycle-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "shutdown_signal_reason_codes_csv=full_supervisor_stop_invalid_shutdown_drain_status,full_supervisor_stop_invalid_shutdown_snapshot_flush_status,full_supervisor_stop_not_signaled_status_mismatch,full_supervisor_stop_not_signaled_snapshot_flush_mismatch,full_supervisor_stop_missing_signal_tick,full_supervisor_stop_missing_drain_ticks,full_supervisor_stop_missing_timeout_ticks,full_supervisor_stop_missing_ignored_signals,full_supervisor_stop_graceful_status_mismatch,full_supervisor_stop_graceful_snapshot_flush_status_mismatch,full_supervisor_stop_graceful_timeout_status_mismatch,full_supervisor_stop_graceful_timeout_snapshot_flush_status_mismatch,full_supervisor_stop_unknown_completion_reason"
    ));
    assert!(CHECKLIST.contains("shutdown_signal_reason_codes_value=none|<csv>"));
    assert!(CHECKLIST.contains("shutdown_signal_hook_runtime_modes=daemon|full"));
    assert!(
        CHECKLIST.contains("shutdown_signal_hook_explicit_override=--daemon-shutdown-os-signals")
    );
    assert!(CHECKLIST.contains("full_supervisor_stop_missing_signal_tick"));
    assert!(CHECKLIST.contains("Regression: #4331"));
}

#[test]
fn checklist_contains_shutdown_drain_checkpoint_reconciliation_gate() {
    assert!(CHECKLIST
        .contains("## Shutdown Drain/Checkpoint Reconciliation Gate (Issues #4332, #4333)"));
    assert!(CHECKLIST.contains(
        "shutdown_checkpoint_reconciliation_reason_taxonomy_version=kamn.runtime.shutdown-checkpoint-reconciliation-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains("full_supervisor_stop_graceful_drain_timeout_contract_mismatch"));
    assert!(CHECKLIST.contains("shutdown_checkpoint_reconciliation_timeout_reason_code_mismatch"));
    assert!(CHECKLIST.contains("shutdown_checkpoint_reconciliation_graceful_checkpoint_mismatch"));
    assert!(CHECKLIST.contains("runtime_shutdown_invariant_violation"));
    assert!(CHECKLIST.contains("Regression: #4333"));
}

#[test]
fn checklist_contains_runtime_observability_endpoint_payload_checker_gate() {
    assert!(
        CHECKLIST.contains("## Runtime Observability Endpoint Payload Checker Gate (Issue #4328)")
    );
    assert!(CHECKLIST.contains(
        "main_tests::observability_endpoint_tests::spec_c01_observability_endpoint_contract_checker_accepts_valid_surface_payloads -- --exact"
    ));
    assert!(CHECKLIST.contains(
        "main_tests::observability_endpoint_tests::spec_c05_observability_endpoint_contract_checker_fails_closed_with_stable_reason_markers -- --exact"
    ));
    assert!(CHECKLIST.contains(
        "reason_taxonomy_version=kamn.runtime.observability-endpoint-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "reason_codes_csv=runtime_observability_policy_required_field_missing,runtime_observability_policy_schema_drift"
    ));
    assert!(CHECKLIST.contains("schema_version=kamn.runtime.observability.endpoint-fail-closed.v1"));
    assert!(CHECKLIST.contains("status=fail_closed"));
    assert!(CHECKLIST.contains("final_decision=NO-GO"));
    assert!(
        CHECKLIST.contains("runtime_observability_policy_schema_drift:<surface>.schema_version")
    );
    assert!(CHECKLIST.contains("Regression: #4328"));
}

#[test]
fn checklist_contains_failover_drift_taxonomy_runbook_parity_gate() {
    assert!(CHECKLIST.contains("## Failover + Sync Drill Evidence Contract (Issues #787, #788)"));
    assert!(CHECKLIST.contains(
        "failover_sync_drill_preflight_contract_lane_contract.sh check-policy --report-file /tmp/failover-sync-preflight-report.json --runbook-file docs/deploy/kolme_devnet_ops.md --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/failover-sync-preflight-policy.json"
    ));
    assert!(CHECKLIST.contains(
        "failover_sync_drill_preflight_contract_lane_contract.sh check-evidence-convergence --report-file /tmp/failover-sync-preflight-report.json --policy-file /tmp/failover-sync-preflight-policy.json --output-json /tmp/failover-sync-preflight-convergence.json"
    ));
    assert!(CHECKLIST.contains("drift_taxonomy_mapping_status=verified"));
    assert!(CHECKLIST.contains("runbook_marker_parity_status=verified"));
    assert!(CHECKLIST.contains(
        "drift_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.failover-drift-taxonomy-runbook-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "drift_taxonomy_runbook_reason_codes_csv=drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
    ));
    assert!(CHECKLIST.contains("drift_taxonomy_mapping_drift_detected"));
    assert!(CHECKLIST.contains("runbook_marker_parity_mismatch"));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_taxonomy_version=kamn.runtime.failover-promotion-decision-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_codes_csv=failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded,drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,failover_sync_drift_policy_expected_decision_mismatch,failover_sync_drift_policy_violation"
    ));
    assert!(CHECKLIST.contains("promotion_decision_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("evidence_convergence_status=verified"));
    assert!(CHECKLIST.contains("promotion_decision_reason_mapping_status=verified"));
    assert!(CHECKLIST.contains(
        "reason_taxonomy_version=kamn.runtime.failover-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "reason_codes_csv=failover_evidence_link_missing,failover_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch"
    ));
    assert!(CHECKLIST.contains("failover_evidence_link_missing:report_file"));
    assert!(CHECKLIST.contains("failover_evidence_payload_tamper_detected:<field>"));
    assert!(CHECKLIST.contains("promotion_decision_reason_mapping_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4287"));
    assert!(CHECKLIST.contains("Regression: #4288"));
    assert!(CHECKLIST.contains("Regression: #4289"));
    assert!(CHECKLIST.contains("Regression: #4290"));
}

#[test]
fn checklist_contains_unified_api_observability_payload_taxonomy_gate_markers() {
    assert!(CHECKLIST.contains("## Unified API-Observability Payload Taxonomy Gate (Issue #4507)"));
    assert!(CHECKLIST.contains(
        "check_unified_api_observability_local_heavy_live_policy.sh --report-file /tmp/unified-api-observability-local-heavy-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/unified-api-observability-local-heavy-policy.json"
    ));
    assert!(CHECKLIST.contains(
        "reason_taxonomy_version=kamn.runtime.unified-api-observability-local-heavy-policy-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "unified_api_observability_local_heavy_policy_correlation_schema_version_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "unified_api_observability_local_heavy_policy_correlation_required_fields_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "unified_api_observability_local_heavy_policy_correlation_id_propagation_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "correlation_schema_version=kamn.runtime.unified-api-observability-correlation-schema.v1"
    ));
    assert!(CHECKLIST.contains(
        "correlation_required_fields_csv=correlation_id,trace_id,trace_parent,span_id,request_id"
    ));
    assert!(CHECKLIST.contains("correlation_id_propagation_status=verified"));
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
    assert!(CHECKLIST.contains("tls_evidence_reason_codes_value=none|<csv>"));
    assert!(CHECKLIST.contains(
        "generator and policy-checker command output must both project this marker set."
    ));
    assert!(CHECKLIST
        .contains("invalid TLS evidence JSON must reject with `gonogo_tls_evidence_invalid_json`"));
    assert!(CHECKLIST.contains("tls_evidence_gate_final_decision=GO|NO-GO"));
    assert!(CHECKLIST.contains("Regression: #4298"));
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
fn checklist_contains_journal_append_checkpoint_integrity_gate() {
    assert!(CHECKLIST.contains(
        "## Journal Append/Checkpoint Integrity Determinism Gate (Issues #4236, #4240, #4241)"
    ));
    assert!(CHECKLIST.contains("test_check_sqlite_crash_recovery_live_policy.sh"));
    assert!(CHECKLIST.contains("test_validate_sqlite_crash_recovery_live.sh"));
    assert!(CHECKLIST.contains("test_validate_sqlite_crash_recovery_live_contract_lane.sh"));
    assert!(CHECKLIST.contains("append_checkpoint_integrity_status=verified"));
    assert!(CHECKLIST.contains(
        "append_checkpoint_reason_taxonomy_version=kamn.runtime.append-checkpoint-integrity-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "append_checkpoint_reason_codes_csv=wal_append_marker_missing,wal_checkpoint_marker_missing,append_checkpoint_marker_parity_mismatch"
    ));
    assert!(CHECKLIST.contains("sqlite_crash_recovery_policy_append_checkpoint_parity_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4240"));
    assert!(CHECKLIST.contains("Regression: #4241"));
}

#[test]
fn checklist_contains_replay_idempotency_taxonomy_runbook_parity_gate() {
    assert!(CHECKLIST.contains(
        "## Replay Idempotency Taxonomy/Runbook Parity Gate (Issues #4237, #4242, #4243)"
    ));
    assert!(CHECKLIST.contains(
        "check_sqlite_crash_recovery_live_policy.sh --report-file /tmp/sqlite-crash-recovery-live-report.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/sqlite-crash-recovery-live-policy-report.json"
    ));
    assert!(CHECKLIST.contains("replay_idempotency_taxonomy_mapping_status=verified"));
    assert!(CHECKLIST.contains("runbook_marker_parity_status=verified"));
    assert!(CHECKLIST.contains(
        "replay_idempotency_runbook_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-replay-idempotency-runbook-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "replay_idempotency_runbook_reason_codes_csv=replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
    ));
    assert!(CHECKLIST.contains("replay_idempotency_runbook_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("replay_idempotency_taxonomy_mapping_drift_detected"));
    assert!(CHECKLIST.contains("runbook_marker_parity_mismatch"));
    assert!(CHECKLIST.contains(
        "sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_taxonomy_version_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "sqlite_crash_recovery_policy_replay_idempotency_runbook_reason_codes_csv_mismatch"
    ));
    assert!(CHECKLIST.contains("Regression: #4242"));
    assert!(CHECKLIST.contains("Regression: #4243"));
}

#[test]
fn checklist_contains_crash_replay_evidence_convergence_and_mapping_gate() {
    assert!(CHECKLIST.contains(
        "## Crash-Replay Evidence Convergence/Promotion Reason Mapping Gate (Issues #4238, #4244, #4245)"
    ));
    assert!(CHECKLIST.contains("test_check_sqlite_crash_recovery_live_evidence_convergence.sh"));
    assert!(CHECKLIST.contains(
        "check_sqlite_crash_recovery_live_evidence_convergence.sh --report-file /tmp/sqlite-crash-recovery-live-contract-lane-report.json --policy-file /tmp/sqlite-crash-recovery-live-policy-report.json --output-json /tmp/sqlite-crash-recovery-live-convergence-report.json"
    ));
    assert!(CHECKLIST.contains("sqlite_crash_replay_evidence_convergence_status=verified"));
    assert!(CHECKLIST.contains("promotion_decision_reason_mapping_status=verified"));
    assert!(CHECKLIST.contains(
        "sqlite_crash_replay_evidence_reason_taxonomy_version=kamn.runtime.sqlite-crash-replay-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "sqlite_crash_replay_evidence_reason_codes_csv=sqlite_crash_replay_evidence_link_missing,sqlite_crash_replay_evidence_payload_tamper_detected,sqlite_crash_replay_promotion_decision_reason_mapping_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-promotion-decision-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_codes_csv=sqlite_crash_recovery_policy_required_field_missing,sqlite_crash_recovery_policy_marker_missing,sqlite_crash_recovery_policy_reason_taxonomy_mismatch,sqlite_crash_recovery_policy_runtime_mode_contract_mismatch,replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,sqlite_crash_recovery_policy_expected_decision_mismatch,sqlite_crash_recovery_policy_violation"
    ));
    assert!(CHECKLIST.contains("promotion_decision_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("sqlite_crash_replay_evidence_link_missing:source_report_file"));
    assert!(CHECKLIST.contains("sqlite_crash_replay_evidence_payload_tamper_detected:<field>"));
    assert!(CHECKLIST.contains("sqlite_crash_replay_promotion_decision_reason_mapping_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4244"));
    assert!(CHECKLIST.contains("Regression: #4245"));
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
        "live_gonogo_reason_codes_csv=milestone_review_operator_runbook_missing,milestone_review_operator_runbook_markers_missing,milestone_review_deployment_preflight_summary_missing,milestone_review_deployment_preflight_summary_invalid_json,milestone_review_deployment_preflight_summary_schema_mismatch,milestone_review_deployment_preflight_summary_status_mismatch,milestone_review_deployment_preflight_scope_mismatch,milestone_review_deployment_preflight_policy_missing,milestone_review_deployment_preflight_policy_invalid_json,milestone_review_deployment_preflight_policy_schema_mismatch,milestone_review_deployment_preflight_policy_final_decision_mismatch,milestone_review_deployment_preflight_policy_rotation_reason_taxonomy_mismatch,milestone_review_deployment_preflight_policy_rotation_reason_codes_value_mismatch,milestone_review_live_node_validation_summary_missing,milestone_review_live_node_validation_summary_invalid_json,milestone_review_live_node_validation_summary_schema_mismatch,milestone_review_live_node_validation_summary_status_mismatch,milestone_review_live_node_validation_scope_mismatch,milestone_review_live_node_validation_runtime_provider_mismatch,milestone_review_live_node_validation_lineage_contract_mismatch,milestone_review_live_node_validation_artifact_paths_missing,milestone_review_live_node_validation_rollback_lineage_missing,milestone_review_live_node_validation_recovery_lineage_missing,milestone_review_live_node_validation_policy_missing,milestone_review_live_node_validation_policy_invalid_json,milestone_review_live_node_validation_policy_schema_mismatch,milestone_review_live_node_validation_policy_final_decision_mismatch,milestone_review_go_no_go_gate_report_missing,milestone_review_go_no_go_gate_report_invalid_json,milestone_review_go_no_go_gate_schema_mismatch,milestone_review_go_no_go_gate_status_mismatch,milestone_review_go_no_go_gate_final_decision_mismatch,milestone_review_go_no_go_gate_ci_local_boundary_contract_mismatch,milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch,milestone_review_go_no_go_gate_combined_transport_reason_codes_mismatch,milestone_review_go_no_go_gate_combined_kolme_runtime_reason_code_mismatch,milestone_review_go_no_go_gate_kolme_runtime_commit_failure_taxonomy_version_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_version_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_status_mismatch,milestone_review_go_no_go_gate_combined_lane_marker_contract_status_mismatch"
    ));
    assert!(CHECKLIST.contains(
        "deployment_safety_gate_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains("deployment_safety_gate_reason_codes_csv=none|<csv>"));
    assert!(CHECKLIST.contains("deployment_safety_gate_reason_codes_value=none|<csv>"));
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
fn checklist_contains_local_full_stack_harness_runbook_parity_gate() {
    assert!(CHECKLIST
        .contains("## Local Full-Stack Harness Taxonomy and Runbook Parity Gate (Issue #4198)"));
    assert!(CHECKLIST.contains("validate_local_full_stack_integration_live_contract_lane.sh"));
    assert!(CHECKLIST.contains(
        "check_local_full_stack_integration_live_policy.sh --report-file /tmp/local-full-stack-integration-report.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/local-full-stack-integration-policy.json"
    ));
    assert!(CHECKLIST.contains("local_full_stack_harness_runbook_marker_parity_status=verified"));
    assert!(CHECKLIST.contains(
        "local_full_stack_harness_runbook_reason_taxonomy_version=kamn.runtime.local-full-stack-harness-runbook-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "local_full_stack_harness_runbook_reason_codes_csv=local_full_stack_harness_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
    ));
    assert!(CHECKLIST.contains("local_full_stack_harness_runbook_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("local_full_stack_harness_taxonomy_mapping_drift_detected"));
    assert!(CHECKLIST.contains("runbook_marker_parity_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4197"));
    assert!(CHECKLIST.contains("Regression: #4198"));
}

#[test]
fn checklist_contains_gonogo_promotion_convergence_reason_mapping_gate() {
    assert!(CHECKLIST
        .contains("## Go/No-Go Promotion Evidence Convergence Reason Mapping Gate (Issue #4200)"));
    assert!(CHECKLIST.contains(
        "run_go_no_go_gate_lane.sh --mode dry-run --max-seconds 120 --output-json /tmp/go-no-go-gate-report.json"
    ));
    assert!(CHECKLIST.contains("promotion_evidence_convergence_status=verified"));
    assert!(CHECKLIST.contains(
        "promotion_evidence_reason_taxonomy_version=kamn.runtime.go-no-go-gate-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "promotion_evidence_reason_codes_csv=promotion_evidence_link_missing,promotion_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch"
    ));
    assert!(CHECKLIST.contains("promotion_evidence_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("promotion_decision_reason_mapping_status=verified"));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_taxonomy_version=kamn.runtime.go-no-go-gate-promotion-decision-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "promotion_decision_reason_codes_csv=release_manifest_missing_required_artifact,release_manifest_success_marker_mismatch,gate_required_artifact_status_mismatch,gate_decision_fault_injection_triggered,runtime_budget_exceeded,gate_policy_unknown_reason_code,gate_policy_native_libp2p_provider_marker_mismatch,gate_policy_libp2p_fallback_marker_blocklist_mismatch,gate_policy_libp2p_fallback_markers_detected,gate_policy_native_libp2p_provider_marker_contract_status_mismatch"
    ));
    assert!(CHECKLIST.contains("promotion_decision_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("release_manifest_missing_required_artifact"));
    assert!(CHECKLIST.contains("release_manifest_success_marker_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4200"));
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
    assert!(CHECKLIST.contains("check_upgrade_compatibility_marker_matrix_policy.py"));
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
    assert!(CHECKLIST.contains(
        "provider_failure_reason_taxonomy_version=kamn.kolme.local-runtime-commit-provider-failure-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "reason_taxonomy_version=kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "reason_codes_csv=version_report_missing,fork_policy_report_missing,version_report_schema_mismatch,version_report_reason_taxonomy_mismatch,version_report_reason_codes_csv_mismatch,version_report_rehearsal_bypass_guard_status_mismatch,version_report_rehearsal_output_normalization_status_mismatch,fork_policy_report_schema_mismatch,fork_policy_report_reason_taxonomy_mismatch,fork_policy_report_reason_codes_csv_mismatch,fork_policy_report_rehearsal_bypass_guard_status_mismatch,fork_policy_report_rehearsal_output_normalization_status_mismatch,expected_final_decision_mismatch,ci_fast_gate_failed"
    ));
    assert!(CHECKLIST.contains("upgrade_compatibility_runbook_marker_parity_status=verified"));
    assert!(CHECKLIST.contains(
        "upgrade_compatibility_runbook_reason_taxonomy_version=kamn.kolme.upgrade-compatibility-runbook-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "upgrade_compatibility_runbook_reason_codes_csv=upgrade_compatibility_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
    ));
    assert!(CHECKLIST.contains("upgrade_compatibility_runbook_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("upgrade_compatibility_taxonomy_mapping_drift_detected"));
    assert!(CHECKLIST.contains("runbook_marker_parity_mismatch"));
    assert!(CHECKLIST.contains("version_report_schema_mismatch"));
    assert!(CHECKLIST.contains("fork_policy_report_rehearsal_bypass_guard_status_mismatch"));
    assert!(CHECKLIST.contains("expected_final_decision_mismatch"));
    assert!(CHECKLIST.contains("ci_fast_gate_failed"));
    assert!(CHECKLIST.contains("provider_failure_reason_codes_csv=provider_client_contract_mismatch,provider_contract_enforcement_mode_mismatch,provider_live_contract_marker_mismatch,provider_live_contract_marker_missing,provider_in_memory_reference_detected,provider_hint_in_memory_provider_reference_detected,provider_submit_profile_contract_mismatch,provider_command_marker_mismatch,provider_command_marker_missing,provider_signing_profile_marker_mismatch,provider_signing_profile_marker_missing,provider_signing_profile_simulated_detected,provider_signer_adapter_contract_mismatch,provider_signing_curve_contract_mismatch,provider_signing_profile_contract_version_mismatch,live_command_in_memory_provider_reference_detected"));
    assert!(CHECKLIST.contains("request_payload_evidence_artifact_path_lineage_mismatch"));
    assert!(CHECKLIST.contains("submit_evidence_artifact_path_lineage_mismatch"));
    assert!(CHECKLIST.contains("finality_evidence_artifact_path_lineage_mismatch"));
    assert!(CHECKLIST.contains("runtime_signing_profile_contract_version=v1"));
    assert!(CHECKLIST.contains("runtime_signing_profile=kolme-fork-secp256k1-v1"));
    assert!(CHECKLIST.contains(
        "native_signer_reason_taxonomy_version=kamn.kolme.local-signed-to-kolme-demo-native-signer-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "native_signer_reason_codes_csv=runtime_commit_native_signing_profile_marker_missing,runtime_commit_simulated_signing_profile_detected,runtime_signing_profile_missing,runtime_signing_profile_mismatch"
    ));
    assert!(CHECKLIST.contains("Regression: #4372"));
    assert!(CHECKLIST.contains("Regression: #4373"));
    assert!(CHECKLIST.contains("Regression: #4378"));
    assert!(CHECKLIST.contains("Regression: #4380"));
    assert!(CHECKLIST.contains("Regression: #4180"));
    assert!(CHECKLIST.contains("Regression: #4181"));
    assert!(CHECKLIST.contains("Regression: #4182"));
    assert!(CHECKLIST.contains("Regression: #4183"));
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
fn checklist_contains_fork_choice_finality_taxonomy_runbook_parity_gate() {
    assert!(CHECKLIST.contains(
        "## Fork-Choice Finality Taxonomy and Runbook Marker Parity Gate (Issues #4252, #4257, #4258)"
    ));
    assert!(CHECKLIST.contains(
        "check_libp2p_convergence_process_isolated_live_policy.sh --report-file /tmp/libp2p-convergence-process-isolated-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/libp2p-convergence-process-isolated-live-policy.json"
    ));
    assert!(
        CHECKLIST.contains("validate_libp2p_convergence_process_isolated_live_contract_lane.sh")
    );
    assert!(CHECKLIST.contains("finality_taxonomy_mapping_status=verified"));
    assert!(CHECKLIST.contains("runbook_marker_parity_status=verified"));
    assert!(CHECKLIST.contains(
        "convergence_reason_taxonomy_version=kamn.runtime.libp2p-convergence-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains("convergence_reason_codes_csv=fork_choice_stale_block_height"));
    assert!(CHECKLIST.contains(
        "finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1"
    ));
    assert!(CHECKLIST.contains(
        "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch"
    ));
    assert!(CHECKLIST.contains("finality_taxonomy_runbook_reason_code=none|<reason>"));
    assert!(CHECKLIST.contains("finality_taxonomy_mapping_drift_detected"));
    assert!(CHECKLIST.contains("runbook_marker_parity_mismatch"));
    assert!(CHECKLIST.contains("Regression: #4257"));
    assert!(CHECKLIST.contains("Regression: #4258"));
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
