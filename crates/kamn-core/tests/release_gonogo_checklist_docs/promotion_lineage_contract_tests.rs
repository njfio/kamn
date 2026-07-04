use super::support::assert_checklist_contains_all;

const CHECKLIST_CONTAINS_SLO_THRESHOLD_POLICY_GATE_CONVERGENCE_MARKERS: &[&str] = &[
    "## SLO Threshold/Policy Gate Convergence Gate (Issue #4468)",
    "--slo-policy-report-file /tmp/deployment-slo-rollback-report.json",
    "--slo-policy-max-age-seconds 1800",
    "slo_policy_reason_taxonomy_version=kamn.release.gonogo-slo-threshold-convergence-reason-taxonomy.v1",
    "slo_policy_reason_codes_csv=gonogo_slo_policy_file_missing,gonogo_slo_policy_invalid_json,gonogo_slo_policy_schema_mismatch,gonogo_slo_policy_status_not_pass,gonogo_slo_policy_final_decision_not_go,gonogo_slo_policy_reason_key_mismatch,gonogo_slo_policy_reason_codes_not_empty,gonogo_slo_policy_freshness_window_exceeded",
    "slo_policy_gate_final_decision=GO|NO-GO",
    "Regression: #4468",
];

#[test]
fn checklist_contains_slo_threshold_policy_gate_convergence() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_SLO_THRESHOLD_POLICY_GATE_CONVERGENCE_MARKERS,
        "checklist_contains_slo_threshold_policy_gate_convergence",
    );
}

const CHECKLIST_CONTAINS_LIVE_RUN_MODE_REHEARSAL_LINEAGE_GATE_MARKERS: &[&str] = &[
    "## Live Run-Mode Rehearsal Lineage Gate (Issue #3245)",
    "run_local_live_node_validation_bundle_lane.sh",
    "check_local_live_node_validation_bundle_policy.py",
    "run_local_live_node_validation_bundle_contract_lane.sh",
    "contracts.live_run_rehearsal_lineage_required=true",
    "run_mode_check_status_mismatch",
    "Regression: #3245",
];

#[test]
fn checklist_contains_live_run_mode_rehearsal_lineage_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_LIVE_RUN_MODE_REHEARSAL_LINEAGE_GATE_MARKERS,
        "checklist_contains_live_run_mode_rehearsal_lineage_gate",
    );
}

const CHECKLIST_CONTAINS_MILESTONE_REVIEW_AGGREGATE_LINEAGE_GATE_MARKERS: &[&str] = &[
    "## Milestone Review Aggregate Lineage Gate (Issue #3247)",
    "--deployment-preflight-summary-file",
    "--deployment-preflight-policy-file",
    "--live-node-validation-summary-file",
    "--live-node-validation-policy-file",
    "--go-no-go-gate-report-file",
    "python3 scripts/deploy/check_upgrade_rehearsal_lineage_policy.py --bundle-file /tmp/gonogo-milestone.json --expected-final-decision GO",
    "milestone_review_bundle",
    "schema_version=kamn.release.milestone-review-bundle.v1",
    "contracts.linked_artifact_lineage_required=true",
    "contracts.live_bundle_runtime_provider_client_required=KolmeRuntimeCommitLiveProvider",
    "contracts.go_no_go_gate_final_decision_required=GO",
    "upgrade_lineage_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1",
    "upgrade_lineage_reason_codes_csv=none|<csv>",
    "upgrade_lineage_reason_codes_value=none|<csv>",
    "promotion_gate_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1",
    "promotion_gate_reason_codes_csv=none|<csv>",
    "promotion_gate_reason_codes_value=none|<csv>",
    "milestone_review_go_no_go_gate_report_missing",
    "milestone_review_live_node_validation_runtime_provider_mismatch",
    "promotion gate reason mapping mismatch",
    "milestone review bundle lineage mismatch",
];

#[test]
fn checklist_contains_milestone_review_aggregate_lineage_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_MILESTONE_REVIEW_AGGREGATE_LINEAGE_GATE_MARKERS,
        "checklist_contains_milestone_review_aggregate_lineage_gate",
    );
}

