const DOC: &str = include_str!("../../../docs/ci/strategy.md");
const OPS_DOC: &str = include_str!("../../../docs/ops/configuration.md");
const FAIRNESS_FIXTURE: &str =
    include_str!("../../../fixtures/runtime/starvation_fairness_fixture_matrix.txt");
const DELETION_FIXTURE: &str =
    include_str!("../../../fixtures/runtime/deletion_proof_artifact_fixture_matrix.txt");
const FAIRNESS_POLICY_SOURCE: &str =
    include_str!("../../kamn-runtime-guards/src/fairness_policy.rs");
const FAIRNESS_POLICY_SHIM_SOURCE: &str = include_str!("../src/fairness_policy.rs");
const SERVICE_API_ENDPOINT_SOURCE: &str =
    include_str!("../../kamn-node/src/service_api_endpoint.rs");
const SERVICE_API_TENANT_ISOLATION_CONTRACT_SOURCE: &str =
    include_str!("../../../scripts/runtime/service_api_tenant_isolation_matrix_live_contract.py");
const API_VERSION_POLICY_CONTRACT_SOURCE: &str =
    include_str!("../../../scripts/runtime/api_version_policy_live_contract.py");
const REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_CONTRACT_SOURCE: &str =
    include_str!("../../../scripts/runtime/request_response_schema_compatibility_live_contract.py");
const OVERLOAD_RUNNER_SOURCE: &str =
    include_str!("../../../scripts/ci/run_daemon_os_signal_stress_matrix.sh");
const LOCAL_HEAVY_REDACTION_RUNNER_SOURCE: &str =
    include_str!("../../../scripts/runtime/local_heavy_redaction_validation_lane_contract.py");

const FAIRNESS_REASON_TAXONOMY_VERSION: &str = "kamn.runtime.fairness-policy-reason-taxonomy.v1";
const FAIRNESS_REASON_CODES_CSV: &str =
    "fairness_scope_unknown,fairness_window_non_positive,fairness_max_gap_non_positive,fairness_weighted_share_exceeds_gap";
const DELETION_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.deletion-proof-checker-reason-taxonomy.v1";
const DELETION_REASON_CODES_CSV: &str =
    "deletion_proof_subject_missing,deletion_proof_tombstone_missing,deletion_proof_status_invalid,deletion_proof_hash_mismatch";
const LOCAL_HEAVY_REDACTION_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.local-heavy-redaction-validation-reason-taxonomy.v1";
const LOCAL_HEAVY_REDACTION_REASON_CODES_CSV: &str =
    "local_heavy_redaction_sensitive_pattern_detected,local_heavy_redaction_runtime_budget_exceeded";
const LOCAL_HEAVY_REDACTION_POLICY_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.local-heavy-redaction-validation-policy-reason-taxonomy.v1";
const LOCAL_HEAVY_REDACTION_POLICY_REASON_CODES_CSV: &str =
    "redaction_policy_required_field_missing,redaction_policy_marker_mismatch,redaction_policy_reason_taxonomy_mismatch,redaction_policy_profile_contract_mismatch,redaction_policy_docs_marker_parity_mismatch,ci_fast_gate_failed,redaction_policy_expected_decision_mismatch,redaction_policy_violation";
const OVERLOAD_REASON_TAXONOMY_VERSION: &str =
    "kamn.ci.daemon-os-signal-stress-matrix-reason-taxonomy.v1";
const OVERLOAD_REASON_CODES_CSV: &str = "runtime_budget_exceeded,matrix_failure_threshold_exceeded,quarantine_registry_missing,quarantine_reference_present_without_followup,matrix_failures_within_threshold,stable_success_with_quarantine_followup,stable_success";
const PERFORMANCE_CI_SMOKE_REASON_TAXONOMY_VERSION: &str =
    "kamn.ci.performance-ci-smoke-threshold-reason-taxonomy.v1";
const PERFORMANCE_CI_SMOKE_REASON_CODES_CSV: &str =
    "performance_ci_smoke_argument_invalid,performance_ci_smoke_threshold_contract_violation,performance_ci_smoke_report_contract_violation,performance_ci_smoke_latency_p50_threshold_exceeded,performance_ci_smoke_latency_p99_threshold_exceeded,performance_ci_smoke_throughput_threshold_below_minimum,performance_ci_smoke_availability_threshold_below_minimum,performance_ci_smoke_selector_missing_checker_entry,performance_ci_smoke_selector_forbidden_entry_present,performance_ci_smoke_workflow_missing_checker_step,performance_ci_smoke_workflow_forbidden_entry_present,performance_ci_smoke_docs_marker_parity_drift,performance_ci_smoke_docs_remediation_marker_missing,performance_ci_smoke_runtime_budget_exceeded";
