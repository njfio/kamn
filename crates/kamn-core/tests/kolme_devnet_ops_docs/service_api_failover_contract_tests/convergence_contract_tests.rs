use super::super::shared_support::{assert_plan_contains_all};

const PLAN_CONTAINS_TRIADIC_SMOKE_CONTRACT_COMMANDS_PLAN_MARKERS: &[&str] = &[
    "run_triadic_devnet_smoke.sh",
    "validate_triadic_devnet_smoke.py",
    "run_triadic_devnet_smoke_contract_lane.sh",
];

#[test]
fn plan_contains_triadic_smoke_contract_commands() {
    assert_plan_contains_all(PLAN_CONTAINS_TRIADIC_SMOKE_CONTRACT_COMMANDS_PLAN_MARKERS, "plan_contains_triadic_smoke_contract_commands");
}

const PLAN_CONTAINS_FALLBACK_MARKER_RETIREMENT_MATRIX_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Fallback Marker Retirement Matrix (Issue #2526)",
    "fixtures/kolme_compatibility/fallback_signer_marker_matrix.json",
    "kamn.kolme.fallback-signer-marker-matrix.v1",
    "check_fallback_signer_marker_matrix_policy.py",
    "bash scripts/kolme/test_check_fallback_signer_marker_matrix_policy.sh",
    "remove-target",
];

#[test]
fn plan_contains_fallback_marker_retirement_matrix_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_FALLBACK_MARKER_RETIREMENT_MATRIX_CONTRACT_PLAN_MARKERS, "plan_contains_fallback_marker_retirement_matrix_contract");
}

const PLAN_CONTAINS_FAILOVER_SYNC_DRILL_LANE_POLICY_PLAN_MARKERS: &[&str] = &[
    "## Failover + Sync Drill Lane Policy",
    "select_failover_sync_drill_lane.sh",
    "run_failover_sync_drill_preflight_contract_lane.sh",
    "run_failover_sync_drill_deep_lane.sh",
    "run_failover_sync_drill_suite.sh",
    "kamn.runtime.failover-sync-drill-suite-report.v1",
    "failover_sync_drill_preflight_contract_lane_contract.sh check-policy --report-file /tmp/failover-sync-preflight-report.json --runbook-file docs/deploy/kolme_devnet_ops.md --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/failover-sync-preflight-policy.json",
    "failover_sync_drill_preflight_contract_lane_contract.sh check-evidence-convergence --report-file /tmp/failover-sync-preflight-report.json --policy-file /tmp/failover-sync-preflight-policy.json --output-json /tmp/failover-sync-preflight-convergence.json",
    "reason_taxonomy_version=kamn.runtime.failover-evidence-convergence-reason-taxonomy.v1",
    "reason_codes_csv=failover_evidence_link_missing,failover_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.failover-promotion-decision-reason-taxonomy.v1",
    "failover_evidence_link_missing:report_file",
    "promotion_decision_reason_mapping_mismatch",
    "Regression: #4289",
    "Regression: #4290",
];

#[test]
fn plan_contains_failover_sync_drill_lane_policy() {
    assert_plan_contains_all(PLAN_CONTAINS_FAILOVER_SYNC_DRILL_LANE_POLICY_PLAN_MARKERS, "plan_contains_failover_sync_drill_lane_policy");
}

const PLAN_CONTAINS_SERVICE_API_WEBSOCKET_EVIDENCE_CONVERGENCE_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Service API Websocket Session Evidence Convergence (Issue #4268)",
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
fn plan_contains_service_api_websocket_evidence_convergence_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_SERVICE_API_WEBSOCKET_EVIDENCE_CONVERGENCE_CONTRACT_PLAN_MARKERS, "plan_contains_service_api_websocket_evidence_convergence_contract");
}