const CHECKLIST_CONTAINS_LIVE_GONOGO_CONVERGENCE_BOUNDARY_GOVERNANCE_GATE_MARKERS: &[&str] = &[
    "## Live Go/No-Go Evidence Convergence and Boundary Governance Gate (Issue #4434)",
    "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract --max-seconds 120",
    "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900",
    "live_gonogo_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1",
    "live_gonogo_reason_codes_csv=milestone_review_operator_runbook_missing,milestone_review_operator_runbook_markers_missing,milestone_review_deployment_preflight_summary_missing,milestone_review_deployment_preflight_summary_invalid_json,milestone_review_deployment_preflight_summary_schema_mismatch,milestone_review_deployment_preflight_summary_status_mismatch,milestone_review_deployment_preflight_scope_mismatch,milestone_review_deployment_preflight_policy_missing,milestone_review_deployment_preflight_policy_invalid_json,milestone_review_deployment_preflight_policy_schema_mismatch,milestone_review_deployment_preflight_policy_final_decision_mismatch,milestone_review_deployment_preflight_policy_rotation_reason_taxonomy_mismatch,milestone_review_deployment_preflight_policy_rotation_reason_codes_value_mismatch,milestone_review_live_node_validation_summary_missing,milestone_review_live_node_validation_summary_invalid_json,milestone_review_live_node_validation_summary_schema_mismatch,milestone_review_live_node_validation_summary_status_mismatch,milestone_review_live_node_validation_scope_mismatch,milestone_review_live_node_validation_runtime_provider_mismatch,milestone_review_live_node_validation_lineage_contract_mismatch,milestone_review_live_node_validation_artifact_paths_missing,milestone_review_live_node_validation_rollback_lineage_missing,milestone_review_live_node_validation_recovery_lineage_missing,milestone_review_live_node_validation_policy_missing,milestone_review_live_node_validation_policy_invalid_json,milestone_review_live_node_validation_policy_schema_mismatch,milestone_review_live_node_validation_policy_final_decision_mismatch,milestone_review_go_no_go_gate_report_missing,milestone_review_go_no_go_gate_report_invalid_json,milestone_review_go_no_go_gate_schema_mismatch,milestone_review_go_no_go_gate_status_mismatch,milestone_review_go_no_go_gate_final_decision_mismatch,milestone_review_go_no_go_gate_ci_local_boundary_contract_mismatch,milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch,milestone_review_go_no_go_gate_combined_transport_reason_codes_mismatch,milestone_review_go_no_go_gate_combined_kolme_runtime_reason_code_mismatch,milestone_review_go_no_go_gate_kolme_runtime_commit_failure_taxonomy_version_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_version_mismatch,milestone_review_go_no_go_gate_kolme_fixture_profile_status_mismatch,milestone_review_go_no_go_gate_combined_lane_marker_contract_status_mismatch",
    "deployment_safety_gate_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1",
    "deployment_safety_gate_reason_codes_csv=none|<csv>",
    "deployment_safety_gate_reason_codes_value=none|<csv>",
    "live_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-live-boundary-reason-taxonomy.v1",
    "live_gonogo_boundary_reason_codes_csv=live_gonogo_ci_smoke_seconds_exceeded,live_gonogo_local_heavy_seconds_exceeded,live_gonogo_local_heavy_opt_in_missing,live_gonogo_evidence_convergence_mismatch",
    "live_gonogo_ci_smoke_max_seconds=120",
    "live_gonogo_local_heavy_max_seconds=900",
    "Regression: #4441",
    "Regression: #4442",
];

#[test]
fn checklist_contains_live_gonogo_convergence_boundary_governance_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_LIVE_GONOGO_CONVERGENCE_BOUNDARY_GOVERNANCE_GATE_MARKERS,
        "checklist_contains_live_gonogo_convergence_boundary_governance_gate",
    );
}

const CHECKLIST_CONTAINS_LOCAL_FULL_STACK_HARNESS_RUNBOOK_PARITY_GATE_MARKERS: &[&str] = &[
    "## Local Full-Stack Harness Taxonomy and Runbook Parity Gate (Issue #4198)",
    "validate_local_full_stack_integration_live_contract_lane.sh",
    "check_local_full_stack_integration_live_policy.sh --report-file /tmp/local-full-stack-integration-report.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/local-full-stack-integration-policy.json",
    "local_full_stack_harness_runbook_marker_parity_status=verified",
    "local_full_stack_harness_runbook_reason_taxonomy_version=kamn.runtime.local-full-stack-harness-runbook-reason-taxonomy.v1",
    "local_full_stack_harness_runbook_reason_codes_csv=local_full_stack_harness_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "local_full_stack_harness_runbook_reason_code=none|<reason>",
    "local_full_stack_harness_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "Regression: #4197",
    "Regression: #4198",
];

#[test]
fn checklist_contains_local_full_stack_harness_runbook_parity_gate() {
    assert_checklist_contains_all(
        CHECKLIST_CONTAINS_LOCAL_FULL_STACK_HARNESS_RUNBOOK_PARITY_GATE_MARKERS,
        "checklist_contains_local_full_stack_harness_runbook_parity_gate",
    );
}