const SERVICE_API_REQUEST_PATH_AUTHZ_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.service-api-auth-reason-taxonomy.v1";
const SERVICE_API_REQUEST_PATH_AUTHZ_REASON_CODES_CSV: &str = "service_api_auth_sender_did_header_missing,service_api_auth_sender_did_invalid,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_did_key_binding_invalid,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected";
const SERVICE_API_REQUEST_PATH_AUTHZ_PUBLIC_ROUTES_CSV: &str = "GET:/healthz,GET:/metrics";
const SERVICE_API_REQUEST_PATH_AUTHZ_PROTECTED_ROUTES_CSV: &str = "POST:/v1/messages/send,POST:/v1/channels/create,POST:/v1/tasks/create,GET:/v1/messages/{message_id},GET:/v1/channels/{channel_id}/messages,GET:/v1/tasks/{task_id},GET:/v1/agents/{agent_did},GET:/v1/events/ws";
const SERVICE_API_REQUEST_PATH_AUTHZ_MISSING_HEADER_REASON_CODE: &str =
    "service_api_auth_sender_did_header_missing";
const SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.service-api-scope-policy-reason-taxonomy.v1";
const SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV: &str =
    "service_api_auth_scope_header_missing,service_api_auth_scope_invalid,service_api_auth_scope_route_mismatch";
const SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION: &str =
    "kamn.runtime.service-api-scope-policy-fixture-matrix.v1";
const SERVICE_API_SCOPE_POLICY_FIXTURE_PATH: &str =
    "fixtures/runtime/service_api_scope_policy_fixture_matrix.txt";
const SERVICE_API_TENANT_ISOLATION_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.service-api-tenant-isolation-matrix-policy-reason-taxonomy.v1";
const SERVICE_API_TENANT_ISOLATION_REASON_CODES_CSV: &str = "ci_fast_gate_failed,service_api_tenant_isolation_policy_schema_mismatch,service_api_tenant_isolation_policy_status_invalid,service_api_tenant_isolation_policy_final_decision_invalid,service_api_tenant_isolation_policy_final_decision_mismatch,service_api_tenant_isolation_policy_lane_mode_invalid,service_api_tenant_isolation_policy_matrix_schema_mismatch,service_api_tenant_isolation_policy_matrix_rows_invalid,service_api_tenant_isolation_policy_matrix_row_count_mismatch,service_api_tenant_isolation_policy_matrix_row_duplicate,service_api_tenant_isolation_policy_matrix_row_id_invalid,service_api_tenant_isolation_policy_matrix_row_missing,service_api_tenant_isolation_policy_matrix_row_status_mismatch,service_api_tenant_isolation_policy_matrix_row_leakage_result_mismatch,service_api_tenant_isolation_policy_matrix_row_reason_code_mismatch,service_api_tenant_isolation_policy_matrix_row_selector_mismatch,service_api_tenant_isolation_policy_marker_missing,service_api_tenant_isolation_policy_execution_reason_code_mismatch,service_api_tenant_isolation_policy_command_count_invalid,service_api_tenant_isolation_policy_command_count_mismatch,service_api_tenant_isolation_policy_elapsed_seconds_invalid,service_api_tenant_isolation_policy_max_seconds_invalid,service_api_tenant_isolation_policy_runtime_budget_exceeded,service_api_tenant_isolation_policy_docs_marker_missing";
const SERVICE_API_TENANT_ISOLATION_MATRIX_SCHEMA_VERSION: &str =
    "kamn.runtime.service-api-tenant-isolation-matrix.v1";
const SERVICE_API_TENANT_ISOLATION_REQUIRED_ROW_IDS_CSV: &str =
    "m2_abac_cross_tenant_visibility_denied,m8_cross_owner_retention_and_shred_denied,m9_cross_owner_dispatch_and_presence_denied,m9_gateway_cross_owner_presence_denied";
