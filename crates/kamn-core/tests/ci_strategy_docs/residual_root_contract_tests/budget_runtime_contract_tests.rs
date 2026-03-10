use super::support::assert_doc_contains_all;

const IGNORED_TEST_AND_SCRIPT_BUDGET_MARKERS: &[&str] = &[
    "run_manifest_lane.sh --manifest scripts/framework/manifests/ci_ignored_test_and_script_budget_trend_contract_lane.json --phase contract --output-json /tmp/ignored-test-script-soft-budget-trend-contract-report.json",
    "test_run_ignored_test_and_script_budget_trend_contract_lane.sh",
    "ignored_test_metadata_stale_entry",
    "combined_shell_surface_shell_line_total_delta_fail_exceeded",
    "combined_shell_surface_ratio_fail_ceiling_exceeded",
    "ignored_test_script_budget_trend_contract_status=pass|fail",
];

const COMBINED_SHELL_SURFACE_REFRESH_MARKERS: &[&str] = &[
    "combined_shell_surface_baseline_refresh_trigger_reason=combined_shell_surface_shell_line_total_delta_fail_exceeded",
    "combined_shell_surface_baseline_refresh_command=bash scripts/ci/generate_combined_shell_surface_trend_report.sh --budget-file .ci/script-surface-budget.env --script-baseline-file .ci/script-surface-baseline.env --combined-baseline-file fixtures/ci/combined_shell_surface_trend_baseline.json --output-json /tmp/combined-shell-surface-trend-report.json",
    "combined_shell_surface_baseline_refresh_contract=update fixtures/ci/combined_shell_surface_trend_baseline.json with report.current metrics in the same PR",
    "combined_shell_surface_baseline_refresh_validation=bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh",
];

const TEST_HARNESS_BUDGET_MARKERS: &[&str] = &[
    "test_harness_loc_soft_budget_reason_taxonomy_version=kamn.ci.test-harness-loc-soft-budget-reason-taxonomy.v1",
    "test_harness_loc_soft_budget_reason_codes_csv=report_file_not_found,budget_file_not_found,baseline_file_not_found,trend_threshold_file_not_found,report_json_invalid,report_schema_mismatch,report_harness_script_count_invalid,report_harness_shell_line_total_invalid,budget_key_missing,budget_value_invalid,baseline_key_missing,baseline_value_invalid,trend_threshold_key_missing,trend_threshold_value_invalid,trend_threshold_order_invalid,harness_script_count_soft_max_exceeded,harness_shell_line_total_soft_max_exceeded,harness_script_count_trend_warn_delta_exceeded,harness_shell_line_total_trend_warn_delta_exceeded,harness_script_count_trend_fail_delta_exceeded,harness_shell_line_total_trend_fail_delta_exceeded,trend_fail_enforcement_triggered",
    "test_harness_loc_soft_budget_reason_codes_value=none|<csv>",
    "test_harness_loc_soft_budget_reason_class=stable|budgeted|violation",
    "test_harness_loc_soft_budget_ci_smoke_lane_cost_profile=low",
    "test_harness_loc_soft_budget_ci_smoke_runtime_budget_status=within|exceeded",
    "test_harness_loc_soft_budget_contract_ci_smoke_lane_cost_profile=low",
    "test_harness_loc_soft_budget_contract_ci_smoke_runtime_budget_status=within|exceeded",
    "test_harness_loc_soft_budget_contract_reason_key=test_harness_loc_soft_budget_contract_ok|test_harness_loc_soft_budget_contract_runtime_budget_exceeded",
];

const RUNTIME_LOCAL_FULL_MODE_MARKERS: &[&str] = &[
    "## Runtime Local Full-Mode Live Validation Contract Lane",
    "validate_local_full_runtime_live.sh --mode dry-run --output-json /tmp/local-full-runtime-live-summary.json",
    "check_local_full_runtime_live_policy.sh --report-file /tmp/local-full-runtime-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-full-runtime-live-policy.json",
    "validate_local_full_runtime_live_contract_lane.sh --output-json /tmp/local-full-runtime-live-contract-lane-report.json --policy-output-json /tmp/local-full-runtime-live-policy.json",
    "runtime_shutdown_gate_status=verified",
    "runtime_fallback_classification_status=verified",
    "runtime_error_reason_taxonomy_version=kamn.runtime.local-full-runtime-error-reason-taxonomy.v1",
    "runtime_error_reason_codes_csv=runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded",
    "ci_local_runtime_extraction_budget_boundary_status=verified",
    "runtime_full_shutdown_gate_drift_detected",
    "runtime_fallback_classification_unstable",
    "ci_local_runtime_extraction_budget_boundary_exceeded",
];

const RUNTIME_LOCAL_FULL_STACK_MARKERS: &[&str] = &[
    "local_heavy_runtime_budget_status",
    "runtime_phase_parity_reason_taxonomy_version=kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1",
    "runtime_phase_parity_reason_codes_csv=runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded",
    "runtime_phase_module_parity_status=verified",
    "runtime_extraction_evidence_output_status=verified",
    "ci_local_runtime_phase_parity_budget_boundary_status=verified",
    "elapsed_seconds",
    "max_seconds",
    "command_max_seconds",
    "local_full_stack_integration_policy_runtime_budget_status_mismatch",
    "local_full_stack_integration_policy_runtime_budget_exceeded",
    "runtime_phase_module_parity_drift_detected",
    "runtime_extraction_evidence_output_unstable",
    "ci_local_runtime_phase_parity_budget_boundary_exceeded",
    "runtime_module_boundary_parity_reason_taxonomy_version=kamn.runtime.module-boundary-parity-reason-taxonomy.v1",
    "runtime_module_boundary_parity_reason_codes_csv=runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded",
    "runtime_module_boundary_reason_codes_value=none|<csv>",
    "runtime_module_boundary_evidence_outputs_csv=runtime_module_boundary_parity_status,runtime_module_boundary_evidence_status,ci_local_runtime_module_boundary_budget_boundary_status",
    "runtime_orchestration_dispatch_boundary_status=verified",
    "runtime_daemon_phase_boundary_status=verified",
    "runtime_kolme_live_boundary_status=verified",
    "runtime_module_boundary_parity_status=verified",
    "runtime_module_boundary_evidence_status=verified",
    "ci_local_runtime_module_boundary_budget_boundary_status=verified",
    "ci_local_runtime_module_boundary_budget_boundary_exceeded",
];

#[test]
fn doc_contains_ignored_test_and_script_budget_trend_composed_contract_markers() {
    assert_doc_contains_all(
        IGNORED_TEST_AND_SCRIPT_BUDGET_MARKERS,
        "ignored test and script budget",
    );
}

#[test]
fn doc_contains_combined_shell_surface_baseline_refresh_workflow_markers() {
    assert_doc_contains_all(
        COMBINED_SHELL_SURFACE_REFRESH_MARKERS,
        "combined shell surface baseline refresh",
    );
}

#[test]
fn doc_contains_test_harness_structural_budget_reason_taxonomy_and_ci_smoke_markers() {
    assert_doc_contains_all(TEST_HARNESS_BUDGET_MARKERS, "test harness budget");
}

#[test]
fn doc_contains_runtime_local_full_mode_live_validation_runtime_error_taxonomy_markers() {
    assert_doc_contains_all(RUNTIME_LOCAL_FULL_MODE_MARKERS, "runtime local full mode");
}

#[test]
fn doc_contains_runtime_local_full_stack_runtime_budget_policy_markers() {
    assert_doc_contains_all(RUNTIME_LOCAL_FULL_STACK_MARKERS, "runtime local full stack");
}
