use super::super::docs_assert_support::{assert_deploy_contains_all};

const DEPLOY_COMPAT_CONTAINS_KOLME_UPGRADE_COMPATIBILITY_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Kolme Upgrade Compatibility Taxonomy and Runbook Marker Parity Contracts (Issues #4182, #4183)",
    "reason_taxonomy_version=kamn.kolme.upgrade-compatibility-marker-matrix-reason-taxonomy.v1",
    "reason_codes_csv=version_report_missing,fork_policy_report_missing,version_report_schema_mismatch,version_report_reason_taxonomy_mismatch,version_report_reason_codes_csv_mismatch,version_report_rehearsal_bypass_guard_status_mismatch,version_report_rehearsal_output_normalization_status_mismatch,fork_policy_report_schema_mismatch,fork_policy_report_reason_taxonomy_mismatch,fork_policy_report_reason_codes_csv_mismatch,fork_policy_report_rehearsal_bypass_guard_status_mismatch,fork_policy_report_rehearsal_output_normalization_status_mismatch,expected_final_decision_mismatch,ci_fast_gate_failed",
    "upgrade_compatibility_runbook_marker_parity_status=verified",
    "upgrade_compatibility_runbook_reason_taxonomy_version=kamn.kolme.upgrade-compatibility-runbook-reason-taxonomy.v1",
    "upgrade_compatibility_runbook_reason_codes_csv=upgrade_compatibility_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "upgrade_compatibility_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "python3 scripts/kolme/check_upgrade_compatibility_marker_matrix_policy.py --version-report-file /tmp/kolme-version-report.json --fork-policy-report-file /tmp/kolme-fork-compatibility-policy-report.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-upgrade-compatibility-marker-matrix-policy-report.json",
    "bash scripts/framework/run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_version_compatibility_contract_lane.json --phase contract",
    "bash scripts/kolme/test_run_version_compatibility_contract_lane.sh",
    "Regression: #4182",
    "Regression: #4183",
];

#[test]
fn deploy_compat_contains_kolme_upgrade_compatibility_taxonomy_runbook_parity_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_KOLME_UPGRADE_COMPATIBILITY_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_kolme_upgrade_compatibility_taxonomy_runbook_parity_markers");
}

const DEPLOY_COMPAT_CONTAINS_ROTATION_PREFLIGHT_QUORUM_PARITY_AND_CUSTODY_TAMPER_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Rotation Preflight Quorum Marker Parity and Custody Tamper Contracts (Issues #4169, #4170)",
    "rotation_preflight_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1",
    "rotation_preflight_reason_codes_csv=signer_key_source_contract_version_mismatch,signer_key_source_invalid,signer_key_source_production_managed_external_required,signer_quorum_minimum_not_met,signer_rotation_epoch_stale,signer_rotation_rehearsal_drift_detected,signer_rotation_promotion_stalled,fallback_signer_secret_present_violation,fallback_signer_secret_checkpoint_reason_mismatch,fallback_signer_secret_remediation_missing,quorum_evidence_missing,quorum_evidence_rotation_metadata_missing,quorum_evidence_rotation_metadata_invalid,runtime_signer_attestation_quorum_shortfall,runtime_signer_attestation_profile_not_approved,runtime_signer_drift_telemetry_missing,runtime_signer_drift_telemetry_rotation_delta_invalid,runtime_signer_drift_matrix_inputs_invalid,runtime_signer_drift_rotation_fail_threshold_exceeded,runtime_signer_drift_quorum_fail_threshold_exceeded,custody_continuity_bypass_detected",
    "rotation_preflight_reason_codes_value=none|<csv>",
    "custody_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-custody-reason-taxonomy.v1",
    "custody_reason_codes_csv=custody_evidence_missing,custody_evidence_sha256_invalid,custody_evidence_file_missing,quorum_evidence_custody_sha256_mismatch,custody_continuity_bypass_detected",
    "custody_reason_codes_value=none|<csv>",
    "quorum_evidence_approval_count_mismatch",
    "quorum_evidence_custody_sha256_mismatch",
    "custody_continuity_bypass_detected",
    "test_check_local_kolme_live_deployment_preflight_policy.sh",
    "check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-deployment-preflight-policy.json",
    "Regression: #4169",
    "Regression: #4170",
];

#[test]
fn deploy_compat_contains_rotation_preflight_quorum_parity_and_custody_tamper_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_ROTATION_PREFLIGHT_QUORUM_PARITY_AND_CUSTODY_TAMPER_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_rotation_preflight_quorum_parity_and_custody_tamper_markers");
}