const API_VERSION_POLICY_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.api-version-policy-reason-taxonomy.v1";
const API_VERSION_POLICY_REASON_CODES_CSV: &str =
    "ci_fast_gate_failed,api_version_policy_schema_mismatch,api_version_policy_status_invalid,api_version_policy_final_decision_invalid,api_version_policy_final_decision_mismatch,api_version_policy_lane_mode_invalid,api_version_policy_fixture_schema_mismatch,api_version_policy_fixture_rows_invalid,api_version_policy_fixture_row_count_mismatch,api_version_policy_fixture_row_duplicate,api_version_policy_fixture_row_id_invalid,api_version_policy_fixture_row_missing,api_version_policy_fixture_row_status_mismatch,api_version_policy_fixture_row_decision_mismatch,api_version_policy_fixture_row_reason_code_mismatch,api_version_policy_fixture_row_version_mismatch,api_version_policy_fixture_row_window_mismatch,api_version_policy_marker_missing,api_version_policy_execution_reason_code_mismatch,api_version_policy_command_count_invalid,api_version_policy_command_count_mismatch,api_version_policy_elapsed_seconds_invalid,api_version_policy_max_seconds_invalid,api_version_policy_runtime_budget_exceeded,api_version_policy_docs_marker_missing";
const API_VERSION_POLICY_FIXTURE_SCHEMA_VERSION: &str =
    "kamn.runtime.api-version-policy-fixture-matrix.v1";
const API_VERSION_POLICY_FIXTURE_PATH: &str =
    "fixtures/runtime/api_version_policy_fixture_matrix.txt";
const API_VERSION_POLICY_REQUIRED_ROW_IDS_CSV: &str =
    "v1_messages_send,v2_channels_create,v0_messages_send,v3_future_route";
const REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.request-response-schema-compatibility-reason-taxonomy.v1";
const REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REASON_CODES_CSV: &str =
    "ci_fast_gate_failed,request_response_schema_compatibility_schema_mismatch,request_response_schema_compatibility_status_invalid,request_response_schema_compatibility_final_decision_invalid,request_response_schema_compatibility_final_decision_mismatch,request_response_schema_compatibility_lane_mode_invalid,request_response_schema_compatibility_fixture_schema_mismatch,request_response_schema_compatibility_fixture_rows_invalid,request_response_schema_compatibility_fixture_row_count_mismatch,request_response_schema_compatibility_fixture_row_duplicate,request_response_schema_compatibility_fixture_row_id_invalid,request_response_schema_compatibility_fixture_row_missing,request_response_schema_compatibility_fixture_row_status_mismatch,request_response_schema_compatibility_fixture_row_decision_mismatch,request_response_schema_compatibility_fixture_row_reason_code_mismatch,request_response_schema_compatibility_fixture_row_version_pair_mismatch,request_response_schema_compatibility_fixture_row_change_class_mismatch,request_response_schema_compatibility_marker_missing,request_response_schema_compatibility_execution_reason_code_mismatch,request_response_schema_compatibility_command_count_invalid,request_response_schema_compatibility_command_count_mismatch,request_response_schema_compatibility_elapsed_seconds_invalid,request_response_schema_compatibility_max_seconds_invalid,request_response_schema_compatibility_runtime_budget_exceeded,request_response_schema_compatibility_docs_marker_missing";
const REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_FIXTURE_SCHEMA_VERSION: &str =
    "kamn.runtime.request-response-schema-compatibility-fixture-matrix.v1";
const REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_FIXTURE_PATH: &str =
    "fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt";
const REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REQUIRED_ROW_IDS_CSV: &str =
    "v1_to_v2_messages_send_optional_request_addition,v1_to_v2_channels_create_optional_response_addition,v1_to_v2_messages_get_required_response_removal,v1_to_v2_tasks_create_required_request_removal";
const AUDIT_INTEGRITY_REASON_TAXONOMY_VERSION: &str =
    "kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1";
const AUDIT_INTEGRITY_REASON_CODES_CSV: &str = "gonogo_audit_integrity_file_missing,gonogo_audit_integrity_invalid_json,gonogo_audit_integrity_schema_mismatch,gonogo_audit_integrity_status_not_ok,gonogo_audit_integrity_final_decision_not_go,gonogo_audit_integrity_policy_status_not_verified,gonogo_audit_integrity_reason_taxonomy_version_mismatch,gonogo_audit_integrity_reason_codes_csv_mismatch,gonogo_audit_integrity_freshness_window_exceeded";
const LIFECYCLE_CI_DRY_RUN_REASON_TAXONOMY_VERSION: &str =
    "kamn.ci.lifecycle-ci-dry-run-governance-reason-taxonomy.v1";
