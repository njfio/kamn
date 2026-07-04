use super::support::assert_checklist_contains_all;

const CHECKLIST_CONTAINS_GONOGO_PROMOTION_CONVERGENCE_REASON_MAPPING_GATE_MARKERS: &[&str] = &[
    "## Go/No-Go Promotion Evidence Convergence Reason Mapping Gate (Issue #4200)",
    "run_go_no_go_gate_lane.sh --mode dry-run --max-seconds 120 --output-json /tmp/go-no-go-gate-report.json",
    "promotion_evidence_convergence_status=verified",
    "promotion_evidence_reason_taxonomy_version=kamn.runtime.go-no-go-gate-evidence-convergence-reason-taxonomy.v1",
    "promotion_evidence_reason_codes_csv=promotion_evidence_link_missing,promotion_evidence_payload_tamper_detected,promotion_decision_reason_mapping_mismatch",
    "promotion_evidence_reason_code=none|<reason>",
    "promotion_decision_reason_mapping_status=verified",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.go-no-go-gate-promotion-decision-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=release_manifest_missing_required_artifact,release_manifest_success_marker_mismatch,gate_required_artifact_status_mismatch,gate_decision_fault_injection_triggered,runtime_budget_exceeded,gate_policy_unknown_reason_code,gate_policy_native_libp2p_provider_marker_mismatch,gate_policy_libp2p_fallback_marker_blocklist_mismatch,gate_policy_libp2p_fallback_markers_detected,gate_policy_native_libp2p_provider_marker_contract_status_mismatch",
    "promotion_decision_reason_code=none|<reason>",
    "go_no_go_required_artifact_ids_csv=go_no_go_evidence,rollback_readiness,dr_readiness,local_full_stack_integration,local_full_runtime_convergence,transport_fault_matrix,cross_store_replay_consistency",
    "cross_store_replay_consistency_status=dry_run_pending|verified",
    "cross_store_replay_consistency_policy_status=verified",
    "release_manifest_missing_required_artifact",
    "release_manifest_success_marker_mismatch",
    "Regression: #4200",
];

#[test]
fn checklist_contains_gonogo_promotion_convergence_reason_mapping_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_GONOGO_PROMOTION_CONVERGENCE_REASON_MAPPING_GATE_MARKERS,
        "checklist_contains_gonogo_promotion_convergence_reason_mapping_gate",
    );
}

const CHECKLIST_CONTAINS_STAGING_REHEARSAL_CONTRACT_MARKERS: &[&str] = &[
    "## Staging Deploy + Rollback Rehearsal Contract",
    "staging_rehearsal_contract.py",
    "generate_staging_rehearsal_bundle.sh",
    "check_staging_rehearsal_policy.sh",
    "run_staging_rehearsal_contract_lane.sh",
    "run_staging_rehearsal_deep_lane.sh",
    "kamn.release.staged-rehearsal-signoff.v1",
    "staged_rehearsal_signoff_status=verified|fail-closed",
    "--recovery-time-seconds",
    "--max-allowed-recovery-time-seconds",
    "mttr-threshold-exceeded",
    "mttr_within_bound",
    "Regression: #2337",
];

#[test]
fn checklist_contains_staging_rehearsal_contract() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_STAGING_REHEARSAL_CONTRACT_MARKERS,
        "checklist_contains_staging_rehearsal_contract",
    );
}

const CHECKLIST_CONTAINS_DURABLE_GUARD_RECOVERY_EVIDENCE_MARKERS: &[&str] = &[
    "## Durable Guard Migration + Recovery Matrix Evidence",
    "run_durable_guard_recovery_contract_lane.sh",
    "durable_guard_recovery_contract_lane_contract.py",
    "run_durable_guard_recovery_deep_lane.sh",
    "performance_durable_guard_recovery_contract_lane_budget",
    "performance_durable_guard_recovery_matrix_deep_lane",
    "performance_bundle_contract_lane_budget",
    "performance_bundle_store_deep_lane_stress",
];

#[test]
fn checklist_contains_durable_guard_recovery_evidence() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_DURABLE_GUARD_RECOVERY_EVIDENCE_MARKERS,
        "checklist_contains_durable_guard_recovery_evidence",
    );
}

