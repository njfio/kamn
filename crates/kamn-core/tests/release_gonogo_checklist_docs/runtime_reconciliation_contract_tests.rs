use super::support::assert_checklist_contains_all;

const CHECKLIST_CONTAINS_SERVICE_API_WEBSOCKET_SESSION_EVIDENCE_CONVERGENCE_GATE_MARKERS: &[&str] = &[
    "## Service API Websocket Session Evidence Convergence Gate (Issue #4268)",
    "check_service_api_websocket_live_evidence_convergence.sh --report-file /tmp/service-api-websocket-live-contract-lane-report.json --policy-file /tmp/service-api-websocket-live-policy-report.json --output-json /tmp/service-api-websocket-live-convergence-report.json",
    "service_api_websocket_evidence_convergence_status=verified",
    "promotion_decision_reason_mapping_status=verified",
    "service_api_websocket_evidence_reason_taxonomy_version=kamn.runtime.service-api-websocket-evidence-convergence-reason-taxonomy.v1",
    "service_api_websocket_evidence_reason_codes_csv=service_api_websocket_evidence_link_missing,service_api_websocket_evidence_payload_tamper_detected,service_api_websocket_promotion_decision_reason_mapping_mismatch",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.service-api-websocket-promotion-decision-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=service_api_websocket_policy_required_field_missing,service_api_websocket_policy_marker_missing,service_api_websocket_policy_reason_taxonomy_mismatch,service_api_websocket_policy_idle_timeout_contract_mismatch,ci_fast_gate_failed,service_api_websocket_policy_expected_decision_mismatch,service_api_websocket_policy_violation",
    "promotion_decision_reason_code=none|<reason>",
    "service_api_websocket_evidence_link_missing:source_report_file",
    "service_api_websocket_promotion_decision_reason_mapping_mismatch",
    "Regression: #4274",
    "Regression: #4275",
];

#[test]
fn checklist_contains_service_api_websocket_session_evidence_convergence_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_SERVICE_API_WEBSOCKET_SESSION_EVIDENCE_CONVERGENCE_GATE_MARKERS,
        "checklist_contains_service_api_websocket_session_evidence_convergence_gate",
    );
}

const CHECKLIST_CONTAINS_SERVICE_API_AXUM_ADMISSION_BACKPRESSURE_EVIDENCE_CONVERGENCE_GATE_MARKERS: &[&str] = &[
    "## Service API Axum Admission/Backpressure Evidence Convergence Gate (Issues #4223, #4229, #4230)",
    "check_service_api_axum_ingress_live_evidence_convergence.sh --report-file /tmp/service-api-axum-ingress-contract-lane-report.json --policy-file /tmp/service-api-axum-ingress-policy-report.json --output-json /tmp/service-api-axum-ingress-convergence-report.json",
    "test_check_service_api_axum_ingress_live_evidence_convergence.sh",
    "service_api_axum_evidence_convergence_status=verified",
    "promotion_decision_reason_mapping_status=verified",
    "service_api_axum_evidence_reason_taxonomy_version=kamn.runtime.service-api-axum-evidence-convergence-reason-taxonomy.v1",
    "service_api_axum_evidence_reason_codes_csv=service_api_axum_evidence_link_missing,service_api_axum_evidence_payload_tamper_detected,service_api_axum_promotion_decision_reason_mapping_mismatch",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation",
    "promotion_decision_reason_code=none|<reason>",
    "service_api_axum_evidence_link_missing:source_report_file",
    "service_api_axum_promotion_decision_reason_mapping_mismatch",
    "Regression: #4229",
    "Regression: #4230",
];

#[test]
fn checklist_contains_service_api_axum_admission_backpressure_evidence_convergence_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_SERVICE_API_AXUM_ADMISSION_BACKPRESSURE_EVIDENCE_CONVERGENCE_GATE_MARKERS, "checklist_contains_service_api_axum_admission_backpressure_evidence_convergence_gate");
}