const LIFECYCLE_CI_DRY_RUN_REASON_CODES_CSV: &str = "lifecycle_ci_dry_run_argument_invalid,lifecycle_ci_dry_run_threshold_contract_violation,lifecycle_ci_dry_run_report_contract_violation,lifecycle_ci_dry_run_lifecycle_marker_parity_drift,lifecycle_ci_dry_run_go_no_go_marker_parity_drift,lifecycle_ci_dry_run_runtime_budget_exceeded,lifecycle_ci_dry_run_fast_mode_selector_drift,lifecycle_ci_dry_run_workflow_exclusion_drift,lifecycle_ci_dry_run_docs_marker_parity_drift,lifecycle_ci_dry_run_docs_remediation_marker_missing";
const DEPENDENCY_LICENSE_METADATA_GOVERNANCE_REASON_CODES_CSV: &str = "expected_license_empty,no_crate_manifests_found,license_policy_file_not_found,license_policy_marker_mismatch,manifest_not_found,manifest_invalid_toml,package_section_missing,license_missing,license_mismatch,metadata_governance_local_heavy_opt_in_required";

fn fairness_reason_codes() -> Vec<&'static str> {
    FAIRNESS_REASON_CODES_CSV.split(',').collect()
}

fn deletion_reason_codes() -> Vec<&'static str> {
    DELETION_REASON_CODES_CSV.split(',').collect()
}

fn local_heavy_redaction_policy_reason_codes() -> Vec<&'static str> {
    LOCAL_HEAVY_REDACTION_POLICY_REASON_CODES_CSV
        .split(',')
        .collect()
}

fn overload_reason_codes() -> Vec<&'static str> {
    OVERLOAD_REASON_CODES_CSV.split(',').collect()
}

fn performance_ci_smoke_reason_codes() -> Vec<&'static str> {
    PERFORMANCE_CI_SMOKE_REASON_CODES_CSV.split(',').collect()
}

fn service_api_request_path_authz_reason_codes() -> Vec<&'static str> {
    SERVICE_API_REQUEST_PATH_AUTHZ_REASON_CODES_CSV
        .split(',')
        .collect()
}

fn service_api_scope_policy_reason_codes() -> Vec<&'static str> {
    SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV
        .split(',')
        .collect()
}

fn service_api_tenant_isolation_reason_codes() -> Vec<&'static str> {
    SERVICE_API_TENANT_ISOLATION_REASON_CODES_CSV
        .split(',')
        .collect()
}

fn api_version_policy_reason_codes() -> Vec<&'static str> {
    API_VERSION_POLICY_REASON_CODES_CSV.split(',').collect()
}

fn request_response_schema_compatibility_reason_codes() -> Vec<&'static str> {
    REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REASON_CODES_CSV
        .split(',')
        .collect()
}

fn audit_integrity_reason_codes() -> Vec<&'static str> {
    AUDIT_INTEGRITY_REASON_CODES_CSV.split(',').collect()
}

fn lifecycle_ci_dry_run_reason_codes() -> Vec<&'static str> {
    LIFECYCLE_CI_DRY_RUN_REASON_CODES_CSV.split(',').collect()
}