const CHECKLIST_CONTAINS_PERSISTENCE_EVIDENCE_TAMPER_FRESHNESS_GATE_MARKERS: &[&str] = &[
    "## Persistence Evidence Tamper/Freshness Gate (Issue #4389)",
    "validate_persistence_adapters_live.sh",
    "persistence_gate_reason_taxonomy_version=kamn.runtime.persistence-gate-reason-taxonomy.v1",
    "persistence_gate_reason_codes_csv=content_storage_corrupt_payload_rejected,did_registry_corrupt_payload_rejected,task_operation_snapshot_schema_mismatch_rejected,durable_guard_snapshot_schema_mismatch_rejected,channel_snapshot_corrupt_payload_rejected,channel_snapshot_schema_mismatch_rejected,message_lifecycle_snapshot_corrupt_payload_rejected,message_lifecycle_snapshot_schema_mismatch_rejected,runtime_snapshot_corrupt_payload_rejected,runtime_snapshot_state_version_regression_rejected,persistence_evidence_tamper_detected,persistence_evidence_freshness_window_exceeded,persistence_evidence_incomplete,persistence_ci_smoke_local_heavy_boundary_violation",
    "persistence_tamper_freshness_drift_fail_closed_status=verified",
    "persistence_evidence_completeness_status=verified",
    "persistence_ci_smoke_local_heavy_boundary_status=verified",
    "persistence_ci_smoke_lane_cost_profile=low",
    "persistence_local_heavy_execution_mode=opt_in",
    "Regression: #4389",
];

#[test]
fn checklist_contains_persistence_evidence_tamper_freshness_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_PERSISTENCE_EVIDENCE_TAMPER_FRESHNESS_GATE_MARKERS,
        "checklist_contains_persistence_evidence_tamper_freshness_gate",
    );
}

const CHECKLIST_CONTAINS_SIGNER_INCIDENT_RECOVERY_CONTRACT_AND_CADENCE_MARKERS: &[&str] = &[
    "## Signer Incident Recovery Contract and Deep-Lane Cadence (Issue #989)",
    "run_signer_incident_recovery_lane.sh",
    "check_signer_incident_recovery_policy.sh",
    "run_signer_incident_recovery_contract_lane.sh",
    "run_signer_incident_recovery_deep_lane.sh",
    "kamn.signer.incident-recovery-report.v1",
    "kamn.signer.incident-recovery-deep-summary.v1",
    "signer_incident_recovery_reason_codes:GO:v1",
    "KAMN_SIGNER_INCIDENT_RECOVERY_DEEP_CADENCE",
];

#[test]
fn checklist_contains_signer_incident_recovery_contract_and_cadence() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_SIGNER_INCIDENT_RECOVERY_CONTRACT_AND_CADENCE_MARKERS,
        "checklist_contains_signer_incident_recovery_contract_and_cadence",
    );
}

const CHECKLIST_CONTAINS_SETTLEMENT_RECONCILIATION_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## Settlement Reconciliation Evidence Contract",
    "run_settlement_reconciliation_contract_lane.sh",
    "run_settlement_reconciliation_deep_lane.sh",
    "run_settlement_reconciliation_race_matrix.py",
    "fixtures/escrow_reconciliation/finality_race_cases.json",
    "--ledger-reference-id",
];

#[test]
fn checklist_contains_settlement_reconciliation_evidence_contract() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_SETTLEMENT_RECONCILIATION_EVIDENCE_CONTRACT_MARKERS,
        "checklist_contains_settlement_reconciliation_evidence_contract",
    );
}

const CHECKLIST_CONTAINS_SOC2_CONTROL_EVIDENCE_CONTRACT_MARKERS: &[&str] = &[
    "## SOC2 Control Evidence Contract",
    "generate_soc2_control_evidence_bundle.sh",
    "check_soc2_control_evidence_policy.sh",
    "run_soc2_control_evidence_contract_lane.sh",
    "run_soc2_control_evidence_deep_lane.sh",
    "run_soc2_control_evidence_replay_matrix.py",
    "fixtures/compliance_soc2/control_evidence_replay_cases.json",
];

#[test]
fn checklist_contains_soc2_control_evidence_contract() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_SOC2_CONTROL_EVIDENCE_CONTRACT_MARKERS,
        "checklist_contains_soc2_control_evidence_contract",
    );
}