const CHECKLIST_CONTAINS_BLOCK_RECONCILIATION_PARTITION_HEALING_MISMATCH_MAPPING_GATE_MARKERS: &[&str] = &[
    "## Block Reconciliation Partition-Healing Mismatch Mapping Gate (Issues #4251, #4255, #4256)",
    "test_check_block_reconciliation_partition_rejoin_live_policy.sh",
    "test_validate_block_reconciliation_partition_rejoin_live_contract_lane.sh",
    "partition_healing_mismatch_reason_mapping_status=verified",
    "partition_healing_mismatch_reason_taxonomy_version=kamn.runtime.block-reconciliation-partition-healing-mismatch-reason-taxonomy.v1",
    "partition_healing_mismatch_reason_codes_csv=block_reconciliation_partition_rejoin_policy_required_field_missing,block_reconciliation_partition_rejoin_policy_marker_mismatch,block_reconciliation_partition_rejoin_policy_transport_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_taxonomy_mismatch,block_reconciliation_partition_rejoin_policy_recovery_contract_mismatch,block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid,block_reconciliation_partition_rejoin_policy_lane_mode_contract_mismatch,block_reconciliation_partition_rejoin_policy_ci_fast_gate_failed,block_reconciliation_partition_rejoin_policy_expected_decision_mismatch,block_reconciliation_partition_rejoin_policy_violation",
    "partition_healing_mismatch_reason_code=none|<reason>",
    "block_reconciliation_partition_rejoin_policy_required_field_missing:<field>",
    "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_invalid",
    "block_reconciliation_partition_rejoin_policy_reconciliation_reason_codes_csv_mismatch",
    "block_reconciliation_partition_rejoin_policy_reconciliation_consistency_reason_taxonomy_version_mismatch",
    "block_reconciliation_partition_rejoin_policy_consistency_classification_status_mismatch",
    "Regression: #4255",
    "Regression: #4256",
];

#[test]
fn checklist_contains_block_reconciliation_partition_healing_mismatch_mapping_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_BLOCK_RECONCILIATION_PARTITION_HEALING_MISMATCH_MAPPING_GATE_MARKERS,
        "checklist_contains_block_reconciliation_partition_healing_mismatch_mapping_gate",
    );
}

const CHECKLIST_CONTAINS_FORK_CHOICE_FINALITY_EVIDENCE_CONVERGENCE_GATE_MARKERS: &[&str] = &[
    "## Fork-Choice Finality Evidence Convergence Gate (Issues #4253, #4259, #4260)",
    "check_libp2p_convergence_process_isolated_live_evidence_convergence.sh --report-file /tmp/libp2p-convergence-process-isolated-live-contract-lane-report.json --policy-file /tmp/libp2p-convergence-process-isolated-live-policy-report.json --output-json /tmp/libp2p-convergence-process-isolated-live-convergence-report.json",
    "test_check_libp2p_convergence_process_isolated_live_evidence_convergence.sh",
    "libp2p_finality_evidence_convergence_status=verified",
    "promotion_decision_reason_mapping_status=verified",
    "libp2p_finality_evidence_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1",
    "libp2p_finality_evidence_reason_codes_csv=libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation",
    "promotion_decision_reason_code=none|<reason>",
    "libp2p_finality_evidence_link_missing:source_report_file",
    "libp2p_finality_evidence_payload_tamper_detected:<field>",
    "libp2p_finality_promotion_decision_reason_mapping_mismatch",
    "Regression: #4259",
    "Regression: #4260",
];

#[test]
fn checklist_contains_fork_choice_finality_evidence_convergence_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_FORK_CHOICE_FINALITY_EVIDENCE_CONVERGENCE_GATE_MARKERS,
        "checklist_contains_fork_choice_finality_evidence_convergence_gate",
    );
}

const CHECKLIST_CONTAINS_SHUTDOWN_SIGNAL_LIFECYCLE_REASON_MAPPING_GATE_MARKERS: &[&str] = &[
    "## Shutdown Signal Lifecycle Reason Mapping Gate (Issue #4331)",
    "main_tests::runtime_tests::regression_full_supervisor_stop_contract_classifier_rejects_empty_or_non_numeric_signal_tick -- --exact",
    "main_tests::runtime_tests::regression_shutdown_policy_rejects_os_signal_hooks_for_non_daemon_modes -- --exact",
    "shutdown_signal_reason_taxonomy_version=kamn.runtime.shutdown-signal-lifecycle-reason-taxonomy.v1",
    "shutdown_signal_reason_codes_csv=full_supervisor_stop_invalid_shutdown_drain_status,full_supervisor_stop_invalid_shutdown_snapshot_flush_status,full_supervisor_stop_not_signaled_status_mismatch,full_supervisor_stop_not_signaled_snapshot_flush_mismatch,full_supervisor_stop_missing_signal_tick,full_supervisor_stop_missing_drain_ticks,full_supervisor_stop_missing_timeout_ticks,full_supervisor_stop_missing_ignored_signals,full_supervisor_stop_graceful_status_mismatch,full_supervisor_stop_graceful_snapshot_flush_status_mismatch,full_supervisor_stop_graceful_timeout_status_mismatch,full_supervisor_stop_graceful_timeout_snapshot_flush_status_mismatch,full_supervisor_stop_unknown_completion_reason",
    "shutdown_signal_reason_codes_value=none|<csv>",
    "shutdown_signal_hook_runtime_modes=daemon|full",
    "shutdown_signal_hook_explicit_override=--daemon-shutdown-os-signals",
    "full_supervisor_stop_missing_signal_tick",
    "Regression: #4331",
];