fn dependency_license_metadata_governance_reason_codes() -> Vec<&'static str> {
    DEPENDENCY_LICENSE_METADATA_GOVERNANCE_REASON_CODES_CSV
        .split(',')
        .collect()
}
#[path = "ci_strategy_docs/service_api_policy_support.rs"]
mod service_api_policy_support;
#[path = "ci_strategy_docs/service_api_request_path_authz_contract_tests.rs"]
mod service_api_request_path_authz_contract_tests;
#[path = "ci_strategy_docs/service_api_scope_policy_contract_tests.rs"]
mod service_api_scope_policy_contract_tests;
#[path = "ci_strategy_docs/service_api_tenant_isolation_contract_tests.rs"]
mod service_api_tenant_isolation_contract_tests;
#[path = "ci_strategy_docs/api_version_policy_contract_tests.rs"]
mod api_version_policy_contract_tests;
#[path = "ci_strategy_docs/request_response_schema_compatibility_contract_tests.rs"]
mod request_response_schema_compatibility_contract_tests;
#[path = "ci_strategy_docs/fairness_deletion_support.rs"]
mod fairness_deletion_support;
#[path = "ci_strategy_docs/fairness_docs_parity_contract_tests.rs"]
mod fairness_docs_parity_contract_tests;
#[path = "ci_strategy_docs/deletion_docs_parity_contract_tests.rs"]
mod deletion_docs_parity_contract_tests;
#[path = "ci_strategy_docs/performance_docs_contract_tests.rs"]
mod performance_docs_contract_tests;
#[path = "ci_strategy_docs/governance_gate_contract_tests.rs"]
mod governance_gate_contract_tests;
#[path = "ci_strategy_docs/convergence_governance_contract_tests.rs"]
mod convergence_governance_contract_tests;
#[path = "ci_strategy_docs/local_heavy_policy_contract_tests.rs"]
mod local_heavy_policy_contract_tests;
#[path = "ci_strategy_docs/overload_governance_contract_tests.rs"]
mod overload_governance_contract_tests;
#[path = "ci_strategy_docs/public_api_surface_contract_tests.rs"]
mod public_api_surface_contract_tests;
#[path = "ci_strategy_docs/service_api_runtime_contract_lane_tests.rs"]
mod service_api_runtime_contract_lane_tests;
#[path = "ci_strategy_docs/runtime_local_contract_lane_tests.rs"]
mod runtime_local_contract_lane_tests;
#[path = "ci_strategy_docs/governance_advisory_tail_contract_tests.rs"]
mod governance_advisory_tail_contract_tests;
#[path = "ci_strategy_docs/make_demo_governance_contract_tests.rs"]
mod make_demo_governance_contract_tests;

#[test]
fn doc_contains_touched_shell_strict_mode_markers() {
    assert!(DOC.contains("test_check_touched_shell_strict_mode.sh"));
    assert!(DOC.contains("fixtures/ci/touched_shell_strict_mode_exceptions.txt"));
    assert!(DOC.contains(
        "check_touched_shell_strict_mode.sh --output-json /tmp/touched-shell-strict-mode-report.json"
    ));
    assert!(DOC.contains("reason_codes=touched_shell_strict_mode_missing_strict_mode"));
    assert!(DOC.contains("reason_codes=touched_shell_strict_mode_git_base_unavailable"));
    assert!(DOC.contains("reason_codes=touched_shell_strict_mode_exception_file_invalid"));
}

#[test]
fn doc_contains_signer_provenance_fallback_policy_contract_markers() {
    assert!(
        DOC.contains("### Signer Provenance and Fallback-Prohibition Docs/Config Parity Contract")
    );
    assert!(DOC.contains("signer_provenance_fallback_policy_contract_status=active"));
    assert!(DOC.contains("signer_provenance_fallback_policy_contract_version=v1"));
    assert!(DOC.contains(
        "signer_provenance_fallback_policy_required_markers_csv=runtime_signer_key_source_policy_reason_codes_csv,managed_signer_backend_response_provenance_missing,managed_signer_backend_response_provenance_malformed,managed_signer_backend_response_provenance_mismatch"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node --test signer_provenance_fallback_policy_contract -- --nocapture"
    ));
    assert!(DOC.contains("production_signer_key_source_env_local_forbidden"));
    assert!(DOC.contains("fallback_signer_secret_present_violation"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_missing"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_malformed"));
    assert!(DOC.contains("managed_signer_backend_response_provenance_mismatch"));
}

#[test]
fn doc_contains_node_runtime_startup_negative_matrix_fast_lane_contract_markers() {
    assert!(DOC.contains("## Node Runtime Startup Negative-Matrix Fast Lane"));
    assert!(DOC.contains(
        "cargo test -p kamn-node main_tests::cli_contract_tests::regression_3599_startup_signer_mode_negative_matrix_corpus -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node cli_tests::regression_3598_startup_paths_have_no_panic_control_flow -- --exact"
    ));
    assert!(DOC.contains("startup_negative_matrix_policy_marker_missing"));
    assert!(DOC.contains("must fail before network dispatch"));
}