const PLAN_CONTAINS_SERVICE_API_AXUM_ADMISSION_BACKPRESSURE_EVIDENCE_CONVERGENCE_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Service API Axum Admission/Backpressure Evidence Convergence (Issue #4223)",
    "check_service_api_axum_ingress_live_evidence_convergence.sh --report-file /tmp/service-api-axum-ingress-contract-lane-report.json --policy-file /tmp/service-api-axum-ingress-policy-report.json --output-json /tmp/service-api-axum-ingress-convergence-report.json",
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
fn plan_contains_service_api_axum_admission_backpressure_evidence_convergence_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_SERVICE_API_AXUM_ADMISSION_BACKPRESSURE_EVIDENCE_CONVERGENCE_CONTRACT_PLAN_MARKERS, "plan_contains_service_api_axum_admission_backpressure_evidence_convergence_contract");
}

const PLAN_CONTAINS_SQLITE_CRASH_REPLAY_EVIDENCE_CONVERGENCE_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Sqlite Crash-Replay Evidence Convergence and Promotion Reason Mapping Contract (Issue #4238)",
    "check_sqlite_crash_recovery_live_evidence_convergence.sh --report-file /tmp/sqlite-crash-recovery-live-contract-lane-report.json --policy-file /tmp/sqlite-crash-recovery-live-policy-report.json --output-json /tmp/sqlite-crash-recovery-live-convergence-report.json",
    "sqlite_crash_replay_evidence_convergence_status=verified",
    "promotion_decision_reason_mapping_status=verified",
    "sqlite_crash_replay_evidence_reason_taxonomy_version=kamn.runtime.sqlite-crash-replay-evidence-convergence-reason-taxonomy.v1",
    "sqlite_crash_replay_evidence_reason_codes_csv=sqlite_crash_replay_evidence_link_missing,sqlite_crash_replay_evidence_payload_tamper_detected,sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-promotion-decision-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=sqlite_crash_recovery_policy_required_field_missing,sqlite_crash_recovery_policy_marker_missing,sqlite_crash_recovery_policy_reason_taxonomy_mismatch,sqlite_crash_recovery_policy_runtime_mode_contract_mismatch,replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,sqlite_crash_recovery_policy_expected_decision_mismatch,sqlite_crash_recovery_policy_violation",
    "promotion_decision_reason_code=none|<reason>",
    "sqlite_crash_replay_evidence_link_missing:source_report_file",
    "sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    "Regression: #4244",
    "Regression: #4245",
];

#[test]
fn plan_contains_sqlite_crash_replay_evidence_convergence_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_SQLITE_CRASH_REPLAY_EVIDENCE_CONVERGENCE_CONTRACT_PLAN_MARKERS, "plan_contains_sqlite_crash_replay_evidence_convergence_contract");
}

const PLAN_CONTAINS_FORK_CHOICE_FINALITY_EVIDENCE_CONVERGENCE_CONTRACT_PLAN_MARKERS: &[&str] = &[
    "## Fork-Choice Finality Evidence Convergence and Promotion Reason Mapping Contract (Issue #4253)",
    "check_libp2p_convergence_process_isolated_live_evidence_convergence.sh --report-file /tmp/libp2p-convergence-process-isolated-live-contract-lane-report.json --policy-file /tmp/libp2p-convergence-process-isolated-live-policy.json --output-json /tmp/libp2p-convergence-process-isolated-live-convergence-report.json",
    "promotion_decision_reason_mapping_status=verified",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1",
    "libp2p_finality_evidence_convergence_status=verified",
    "libp2p_finality_evidence_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1",
    "libp2p_finality_evidence_reason_codes_csv=libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch",
    "libp2p_finality_evidence_link_missing:source_report_file",
    "libp2p_finality_promotion_decision_reason_mapping_mismatch",
    "Regression: #4259",
    "Regression: #4260",
];

#[test]
fn plan_contains_fork_choice_finality_evidence_convergence_contract() {
    assert_plan_contains_all(PLAN_CONTAINS_FORK_CHOICE_FINALITY_EVIDENCE_CONVERGENCE_CONTRACT_PLAN_MARKERS, "plan_contains_fork_choice_finality_evidence_convergence_contract");
}