#[test]
fn checklist_contains_shutdown_signal_lifecycle_reason_mapping_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_SHUTDOWN_SIGNAL_LIFECYCLE_REASON_MAPPING_GATE_MARKERS,
        "checklist_contains_shutdown_signal_lifecycle_reason_mapping_gate",
    );
}

const CHECKLIST_CONTAINS_SHUTDOWN_DRAIN_CHECKPOINT_RECONCILIATION_GATE_MARKERS: &[&str] = &[
    "## Shutdown Drain/Checkpoint Reconciliation Gate (Issues #4332, #4333)",
    "shutdown_checkpoint_reconciliation_reason_taxonomy_version=kamn.runtime.shutdown-checkpoint-reconciliation-reason-taxonomy.v1",
    "full_supervisor_stop_graceful_drain_timeout_contract_mismatch",
    "shutdown_checkpoint_reconciliation_timeout_reason_code_mismatch",
    "shutdown_checkpoint_reconciliation_graceful_checkpoint_mismatch",
    "runtime_shutdown_invariant_violation",
    "Regression: #4333",
];

#[test]
fn checklist_contains_shutdown_drain_checkpoint_reconciliation_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_SHUTDOWN_DRAIN_CHECKPOINT_RECONCILIATION_GATE_MARKERS,
        "checklist_contains_shutdown_drain_checkpoint_reconciliation_gate",
    );
}

const CHECKLIST_CONTAINS_RUNTIME_OBSERVABILITY_ENDPOINT_PAYLOAD_CHECKER_GATE_MARKERS: &[&str] = &[
    "## Runtime Observability Endpoint Payload Checker Gate (Issue #4328)",
    "main_tests::observability_endpoint_tests::spec_c01_observability_endpoint_contract_checker_accepts_valid_surface_payloads -- --exact",
    "main_tests::observability_endpoint_tests::spec_c05_observability_endpoint_contract_checker_fails_closed_with_stable_reason_markers -- --exact",
    "reason_taxonomy_version=kamn.runtime.observability-endpoint-reason-taxonomy.v1",
    "reason_codes_csv=runtime_observability_policy_required_field_missing,runtime_observability_policy_schema_drift",
    "schema_version=kamn.runtime.observability.endpoint-fail-closed.v1",
    "status=fail_closed",
    "final_decision=NO-GO",
    "runtime_observability_policy_schema_drift:<surface>.schema_version",
    "Regression: #4328",
];

#[test]
fn checklist_contains_runtime_observability_endpoint_payload_checker_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_RUNTIME_OBSERVABILITY_ENDPOINT_PAYLOAD_CHECKER_GATE_MARKERS,
        "checklist_contains_runtime_observability_endpoint_payload_checker_gate",
    );
}

const CHECKLIST_CONTAINS_FAILOVER_DRIFT_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS: &[&str] = &[
    "## Failover + Sync Drill Evidence Contract (Issues #787, #788)",
    "failover_sync_drill_preflight_contract_lane_contract.sh check-policy --report-file /tmp/failover-sync-preflight-report.json --runbook-file docs/deploy/kolme_devnet_ops.md --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/failover-sync-preflight-policy.json",
    "failover_sync_drill_preflight_contract_lane_contract.sh check-evidence-convergence --report-file /tmp/failover-sync-preflight-report.json --policy-file /tmp/failover-sync-preflight-policy.json --output-json /tmp/failover-sync-preflight-convergence.json",
    "drift_taxonomy_mapping_status=verified",
    "runbook_marker_parity_status=verified",
    "drift_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.failover-drift-taxonomy-runbook-reason-taxonomy.v1",
    "drift_taxonomy_runbook_reason_codes_csv=drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "drift_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.failover-promotion-decision-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=failover_readiness_progress_stalled,live_node_drift_marker_parity_mismatch,ci_local_promotion_budget_boundary_exceeded,drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,failover_sync_drift_policy_expected_decision_mismatch,failover_sync_drift_policy_violation",
    "promotion_decision_reason_code=none|<reason>",
    "evidence_convergence_status=verified",
    "promotion_decision_reason_mapping_status=verified",
    "reason_taxonomy_version=kamn.runtime.failover-evidence-convergence-reason-taxonomy.v1",
    "reason_codes_csv=failover_evidence_link_missing,failover_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch",
    "failover_evidence_link_missing:report_file",
    "failover_evidence_payload_tamper_detected:<field>",
    "promotion_decision_reason_mapping_mismatch",
    "Regression: #4287",
    "Regression: #4288",
    "Regression: #4289",
    "Regression: #4290",
];

#[test]
fn checklist_contains_failover_drift_taxonomy_runbook_parity_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_FAILOVER_DRIFT_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS,
        "checklist_contains_failover_drift_taxonomy_runbook_parity_gate",
    );
}