#[test]
fn regression_requires_make_and_selector_demo_contract_marker() {
    // Regression: #900
    assert!(DOC.contains("Regression: #900"));
    assert!(DOC.contains("make-target and selector workflow drift"));
    assert!(DOC.contains("Regression: #1419"));
    assert!(DOC.contains("Regression: #1431"));
    assert!(DOC.contains("Regression: #1682"));
    assert!(DOC.contains("Regression: #1687"));
    assert!(DOC.contains("Regression: #1697"));
    assert!(DOC.contains("Regression: #1702"));
    assert!(DOC.contains("Regression: #1707"));
    assert!(DOC.contains("Regression: #1692"));
    assert!(DOC.contains("Regression: #1441"));
    assert!(DOC.contains("Regression: #1451"));
    assert!(DOC.contains("Regression: #1467"));
    assert!(DOC.contains("Regression: #1468"));
    assert!(DOC.contains("Regression: #1482"));
    assert!(DOC.contains("Regression: #1483"));
    assert!(DOC.contains("Regression: #1488"));
    assert!(DOC.contains("Regression: #1489"));
    assert!(DOC.contains("Regression: #1494"));
    assert!(DOC.contains("Regression: #1462"));
    assert!(DOC.contains("Regression: #1466"));
    assert!(DOC.contains("Regression: #1497"));
    assert!(DOC.contains("Regression: #2694"));
    assert!(DOC.contains("Regression: #2690"));
    assert!(DOC.contains("Regression: #2691"));
    assert!(DOC.contains("Regression: #2692"));
    assert!(DOC.contains("Regression: #2693"));
    assert!(DOC.contains("Regression: #2093"));
    assert!(DOC.contains("Regression: #2095"));
    assert!(DOC.contains("Regression: #2658"));
    assert!(DOC.contains("Regression: #2703"));
    assert!(DOC.contains("Regression: #2705"));
    assert!(DOC.contains("Regression: #2711"));
    assert!(DOC.contains("Regression: #2714"));
    assert!(DOC.contains("Regression: #2717"));
    assert!(DOC.contains("Regression: #2720"));
    assert!(DOC.contains("Regression: #2723"));
    assert!(DOC.contains("Regression: #2726"));
    assert!(DOC.contains("Regression: #2729"));
    assert!(DOC.contains("Regression: #2732"));
    assert!(DOC.contains("Regression: #2735"));
    assert!(DOC.contains("Regression: #2738"));
    assert!(DOC.contains("Regression: #2741"));
}

#[test]
fn doc_contains_ignored_test_and_script_budget_trend_composed_contract_markers() {
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/ci_ignored_test_and_script_budget_trend_contract_lane.json --phase contract --output-json /tmp/ignored-test-script-soft-budget-trend-contract-report.json"
    ));
    assert!(DOC.contains("test_run_ignored_test_and_script_budget_trend_contract_lane.sh"));
    assert!(DOC.contains("ignored_test_metadata_stale_entry"));
    assert!(DOC.contains("combined_shell_surface_shell_line_total_delta_fail_exceeded"));
    assert!(DOC.contains("combined_shell_surface_ratio_fail_ceiling_exceeded"));
    assert!(DOC.contains("ignored_test_script_budget_trend_contract_status=pass|fail"));
}

#[test]
fn doc_contains_combined_shell_surface_baseline_refresh_workflow_markers() {
    assert!(DOC.contains(
        "combined_shell_surface_baseline_refresh_trigger_reason=combined_shell_surface_shell_line_total_delta_fail_exceeded"
    ));
    assert!(DOC.contains(
        "combined_shell_surface_baseline_refresh_command=bash scripts/ci/generate_combined_shell_surface_trend_report.sh --budget-file .ci/script-surface-budget.env --script-baseline-file .ci/script-surface-baseline.env --combined-baseline-file fixtures/ci/combined_shell_surface_trend_baseline.json --output-json /tmp/combined-shell-surface-trend-report.json"
    ));
    assert!(DOC.contains(
        "combined_shell_surface_baseline_refresh_contract=update fixtures/ci/combined_shell_surface_trend_baseline.json with report.current metrics in the same PR"
    ));
    assert!(DOC.contains(
        "combined_shell_surface_baseline_refresh_validation=bash scripts/ci/test_check_combined_shell_surface_trend_policy.sh"
    ));
}