const DEPLOY_COMPAT_CONTAINS_DEPLOYMENT_PREFLIGHT_CHECKER_OUTPUT_RUNBOOK_SYNC_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Deployment Preflight Fail-Closed Checker Output and Runbook Marker Synchronization (Issue #4151)",
    "deployment_preflight_marker_contract_status=verified|failed",
    "deployment_preflight_marker_contract_version=kamn.kolme.local-live-deployment-preflight-marker-contract.v1",
    "deployment_preflight_required_markers_csv=rotation_preflight_reason_taxonomy_version,rotation_preflight_reason_codes_csv,rotation_preflight_reason_codes_value,custody_reason_taxonomy_version,custody_reason_codes_csv,custody_reason_codes_value",
    "deployment_preflight_schema_parity_status=verified|failed",
    "deployment_preflight_schema_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-schema-parity-reason-taxonomy.v1",
    "deployment_preflight_schema_reason_codes_csv=deployment_preflight_required_marker_missing,deployment_preflight_schema_parity_mismatch,deployment_preflight_reason_taxonomy_version_mismatch,deployment_preflight_reason_codes_csv_mismatch,deployment_preflight_reason_codes_value_mismatch",
    "deployment_preflight_schema_reason_code=none|<reason>",
    "deployment_preflight_runbook_marker_parity_status=verified|failed",
    "deployment_preflight_runbook_reason_taxonomy_version=kamn.kolme.local-live-deployment-preflight-runbook-reason-taxonomy.v1",
    "deployment_preflight_runbook_reason_codes_csv=deployment_preflight_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "deployment_preflight_runbook_reason_code=none|<reason>",
    "deployment_preflight_required_marker_missing",
    "deployment_preflight_schema_parity_mismatch",
    "deployment_preflight_reason_taxonomy_version_mismatch",
    "deployment_preflight_reason_codes_value_mismatch",
    "deployment_preflight_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "test_check_local_kolme_live_deployment_preflight_policy.sh",
    "check_local_kolme_live_deployment_preflight_policy.py --report-file /tmp/kolme-local-live-deployment-preflight-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/kolme-local-live-deployment-preflight-policy.json",
    "Regression: #4151",
];

#[test]
fn deploy_compat_contains_deployment_preflight_checker_output_runbook_sync_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_DEPLOYMENT_PREFLIGHT_CHECKER_OUTPUT_RUNBOOK_SYNC_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_deployment_preflight_checker_output_runbook_sync_markers");
}

const DEPLOY_COMPAT_CONTAINS_LOCAL_FULL_STACK_HARNESS_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Local Full-Stack Harness Taxonomy and Runbook Marker Parity Contracts (Issue #4197)",
    "combined_reason_taxonomy_version=kamn.runtime.local-full-stack-integration-reason-taxonomy.v1",
    "runtime_phase_parity_reason_taxonomy_version=kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1",
    "runtime_phase_parity_reason_codes_csv=runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded",
    "runtime_module_boundary_parity_reason_taxonomy_version=kamn.runtime.module-boundary-parity-reason-taxonomy.v1",
    "runtime_module_boundary_parity_reason_codes_csv=runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded",
    "runtime_phase_module_parity_status=verified",
    "runtime_module_boundary_parity_status=verified",
    "local_full_stack_harness_runbook_marker_parity_status=verified",
    "local_full_stack_harness_runbook_reason_taxonomy_version=kamn.runtime.local-full-stack-harness-runbook-reason-taxonomy.v1",
    "local_full_stack_harness_runbook_reason_codes_csv=local_full_stack_harness_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "local_full_stack_harness_runbook_reason_code=none|<reason>",
    "local_full_stack_harness_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "check_local_full_stack_integration_live_policy.sh --report-file /tmp/local-full-stack-integration-report.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/local-full-stack-integration-policy.json",
    "test_check_local_full_stack_integration_live_policy.sh",
    "test_validate_local_full_stack_integration_live_contract_lane.sh",
    "Regression: #4197",
    "Regression: #4198",
];

#[test]
fn deploy_compat_contains_local_full_stack_harness_taxonomy_runbook_parity_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_LOCAL_FULL_STACK_HARNESS_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_local_full_stack_harness_taxonomy_runbook_parity_markers");
}

const DEPLOY_COMPAT_CONTAINS_DRIFT_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Drift Taxonomy and Runbook Marker Parity Contracts (Issue #4282)",
    "drift_taxonomy_mapping_status=verified",
    "runbook_marker_parity_status=verified",
    "drift_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.failover-drift-taxonomy-runbook-reason-taxonomy.v1",
    "drift_taxonomy_runbook_reason_codes_csv=drift_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "drift_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "failover_sync_drill_preflight_contract_lane_contract.sh check-policy",
    "Regression: #4287",
    "Regression: #4288",
];

#[test]
fn deploy_compat_contains_drift_taxonomy_runbook_parity_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_DRIFT_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_drift_taxonomy_runbook_parity_markers");
}

const DEPLOY_COMPAT_CONTAINS_SQLITE_REPLAY_IDEMPOTENCY_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Crash-Recovery Replay Idempotency Taxonomy and Runbook Marker Parity Contracts (Issue #4237)",
    "replay_idempotency_taxonomy_mapping_status=verified",
    "runbook_marker_parity_status=verified",
    "replay_idempotency_runbook_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-replay-idempotency-runbook-reason-taxonomy.v1",
    "replay_idempotency_runbook_reason_codes_csv=replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "replay_idempotency_runbook_reason_code=none|<reason>",
    "replay_idempotency_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "check_sqlite_crash_recovery_live_policy.sh --report-file /tmp/sqlite-crash-recovery-live-report.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/sqlite-crash-recovery-live-policy-report.json",
    "Regression: #4242",
    "Regression: #4243",
];

#[test]
fn deploy_compat_contains_sqlite_replay_idempotency_taxonomy_runbook_parity_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_SQLITE_REPLAY_IDEMPOTENCY_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_sqlite_replay_idempotency_taxonomy_runbook_parity_markers");
}

const DEPLOY_COMPAT_CONTAINS_SQLITE_CRASH_REPLAY_EVIDENCE_CONVERGENCE_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Crash-Replay Evidence Convergence and Promotion Reason Mapping Contracts (Issue #4238)",
    "sqlite_crash_replay_evidence_convergence_status=verified",
    "promotion_decision_reason_mapping_status=verified",
    "sqlite_crash_replay_evidence_reason_taxonomy_version=kamn.runtime.sqlite-crash-replay-evidence-convergence-reason-taxonomy.v1",
    "sqlite_crash_replay_evidence_reason_codes_csv=sqlite_crash_replay_evidence_link_missing,sqlite_crash_replay_evidence_payload_tamper_detected,sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.sqlite-crash-recovery-promotion-decision-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=sqlite_crash_recovery_policy_required_field_missing,sqlite_crash_recovery_policy_marker_missing,sqlite_crash_recovery_policy_reason_taxonomy_mismatch,sqlite_crash_recovery_policy_runtime_mode_contract_mismatch,replay_idempotency_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,sqlite_crash_recovery_policy_expected_decision_mismatch,sqlite_crash_recovery_policy_violation",
    "sqlite_crash_replay_evidence_link_missing:source_report_file",
    "sqlite_crash_replay_promotion_decision_reason_mapping_mismatch",
    "check_sqlite_crash_recovery_live_evidence_convergence.sh --report-file /tmp/sqlite-crash-recovery-live-contract-lane-report.json --policy-file /tmp/sqlite-crash-recovery-live-policy-report.json --output-json /tmp/sqlite-crash-recovery-live-convergence-report.json",
    "Regression: #4244",
    "Regression: #4245",
];

#[test]
fn deploy_compat_contains_sqlite_crash_replay_evidence_convergence_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_SQLITE_CRASH_REPLAY_EVIDENCE_CONVERGENCE_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_sqlite_crash_replay_evidence_convergence_markers");
}

const DEPLOY_COMPAT_CONTAINS_SQLITE_CRASH_RESTART_RUNBOOK_POLICY_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Crash-Restart Recovery Marker and Runbook Parity Contracts (Issue #4018)",
    "sqlite_crash_restart_recovery_marker_status=verified",
    "sqlite_crash_restart_runbook_marker_parity_status=verified",
    "sqlite_crash_restart_runbook_reason_taxonomy_version=kamn.runtime.sqlite-crash-restart-local-heavy-runbook-reason-taxonomy.v1",
    "sqlite_crash_restart_runbook_reason_codes_csv=sqlite_crash_restart_recovery_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "sqlite_crash_restart_runbook_reason_code=none|<reason>",
    "check_sqlite_crash_restart_local_heavy_policy.sh --report-file /tmp/sqlite-crash-restart-local-heavy-lane-report.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --strategy-doc docs/ci/strategy.md --output-json /tmp/sqlite-crash-restart-local-heavy-policy-report.json",
    "sqlite_crash_restart_recovery_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "Regression: #4018",
];

#[test]
fn deploy_compat_contains_sqlite_crash_restart_runbook_policy_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_SQLITE_CRASH_RESTART_RUNBOOK_POLICY_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_sqlite_crash_restart_runbook_policy_markers");
}