#[test]
fn doc_contains_test_harness_structural_budget_reason_taxonomy_and_ci_smoke_markers() {
    assert!(DOC.contains(
        "test_harness_loc_soft_budget_reason_taxonomy_version=kamn.ci.test-harness-loc-soft-budget-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "test_harness_loc_soft_budget_reason_codes_csv=report_file_not_found,budget_file_not_found,baseline_file_not_found,trend_threshold_file_not_found,report_json_invalid,report_schema_mismatch,report_harness_script_count_invalid,report_harness_shell_line_total_invalid,budget_key_missing,budget_value_invalid,baseline_key_missing,baseline_value_invalid,trend_threshold_key_missing,trend_threshold_value_invalid,trend_threshold_order_invalid,harness_script_count_soft_max_exceeded,harness_shell_line_total_soft_max_exceeded,harness_script_count_trend_warn_delta_exceeded,harness_shell_line_total_trend_warn_delta_exceeded,harness_script_count_trend_fail_delta_exceeded,harness_shell_line_total_trend_fail_delta_exceeded,trend_fail_enforcement_triggered"
    ));
    assert!(DOC.contains("test_harness_loc_soft_budget_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("test_harness_loc_soft_budget_reason_class=stable|budgeted|violation"));
    assert!(DOC.contains("test_harness_loc_soft_budget_ci_smoke_lane_cost_profile=low"));
    assert!(
        DOC.contains("test_harness_loc_soft_budget_ci_smoke_runtime_budget_status=within|exceeded")
    );
    assert!(DOC.contains("test_harness_loc_soft_budget_contract_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains(
        "test_harness_loc_soft_budget_contract_ci_smoke_runtime_budget_status=within|exceeded"
    ));
    assert!(DOC.contains(
        "test_harness_loc_soft_budget_contract_reason_key=test_harness_loc_soft_budget_contract_ok|test_harness_loc_soft_budget_contract_runtime_budget_exceeded"
    ));
}

#[test]
fn doc_contains_runtime_local_full_mode_live_validation_runtime_error_taxonomy_markers() {
    assert!(DOC.contains("## Runtime Local Full-Mode Live Validation Contract Lane"));
    assert!(DOC.contains(
        "validate_local_full_runtime_live.sh --mode dry-run --output-json /tmp/local-full-runtime-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_local_full_runtime_live_policy.sh --report-file /tmp/local-full-runtime-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-full-runtime-live-policy.json"
    ));
    assert!(DOC.contains(
        "validate_local_full_runtime_live_contract_lane.sh --output-json /tmp/local-full-runtime-live-contract-lane-report.json --policy-output-json /tmp/local-full-runtime-live-policy.json"
    ));
    assert!(DOC.contains("runtime_shutdown_gate_status=verified"));
    assert!(DOC.contains("runtime_fallback_classification_status=verified"));
    assert!(DOC.contains(
        "runtime_error_reason_taxonomy_version=kamn.runtime.local-full-runtime-error-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "runtime_error_reason_codes_csv=runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("ci_local_runtime_extraction_budget_boundary_status=verified"));
    assert!(DOC.contains("runtime_full_shutdown_gate_drift_detected"));
    assert!(DOC.contains("runtime_fallback_classification_unstable"));
    assert!(DOC.contains("ci_local_runtime_extraction_budget_boundary_exceeded"));
}

#[test]
fn doc_contains_runtime_local_full_stack_runtime_budget_policy_markers() {
    assert!(DOC.contains("local_heavy_runtime_budget_status"));
    assert!(DOC.contains(
        "runtime_phase_parity_reason_taxonomy_version=kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "runtime_phase_parity_reason_codes_csv=runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("runtime_phase_module_parity_status=verified"));
    assert!(DOC.contains("runtime_extraction_evidence_output_status=verified"));
    assert!(DOC.contains("ci_local_runtime_phase_parity_budget_boundary_status=verified"));
    assert!(DOC.contains("elapsed_seconds"));
    assert!(DOC.contains("max_seconds"));
    assert!(DOC.contains("command_max_seconds"));
    assert!(DOC.contains("local_full_stack_integration_policy_runtime_budget_status_mismatch"));
    assert!(DOC.contains("local_full_stack_integration_policy_runtime_budget_exceeded"));
    assert!(DOC.contains("runtime_phase_module_parity_drift_detected"));
    assert!(DOC.contains("runtime_extraction_evidence_output_unstable"));
    assert!(DOC.contains("ci_local_runtime_phase_parity_budget_boundary_exceeded"));
    assert!(DOC.contains(
        "runtime_module_boundary_parity_reason_taxonomy_version=kamn.runtime.module-boundary-parity-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "runtime_module_boundary_parity_reason_codes_csv=runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("runtime_module_boundary_reason_codes_value=none|<csv>"));
    assert!(DOC.contains(
        "runtime_module_boundary_evidence_outputs_csv=runtime_module_boundary_parity_status,runtime_module_boundary_evidence_status,ci_local_runtime_module_boundary_budget_boundary_status"
    ));
    assert!(DOC.contains("runtime_orchestration_dispatch_boundary_status=verified"));
    assert!(DOC.contains("runtime_daemon_phase_boundary_status=verified"));
    assert!(DOC.contains("runtime_kolme_live_boundary_status=verified"));
    assert!(DOC.contains("runtime_module_boundary_parity_status=verified"));
    assert!(DOC.contains("runtime_module_boundary_evidence_status=verified"));
    assert!(DOC.contains("ci_local_runtime_module_boundary_budget_boundary_status=verified"));
    assert!(DOC.contains("ci_local_runtime_module_boundary_budget_boundary_exceeded"));
}

#[test]
fn doc_contains_message_anchoring_ci_boundary_taxonomy_markers() {
    assert!(DOC.contains(
        "anchoring_gate_reason_taxonomy_version=kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "anchoring_gate_reason_codes_csv=message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required"
    ));
    assert!(DOC.contains("ci_smoke_local_heavy_boundary_status=verified"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=opt_in"));
    assert!(DOC.contains("test_run_message_proof_anchoring_contract_lane.sh"));
    assert!(DOC.contains("test_validate_message_proof_anchoring_live.sh"));
}

#[test]
fn doc_contains_dependency_license_metadata_governance_taxonomy_and_boundary_markers() {
    assert!(DOC.contains(
        "metadata_governance_reason_taxonomy_version=kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(&format!(
        "metadata_governance_reason_codes_csv={DEPENDENCY_LICENSE_METADATA_GOVERNANCE_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains("metadata_governance_reason_codes_value=none|<csv>"));
    assert!(DOC.contains(
        "metadata_governance_reason_class=stable|metadata_mismatch|configuration|boundary|mixed"
    ));
    assert!(DOC.contains("ci_smoke_local_heavy_boundary_status=verified|violation"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low|not-applicable"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=not_requested|opt_in|blocked"));
    assert!(DOC.contains(
        "python3 scripts/ci/check_workspace_license_policy.py --workspace-root . --expected-license Apache-2.0 --license-policy-file LICENSE --lane-profile ci-smoke"
    ));
    assert!(DOC.contains(
        "python3 scripts/ci/check_workspace_license_policy.py --workspace-root . --expected-license Apache-2.0 --license-policy-file LICENSE --lane-profile local-heavy --local-heavy-opt-in"
    ));
}

fn assert_supply_chain_doc_marker(marker: &str) {
    assert!(DOC.contains(marker), "missing supply-chain advisory marker: {marker}");
}

#[test]
fn doc_contains_supply_chain_advisory_lane_markers() {
    for marker in [
        "supply_chain_advisory_lane_status=advisory_only",
        "supply_chain_advisory_tools_csv=trivy_fs,trivy_image,workspace_license_policy",
        "supply_chain_advisory_sbom_format=cyclonedx",
        "supply_chain_advisory_false_positive_controls=.trivyignore + workflow continue-on-error",
        "supply_chain_advisory_promotion_follow_up_issue=",
    ] {
        assert_supply_chain_doc_marker(marker);
    }
}

#[test]
fn doc_enforces_dependency_license_metadata_remediation_markers_cover_reason_codes() {
    assert!(DOC.contains("metadata_governance_remediation_map_version=v1"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_dependency_license_metadata_remediation_markers_cover_reason_codes -- --exact"
    ));
    for reason_code in dependency_license_metadata_governance_reason_codes() {
        assert!(
            DOC.contains(&format!("metadata_governance_remediation.{reason_code}=")),
            "missing dependency-license remediation marker for {reason_code}"
        );
        assert!(
            OPS_DOC.contains(&format!("metadata_governance_remediation.{reason_code}=")),
            "ops docs missing dependency-license remediation marker for {reason_code}"
        );
    }
}
