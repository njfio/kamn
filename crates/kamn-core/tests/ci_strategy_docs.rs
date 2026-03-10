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

#[test]
fn doc_contains_make_and_demo_scope_contract_rules() {
    assert!(DOC.contains("make check"));
    assert!(DOC.contains("make test"));
    assert!(DOC.contains("make demo"));
    assert!(DOC.contains("## Test Layering Policy Contract"));
    assert!(DOC.contains("scripts/ci/check_test_layering_policy.py"));
    assert!(DOC.contains("scripts/ci/test_check_test_layering_policy.sh"));
    assert!(DOC.contains("docs/planning/test_layering_policy.md"));
    assert!(DOC.contains("## Snapshot + Journal Durability Replay Contract"));
    assert!(DOC.contains("docs/planning/persistence_durability_model.md"));
    assert!(DOC.contains("cargo test -p kamn-core --lib journal"));
    assert!(DOC.contains("channel_snapshot_journal_corrupt_tail:<line>"));
    assert!(DOC.contains("message_lifecycle_snapshot_journal_corrupt_tail:<line>"));
    assert!(DOC.contains("task_operation_snapshot_journal_corrupt_tail:<line>"));
    assert!(DOC.contains("## Runtime Backpressure Enforcement Contract"));
    assert!(DOC.contains("docs/planning/runtime_backpressure_policy.md"));
    assert!(DOC.contains("cargo test -p kamn-core --lib backpressure"));
    assert!(DOC.contains("cargo test -p kamn-core --lib network_fault_simulation"));
    assert!(DOC.contains("runtime_backpressure_reject_new_enqueue"));
    assert!(DOC.contains("runtime_backpressure_purge_stale_peer_queue"));
    assert!(DOC.contains("## Lifecycle Property Shrinking Contract"));
    assert!(DOC.contains("docs/planning/property_invariant_matrix.md"));
    assert!(DOC.contains("cargo test -p kamn-core --test lifecycle_property_shrinking"));
    assert!(DOC.contains("cargo test -p kamn-core --test lifecycle_evidence_property_matrix"));
    assert!(DOC.contains("minimal failing prefix"));
    assert!(DOC.contains("## Coverage-Guided Parser Fuzz Contract"));
    assert!(DOC.contains("docs/planning/fuzz_harness_budget_policy.md"));
    assert!(DOC.contains(
        "run_input_mutation_coverage_guided_contract_lane.sh --output-json /tmp/input-mutation-coverage-guided-contract-report.json"
    ));
    assert!(DOC.contains(
        "run_input_mutation_coverage_guided_contract_lane.sh --target envelope --output-json /tmp/input-mutation-coverage-guided-envelope-report.json"
    ));
    assert!(DOC.contains(
        "run_input_mutation_coverage_guided_contract_lane.sh --target did --output-json /tmp/input-mutation-coverage-guided-did-report.json"
    ));
    assert!(DOC.contains("run_input_mutation_coverage_guided_deep_lane.sh"));
    assert!(DOC.contains("runtime_input_mutation_coverage_guided_deep=skipped_local_only"));
    assert!(DOC.contains("KAMN_RUNTIME_INPUT_MUTATION_COVERAGE_GUIDED_DEEP_LOCAL_ONLY=true"));
    assert!(DOC.contains(
        "main_tests::functional_kolme_live_retry_emits_structured_retry_markers -- --exact"
    ));
    assert!(DOC.contains(
        "main_tests::functional_runtime_daemon_emits_structured_transition_markers -- --exact"
    ));
    assert!(DOC.contains("kolme.live.submit.retry"));
    assert!(DOC.contains("kolme.live.finality.retry"));
    assert!(DOC.contains("node.runtime.daemon.execute.start"));
    assert!(DOC.contains("node.runtime.daemon.execute.complete"));
    assert!(DOC.contains("layering_marker_missing"));
    assert!(DOC.contains("run_localhost_signed_integration_contract_lane_tests"));
    assert!(DOC.contains("sdk-live-localhost-integration"));
    assert!(DOC.contains("KAMN_CI_TOOLS_FAST_MODE=true"));
    assert!(DOC.contains(
        "bash scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh"
    ));
    assert!(DOC.contains(
        "`validate_unified_api_observability_local_heavy_live.sh --mode run --ci-fast-gate FAIL` must not appear in `.github/workflows/ci-fast-gate.yml` or `scripts/ci/test_ci_tools.sh` fast-mode block."
    ));
    assert!(DOC.contains("cargo test -p kamn-core --test shell_test_surface_migration_wave1"));
    assert!(DOC.contains("cargo test -p kamn-core --test shell_test_surface_ratio_policy"));
    assert!(DOC.contains("legacy ingress parser drift checker contract"));
    assert!(DOC.contains(
        "check_legacy_ingress_parser_drift.sh --source-root crates/kamn-node/src --baseline-file fixtures/ci/legacy_ingress_parser_baseline.json --output-json /tmp/legacy-ingress-parser-drift-report.json"
    ));
    assert!(DOC.contains("fixtures/ci/shell_test_surface_ratio_baseline.env"));
    assert!(DOC.contains(".ci/shell_test_surface_ratio_thresholds.env"));
    assert!(DOC.contains("policy_status=within|waiver-applied|fail"));
    assert!(DOC.contains("reason_codes=ratio_fail_threshold_exceeded_unwaived"));
    assert!(DOC.contains("reason_codes=ratio_fail_threshold_waiver_applied"));
    assert!(DOC.contains("reason_codes=legacy_ingress_parser_marker_count_increased"));
    assert!(DOC.contains("reason_codes=legacy_ingress_parser_marker_new_file"));
    assert!(DOC.contains("reason_codes=legacy_ingress_parser_baseline_missing"));
    assert!(DOC.contains("reason_codes=legacy_ingress_parser_baseline_invalid"));
    assert!(DOC.contains("run_localhost_signed_integration_contract_lane.sh"));
    assert!(DOC.contains("scripts/ci/select_targets.sh"));
    assert!(DOC.contains("run_kolme_version_compatibility_contract_tests=true"));
    assert!(DOC.contains("test_run_fast_gate_native_api_parity_contract_lane.sh"));
    assert!(DOC.contains("run_fast_gate_native_api_parity_contract_lane.sh"));
    assert!(DOC.contains("check_fast_gate_native_api_parity_policy.py"));
    assert!(DOC.contains("KAMN_KOLME_FAST_GATE_NATIVE_PARITY_MAX_SECONDS=120"));
    assert!(DOC.contains("test_run_continuous_runtime_commit_contract_lane.sh"));
    assert!(DOC.contains("test_run_did_lifecycle_chain_adapter_contract_lane.sh"));
    assert!(DOC.contains("test_run_message_proof_anchoring_contract_lane.sh"));
    assert!(DOC.contains("test_run_managed_signer_startup_live_validation_contract_lane.sh"));
    assert!(DOC.contains("test_validate_continuous_runtime_commit_live.sh"));
    assert!(DOC.contains("test_validate_did_lifecycle_chain_adapter_live.sh"));
    assert!(DOC.contains("test_validate_message_proof_anchoring_live.sh"));
    assert!(DOC.contains("non_kolme_wave5_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave5_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave5_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave5_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave5_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave5_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave6_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave6_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave6_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave6_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave6_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave6_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave7_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave7_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave7_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave7_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave7_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave7_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave8_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave8_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave8_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave8_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave8_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave8_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave9_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave9_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave9_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave9_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave9_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave9_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave10_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave10_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave10_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave10_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave10_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave10_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("kolme_wave10_wrapper_family_matrix.json"));
    assert!(DOC.contains("kolme_wave10_wrapper_family_baseline.json"));
    assert!(DOC.contains("kolme_wave10_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_kolme_wave10_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_kolme_wave10_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_kolme_wave10_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("kolme_wave11_wrapper_family_matrix.json"));
    assert!(DOC.contains("kolme_wave11_wrapper_family_baseline.json"));
    assert!(DOC.contains("kolme_wave11_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_kolme_wave11_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_kolme_wave11_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_kolme_wave11_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave11_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave11_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave11_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave11_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave11_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave11_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave12_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave12_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave12_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave12_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave12_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave12_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave13_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave13_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave13_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave13_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave13_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave13_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave14_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave14_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave14_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave14_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave14_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave14_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave15_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave15_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave15_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave15_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave15_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave15_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave16_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave16_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave16_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave16_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave16_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave16_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave17_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave17_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave17_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave17_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave17_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave17_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("non_kolme_wave18_wrapper_family_matrix.json"));
    assert!(DOC.contains("non_kolme_wave18_wrapper_family_baseline.json"));
    assert!(DOC.contains("non_kolme_wave18_wrapper_family_trend_thresholds.json"));
    assert!(DOC.contains("test_non_kolme_wave18_wrapper_family_baseline_contract.sh"));
    assert!(DOC.contains("test_check_non_kolme_wave18_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("check_non_kolme_wave18_wrapper_family_budget_trend.sh"));
    assert!(DOC.contains("Non-Kolme bridge dispatcher wrapper-matrix guard stays on PR fast gate:"));
    assert!(DOC.contains("test_non_kolme_bridge_contract_lane_dispatch_wrapper_matrix.sh"));
    assert!(DOC.contains("Non-Kolme sdk dispatcher wrapper-matrix guard stays on PR fast gate:"));
    assert!(DOC.contains("test_non_kolme_sdk_contract_lane_dispatch_wrapper_matrix.sh"));
    assert!(DOC
        .contains("Non-Kolme lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"));
    assert!(DOC.contains("test_non_kolme_lightweight_contract_lane_dispatch_wrapper_matrix.sh"));
    assert!(DOC.contains(
        "Non-Kolme wave-10 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave10_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-11 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave11_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-12 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave12_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-13 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave13_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-14 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave14_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-15 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave15_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-16 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave16_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-17 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave17_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains(
        "Non-Kolme wave-18 lightweight dispatcher wrapper-matrix guard stays on PR fast gate:"
    ));
    assert!(
        DOC.contains("test_non_kolme_wave18_lightweight_contract_lane_dispatch_wrapper_matrix.sh")
    );
    assert!(DOC.contains("run_local_fork_sync_metadata_lane.sh --mode run"));
    assert!(DOC.contains("run_local_fork_smoke_evidence_lane.sh --mode run"));
    assert!(DOC.contains(
        "run_local_kolme_api_probe_lane.sh --mode run --base-url http://127.0.0.1:3000 --fork-chain-version v0.15.2"
    ));
    assert!(DOC.contains("run_local_kolme_api_smoke_lane.sh --mode run"));
    assert!(DOC.contains("run_local_kolme_live_api_conformance_harness.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_live_api_conformance_policy.py --report-file /tmp/kolme-local-live-api-conformance-summary.json"
    ));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_live_api_conformance_contract_lane.json --phase contract --output-json /tmp/kolme-local-live-api-conformance-summary.json --policy-output-json /tmp/kolme-local-live-api-conformance-policy.json"
    ));
    assert!(DOC.contains("run_local_kolme_fork_bootstrap_readiness_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_fork_bootstrap_readiness_policy.py --report-file /tmp/kolme-local-fork-bootstrap-readiness-summary.json"
    ));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_fork_bootstrap_readiness_contract_lane.json --phase contract --output-json /tmp/kolme-local-fork-bootstrap-readiness-summary.json --policy-output-json /tmp/kolme-local-fork-bootstrap-readiness-policy.json"
    ));
    assert!(DOC.contains("run_local_kamn_live_runtime_integration_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kamn_live_runtime_integration_policy.py --report-file /tmp/kolme-local-kamn-live-runtime-integration-summary.json"
    ));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kamn_live_runtime_integration_contract_lane.json --phase contract --output-json /tmp/kolme-local-kamn-live-runtime-integration-summary.json --policy-output-json /tmp/kolme-local-kamn-live-runtime-integration-policy.json"
    ));
    assert!(DOC.contains(
        "composite_gate_reason_taxonomy_version=kamn.kolme.live-provider-native-signer-composite-gate-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "composite_gate_reason_codes_csv=dry_run_no_commands_executed,live_runtime_integration_passed,runtime_signer_fallback_private_key_present_violation,runtime_signer_managed_external_raw_private_key_present_violation,local_opt_in_missing,bootstrap_readiness_failed,localhost_signed_integration_failed,live_api_conformance_failed,runtime_commit_endpoint_failed,runtime_commit_policy_failed,runtime_integration_budget_exceeded"
    ));
    assert!(DOC.contains("composite_gate_evidence_convergence_status=verified"));
    assert!(DOC.contains("composite_gate_ci_smoke_local_heavy_boundary_status=verified"));
    assert!(DOC.contains("composite_gate_ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("composite_gate_local_heavy_execution_mode=not_requested"));
    assert!(DOC.contains("run_local_kolme_fork_process_lifecycle_lane.sh --mode run"));
    assert!(DOC.contains(
        "check_local_kolme_fork_process_lifecycle_policy.py --report-file /tmp/kolme-local-fork-process-lifecycle-summary.json"
    ));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/kolme_local_kolme_fork_process_lifecycle_contract_lane.json --phase contract --output-json /tmp/kolme-local-fork-process-lifecycle-summary.json --policy-output-json /tmp/kolme-local-fork-process-lifecycle-policy.json"
    ));
    assert!(DOC.contains("run_local_kolme_fork_profile_preflight_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_profile_preflight_contract_lane.sh"));
    assert!(DOC.contains("run_local_kolme_fork_self_test_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_self_test_contract_lane.sh"));
    assert!(DOC.contains("run_local_kolme_fork_portability_preflight_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_portability_preflight_contract_lane.sh"));
    assert!(DOC.contains("run_local_runtime_commit_live_lane.sh --mode run"));
    assert!(DOC.contains("run_local_native_api_parity_live_proof_lane.sh --mode run"));
    assert!(
        DOC.contains("--request PUT --data '{\\\"message\\\":\\\"native-parity\\\",\\\"signature\\\":\\\"sig\\\",\\\"recovery_id\\\":1}' http://127.0.0.1:3000/broadcast")
    );
    assert!(DOC.contains("test_run_local_runtime_commit_live_lane.sh"));
    assert!(DOC.contains("test_run_local_native_api_parity_live_proof_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_live_api_conformance_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_bootstrap_readiness_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kamn_live_runtime_integration_contract_lane.sh"));
    assert!(DOC.contains("test_run_local_kolme_fork_process_lifecycle_contract_lane.sh"));
    assert!(DOC.contains("run_nonce_broadcast_parity_contract_lane.sh"));
    assert!(DOC.contains("test_run_nonce_broadcast_parity_contract_lane.sh"));
    assert!(DOC.contains("KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS=60"));
    assert!(DOC.contains("run_local_bootstrap_health_checks.sh"));
    assert!(DOC.contains("check_local_bootstrap_health_policy.py"));
    assert!(DOC.contains("run_local_bootstrap_health_checks_contract_lane.sh"));
    assert!(DOC.contains("test_check_local_bootstrap_health_policy.sh"));
    assert!(DOC.contains("test_run_local_bootstrap_health_checks_contract_lane.sh"));
    assert!(DOC.contains("run_local_e2e_integration_lane.sh"));
    assert!(DOC.contains("check_local_e2e_integration_policy.py"));
    assert!(DOC.contains("run_local_e2e_integration_contract_lane.sh"));
    assert!(DOC.contains("run_local_heavy_validation_matrix.sh"));
    assert!(DOC.contains("check_local_heavy_validation_matrix_policy.py"));
    assert!(DOC.contains("run_local_heavy_validation_matrix_contract_lane.sh"));
    assert!(DOC.contains("KAMN_KOLME_LOCAL_HEAVY=1"));
    assert!(
        DOC.contains("local-only heavy Kolme run-mode commands remain excluded from ci-fast-gate.")
    );
    assert!(DOC.contains("kolme_local_heavy_lane_mode=local-only|manual-opt-in|not-applicable"));
    assert!(DOC.contains("manual-hardened mode: manual"));
    assert!(DOC.contains(
        "local-only fork sync/smoke run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local Kolme API probe/smoke run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local live API conformance harness run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local fork bootstrap/readiness run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local KAMN live runtime integration run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local fork process lifecycle integration run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local fork profile preflight run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(
        DOC.contains("local fork self-test run-mode commands remain excluded from ci-fast-gate.")
    );
    assert!(DOC.contains(
        "local fork portability preflight run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "check_local_kolme_fork_portability_preflight_policy.py --report-file /tmp/kolme-local-fork-portability-preflight-summary.json"
    ));
    assert!(DOC.contains(
        "local runtime-commit live run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "local native API parity live-proof run-mode commands remain excluded from ci-fast-gate."
    ));
    assert!(DOC.contains(
        "native parity fast/local command matrix remains synchronized across `README.md` and `docs/planning/kolme-devnet-ops.md`."
    ));
    assert!(DOC.contains(
        "baseline script inventory remains authoritative; any new script path must be documented by refreshing the committed baseline fixture in the same change."
    ));
    assert!(DOC.contains("reason_codes=unexpected_current_scripts"));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/ci_fast_gate_budget_delta_contract_lane.json --phase contract --output-json /tmp/fast-gate-budget-delta-contract-report.json"
    ));
    assert!(DOC.contains("test_run_fast_gate_budget_delta_contract_lane.sh"));
    assert!(DOC.contains("reason_codes=fast_gate_delta_threshold_file_stale"));
    assert!(DOC.contains("reason_codes=fast_gate_delta_threshold_file_corrupt"));
    assert!(DOC.contains("refresh .ci/fast-gate-budget-delta.env baseline and threshold metadata"));
    assert!(DOC.contains(
        "check_non_kolme_wave_trend_test_loc_soft_budget.sh --waiver-file .ci/non_kolme_wave_trend_test_loc_soft_budget_waiver.json --output-json /tmp/non-kolme-wave-trend-test-loc-soft-budget-report.json"
    ));
    assert!(DOC.contains("reason_codes=delta_threshold_violation_unwaived"));
    assert!(DOC.contains("reason_codes=delta_threshold_waiver_applied"));
    assert!(DOC.contains("reason_codes=waiver_expired"));
    assert!(DOC.contains("reason_codes=waiver_scope_mismatch"));
    assert!(DOC.contains("native_libp2p_provider_marker=p2p-live-libp2p-provider:native"));
    assert!(DOC.contains(
        "libp2p_fallback_marker_blocklist=p2p-in-memory-transport-fallback,p2p-live-libp2p-provider:contract-only"
    ));
    assert!(DOC.contains("libp2p_fallback_markers_detected=none"));
    assert!(DOC.contains("native_libp2p_provider_marker_contract_status=verified"));
    assert!(DOC.contains("gate_policy_native_libp2p_provider_marker_mismatch"));
    assert!(DOC.contains("gate_policy_libp2p_fallback_marker_blocklist_mismatch"));
    assert!(DOC.contains("gate_policy_libp2p_fallback_markers_detected"));
    assert!(DOC.contains("gate_policy_native_libp2p_provider_marker_contract_status_mismatch"));
    assert!(DOC.contains("waiver_status=none|applied"));
    assert!(DOC.contains("waived_reason_codes=none|..."));
    assert!(DOC.contains("remediation=..."));
    assert!(DOC.contains("test_check_kamn_node_main_rs_extraction_threshold.sh"));
    assert!(DOC.contains("fixtures/ci/kamn_node_main_rs_extraction_thresholds.json"));
    assert!(DOC.contains(
        "check_kamn_node_main_rs_extraction_threshold.sh --output-json /tmp/kamn-node-main-rs-extraction-threshold-report.json"
    ));
    assert!(DOC.contains(
        "check_kamn_node_main_rs_extraction_threshold.sh --exception-file .ci/kamn_node_main_rs_extraction_threshold_exception.json --output-json /tmp/kamn-node-main-rs-extraction-threshold-report.json"
    ));
    assert!(DOC.contains("policy_decision=GO|WARN|NO-GO"));
    assert!(DOC.contains("exception_status=not-required|not-provided|applied|invalid|cap-exceeded"));
    assert!(DOC.contains("reason_codes=main_rs_line_count_warn_threshold_exceeded"));
    assert!(DOC.contains("reason_codes=main_rs_line_count_fail_threshold_exceeded"));
    assert!(DOC.contains("reason_codes=main_rs_threshold_exception_applied"));
    assert!(DOC.contains("reason_codes=main_rs_threshold_exception_expired"));
    assert!(DOC.contains("reason_codes=main_rs_threshold_exception_cap_exceeded"));
    assert!(DOC.contains("reason_codes=threshold_order_invalid"));
    assert!(DOC.contains("test_check_kamn_sdk_service_rs_extraction_threshold.sh"));
    assert!(DOC.contains("fixtures/ci/kamn_sdk_service_rs_extraction_thresholds.json"));
    assert!(DOC.contains(
        "check_kamn_sdk_service_rs_extraction_threshold.sh --output-json /tmp/kamn-sdk-service-rs-extraction-threshold-report.json"
    ));
    assert!(DOC.contains(
        "check_kamn_sdk_service_rs_extraction_threshold.sh --exception-file .ci/kamn_sdk_service_rs_extraction_threshold_exception.json --output-json /tmp/kamn-sdk-service-rs-extraction-threshold-report.json"
    ));
    assert!(DOC.contains("reason_codes=service_rs_line_count_warn_threshold_exceeded"));
    assert!(DOC.contains("reason_codes=service_rs_line_count_fail_threshold_exceeded"));
    assert!(DOC.contains("reason_codes=service_rs_threshold_exception_applied"));
    assert!(DOC.contains("reason_codes=service_rs_threshold_exception_expired"));
    assert!(DOC.contains("reason_codes=service_rs_threshold_exception_cap_exceeded"));
    assert!(DOC.contains("test_check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh"));
    assert!(
        DOC.contains("fixtures/ci/kamn_node_runtime_orchestration_rs_extraction_thresholds.json")
    );
    assert!(DOC.contains("check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh --output-json /tmp/kamn-node-runtime-orchestration-rs-extraction-threshold-report.json"));
    assert!(DOC.contains("check_kamn_node_runtime_orchestration_rs_extraction_threshold.sh --exception-file .ci/kamn_node_runtime_orchestration_rs_extraction_threshold_exception.json --output-json /tmp/kamn-node-runtime-orchestration-rs-extraction-threshold-report.json"));
    assert!(DOC.contains("cargo test -p kamn-node --test main_module_extraction_contract"));
    assert!(
        DOC.contains("reason_codes=runtime_orchestration_rs_line_count_warn_threshold_exceeded")
    );
    assert!(
        DOC.contains("reason_codes=runtime_orchestration_rs_line_count_fail_threshold_exceeded")
    );
    assert!(DOC.contains("reason_codes=runtime_orchestration_rs_threshold_exception_applied"));
    assert!(DOC.contains("reason_codes=runtime_orchestration_rs_threshold_exception_expired"));
    assert!(DOC.contains("reason_codes=runtime_orchestration_rs_threshold_exception_cap_exceeded"));
    assert!(DOC.contains("test_check_touched_rust_size_policy.sh"));
    assert!(DOC.contains("fixtures/ci/touched_rust_size_policy_thresholds.json"));
    assert!(DOC.contains("fixtures/ci/touched_rust_size_policy_baseline.json"));
    assert!(DOC.contains(
        "check_touched_rust_size_policy.sh --output-json /tmp/touched-rust-size-policy-report.json"
    ));
    assert!(DOC.contains("reason_codes=touched_rust_size_policy_new_oversized_file"));
    assert!(DOC.contains("reason_codes=touched_rust_size_policy_new_oversized_function"));
    assert!(DOC.contains("reason_codes=touched_rust_size_policy_git_base_unavailable"));
    assert!(DOC.contains("reason_codes=touched_rust_size_policy_threshold_invalid"));
    assert!(DOC.contains("reason_codes=touched_rust_size_policy_baseline_invalid"));
}

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

#[test]
fn doc_contains_sbom_provenance_artifact_generator_contract_markers() {
    assert!(DOC.contains("## SBOM-Provenance Artifact Generator Contract (Issue #4036)"));
    assert!(DOC.contains(
        "cargo run -p kamn-core --bin sbom_provenance_artifact_generator_contract -- --profile baseline --mode dry-run --ci-fast-gate PASS --max-seconds 120 --output-json /tmp/sbom-provenance-baseline.json"
    ));
    assert!(DOC.contains(
        "cargo run -p kamn-core --bin sbom_provenance_artifact_generator_contract -- --profile injected-drift --mode dry-run --ci-fast-gate PASS --max-seconds 120 --output-json /tmp/sbom-provenance-injected-drift.json"
    ));
    assert!(DOC.contains("kamn.runtime.sbom-provenance-artifact-report.v1"));
    assert!(DOC.contains("kamn.runtime.sbom-provenance-artifact-schema.v1"));
    assert!(DOC.contains("kamn.ci.sbom-provenance-artifact-fixture-matrix.v1"));
    assert!(DOC.contains("kamn.runtime.sbom-provenance-artifact-reason-taxonomy.v1"));
    assert!(DOC.contains(
        "sbom_provenance_reason_codes_csv=sbom_provenance_profile_contract_violation,sbom_provenance_runtime_budget_exceeded"
    ));
    assert!(DOC.contains("sbom_schema_version=spdx-2.3"));
    assert!(DOC.contains("provenance_schema_version=slsa-v1"));
    assert!(DOC.contains("release_manifest_required_artifact_id=sbom_provenance"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test sbom_provenance_artifact_generator_contract -- --nocapture"
    ));
    assert!(DOC.contains("Regression: #4036"));
}

#[test]
fn doc_contains_sbom_provenance_release_gonogo_checker_contract_markers() {
    assert!(DOC.contains("## SBOM-Provenance Release Go-No-Go Checker Contract (Issue #4037)"));
    assert!(DOC.contains(
        "python3 scripts/deploy/sbom_provenance_release_gonogo_checker_contract.py --artifact-json /tmp/sbom-provenance-baseline.json --ci-strategy-doc docs/ci/strategy.md --ops-configuration-doc docs/ops/configuration.md --max-seconds 120 --output-json /tmp/sbom-provenance-release-gonogo-checker.json"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_checker_schema_version=kamn.runtime.sbom-provenance-release-gonogo-checker-report.v1"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_checker_reason_taxonomy_version=kamn.runtime.sbom-provenance-release-gonogo-checker-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_checker_reason_codes_csv=sbom_provenance_artifact_marker_missing,sbom_provenance_artifact_marker_invalid,sbom_provenance_artifact_decision_not_go,sbom_provenance_docs_parity_marker_missing,sbom_provenance_runtime_budget_exceeded"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_required_artifact_markers_csv=schema_version,artifact_schema_version,fixture_schema_version,reason_taxonomy_version,release_manifest_required_artifact_id,status,final_decision,reason_code"
    ));
    assert!(DOC.contains(
        "sbom_provenance_release_gonogo_docs_parity_required_markers_csv=sbom_provenance_release_gonogo_checker_schema_version,sbom_provenance_release_gonogo_checker_reason_taxonomy_version,sbom_provenance_release_gonogo_checker_reason_codes_csv,sbom_provenance_release_gonogo_required_artifact_markers_csv"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test sbom_provenance_release_gonogo_checker_contract -- --nocapture"
    ));
    assert!(DOC.contains("Regression: #4037"));
}

#[test]
fn doc_contains_dependency_ci_smoke_advisory_fixture_contract_markers() {
    assert!(DOC.contains("## Dependency CI Smoke Advisory Fixture Contract"));
    assert!(DOC.contains(
        "dependency_ci_smoke_reason_taxonomy_version=kamn.ci.dependency-ci-smoke-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_reason_codes_csv=dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_fixture_schema_version=kamn.ci.dependency-ci-smoke-advisory-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_fixture_path=fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt"
    ));
    assert!(DOC.contains("dependency_ci_smoke_threshold_max_severity=moderate"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test dependency_ci_smoke_advisory_fixture_parser_contract"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_dependency_ci_smoke_advisory_fixture_contract_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #4030"));
}

#[test]
fn doc_contains_dependency_ci_smoke_checker_threshold_parity_markers() {
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_reason_taxonomy_version=kamn.ci.dependency-ci-smoke-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_reason_codes_csv=dependency_advisory_input_empty,dependency_advisory_severity_unknown,dependency_advisory_threshold_exceeded"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_fixture_schema_version=kamn.ci.dependency-ci-smoke-advisory-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_fixture_path=fixtures/ci/dependency_ci_smoke_advisory_fixture_matrix.txt"
    ));
    assert!(DOC.contains("dependency_ci_smoke_checker_threshold_max_severity=moderate"));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_remediation.dependency_advisory_input_empty=provide at least one advisory record from the CI smoke advisory feed before evaluating thresholds"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_remediation.dependency_advisory_severity_unknown=normalize advisory severity to low|moderate|high|critical before evaluation"
    ));
    assert!(DOC.contains(
        "dependency_ci_smoke_checker_remediation.dependency_advisory_threshold_exceeded=reduce dependency advisory severity exposure or update approved threshold with review evidence"
    ));
    assert!(DOC.contains("cargo test -p kamn-core --test dependency_ci_smoke_checker_contract"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_dependency_ci_smoke_checker_threshold_parity_markers -- --exact"
    ));
    assert!(DOC.contains("Regression: #4031"));
}

#[test]
fn doc_contains_cargo_audit_runner_impact_measurement_markers() {
    assert!(
        DOC.contains("cargo_audit_runner_impact_method=github-actions-step-duration+policy-output")
    );
    assert!(DOC.contains("cargo_audit_fast_gate_observed_seconds=156"));
    assert!(DOC.contains("cargo_audit_workspace_premerge_observed_seconds=156"));
    assert!(DOC.contains("cargo_audit_runner_impact_baseline_captured_at=2026-03-05"));
    assert!(DOC.contains("cargo_audit_runner_impact_source_fast_gate_run=22707945091"));
    assert!(DOC.contains("cargo_audit_runner_impact_source_job=65838851460"));
    assert!(DOC.contains("cargo_audit_policy_elapsed_seconds=<float>"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_cargo_audit_runner_impact_measurement_markers -- --exact"
    ));
}

#[test]
fn doc_contains_anti_flake_rerun_policy_reason_taxonomy_markers() {
    assert!(DOC.contains(
        "anti_flake_policy_reason_taxonomy_version=kamn.ci.anti-flake-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "anti_flake_policy_reason_codes_csv=no_active_flaky_entries,active_flaky_entries_within_budget,active_flaky_entries_exceed_max,registry_validation_failed,registry_file_missing,expected_final_decision_mismatch,rerun_policy_fast_workflow_missing,rerun_policy_deep_workflow_missing,rerun_policy_bounded_retry_missing,rerun_policy_invariant_non_retry_missing,rerun_policy_excessive_retry_detected"
    ));
    assert!(DOC.contains("anti_flake_policy_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("anti_flake_policy_reason_class=stable|budgeted|violation"));
    assert!(DOC.contains(
        "check_anti_flake_policy.sh --registry-file .ci/flaky-tests.txt --expected-final-decision GO --max-active-entries 0 --fast-workflow-file .github/workflows/ci-fast-gate.yml --deep-workflow-file .github/workflows/ci-deep-validate.yml --output-json /tmp/anti-flake-policy-report.json"
    ));
}

#[test]
fn doc_contains_merge_gate_reliability_ci_smoke_local_heavy_boundary_markers() {
    assert!(DOC.contains("ci_smoke_local_heavy_boundary_status=verified|violation"));
    assert!(DOC.contains("ci_smoke_performance_report_step_missing"));
    assert!(DOC.contains("ci_smoke_threshold_check_step_missing"));
    assert!(DOC.contains("local_heavy_opt_in_boundary_missing"));
}

#[test]
fn doc_contains_incident_gonogo_boundary_governance_matrix() {
    assert!(DOC.contains("Incident go/no-go convergence and boundary governance"));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract --max-seconds 120"
    ));
    assert!(DOC.contains(
        "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900"
    ));
    assert!(DOC.contains("incident_gonogo_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("incident_gonogo_local_heavy_max_seconds=900"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=opt_in"));
}

#[test]
fn doc_contains_incident_gonogo_boundary_reason_taxonomy_markers() {
    assert!(DOC.contains(
        "incident_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-incident-boundary-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "incident_gonogo_boundary_reason_codes_csv=incident_gonogo_ci_smoke_seconds_exceeded,incident_gonogo_local_heavy_seconds_exceeded,incident_gonogo_local_heavy_opt_in_missing,incident_gonogo_evidence_convergence_mismatch"
    ));
    assert!(DOC.contains("incident_gonogo_ci_smoke_seconds_exceeded"));
    assert!(DOC.contains("incident_gonogo_local_heavy_seconds_exceeded"));
    assert!(DOC.contains("incident_gonogo_local_heavy_opt_in_missing"));
    assert!(DOC.contains("incident_gonogo_evidence_convergence_mismatch"));
    assert!(DOC.contains("Regression: #4471"));
}

#[test]
fn doc_contains_live_gonogo_boundary_governance_matrix() {
    assert!(DOC.contains("Live go/no-go convergence and boundary governance"));
    assert!(DOC.contains(
        "run_manifest_lane.sh --manifest scripts/framework/manifests/deploy_gonogo_evidence_contract_lane.json --phase contract --max-seconds 120"
    ));
    assert!(DOC.contains(
        "KAMN_GONOGO_GATE_LOCAL_OPT_IN=1 bash scripts/deploy/run_gonogo_evidence_deep_lane.sh --max-seconds 900"
    ));
    assert!(DOC.contains("live_gonogo_ci_smoke_max_seconds=120"));
    assert!(DOC.contains("live_gonogo_local_heavy_max_seconds=900"));
    assert!(DOC.contains("ci_smoke_lane_cost_profile=low"));
    assert!(DOC.contains("local_heavy_lane_execution_mode=opt_in"));
}

#[test]
fn doc_contains_live_gonogo_boundary_reason_taxonomy_markers() {
    assert!(DOC.contains(
        "live_gonogo_boundary_reason_taxonomy_version=kamn.release.gonogo-live-boundary-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "live_gonogo_boundary_reason_codes_csv=live_gonogo_ci_smoke_seconds_exceeded,live_gonogo_local_heavy_seconds_exceeded,live_gonogo_local_heavy_opt_in_missing,live_gonogo_evidence_convergence_mismatch"
    ));
    assert!(DOC.contains("live_gonogo_ci_smoke_seconds_exceeded"));
    assert!(DOC.contains("live_gonogo_local_heavy_seconds_exceeded"));
    assert!(DOC.contains("live_gonogo_local_heavy_opt_in_missing"));
    assert!(DOC.contains("live_gonogo_evidence_convergence_mismatch"));
    assert!(DOC.contains(
        "deployment_safety_gate_reason_taxonomy_version=kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("deployment_safety_gate_reason_codes_csv=none|<csv>"));
    assert!(DOC.contains("deployment_safety_gate_reason_codes_value=none|<csv>"));
    assert!(DOC.contains(
        "contracts.deployment_preflight_rotation_reason_taxonomy_version_required=kamn.kolme.local-live-deployment-preflight-rotation-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("contracts.go_no_go_gate_ci_local_boundary_contract_required=true"));
    assert!(DOC.contains(
        "milestone_review_deployment_preflight_policy_rotation_reason_taxonomy_mismatch"
    ));
    assert!(DOC.contains(
        "milestone_review_deployment_preflight_policy_rotation_reason_codes_value_mismatch"
    ));
    assert!(DOC.contains("milestone_review_go_no_go_gate_ci_local_boundary_contract_mismatch"));
    assert!(DOC.contains("Regression: #4442"));
}

#[test]
fn doc_contains_audit_integrity_dry_run_governance_markers() {
    assert!(DOC.contains("### Audit-Integrity Dry-Run Governance Contract"));
    assert!(DOC.contains("bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh"));
    assert!(DOC.contains("bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh"));
    assert!(DOC.contains(
        "bash scripts/deploy/generate_gonogo_evidence_bundle.sh --output-file /tmp/gonogo-audit-integrity.json"
    ));
    assert!(DOC.contains(
        "bash scripts/deploy/check_gonogo_evidence_policy.sh --bundle-file /tmp/gonogo-audit-integrity.json"
    ));
    assert!(DOC.contains(&format!(
        "audit_integrity_reason_taxonomy_version={AUDIT_INTEGRITY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "audit_integrity_reason_codes_csv={AUDIT_INTEGRITY_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains("audit_integrity_reason_codes_value=none|<csv>"));
    assert!(DOC.contains("audit_integrity_gate_final_decision=GO|NO-GO"));
    assert!(DOC.contains("audit integrity gate convergence mismatch"));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test audit_evidence_integrity_contract spec_c01_audit_integrity_generate_bundle_emits_deterministic_go_markers -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_contains_audit_integrity_dry_run_governance_markers -- --exact"
    ));
    for reason_code in audit_integrity_reason_codes() {
        assert!(
            DOC.contains(reason_code),
            "missing audit-integrity fail-closed reason marker {reason_code}"
        );
    }
    assert!(DOC.contains("Regression: #4059"));
}

#[test]
fn doc_contains_lifecycle_ci_dry_run_governance_markers() {
    assert!(DOC.contains("### Lifecycle Artifact CI Dry-Run Governance Contract"));
    assert!(DOC.contains(
        "bash scripts/runtime/generate_lifecycle_artifact_integrity_evidence_bundle.sh --output-file /tmp/lifecycle-artifact-integrity-baseline.json --artifact-id lifecycle-artifact-baseline --lifecycle-stage retention --profile baseline --record-count 42 --ci-fast-gate PASS"
    ));
    assert!(DOC.contains(
        "bash scripts/runtime/run_go_no_go_gate_lane.sh --mode dry-run --max-seconds 120 --output-json /tmp/go-no-go-gate-report.json"
    ));
    assert!(DOC.contains(
        "python3 scripts/ci/check_lifecycle_ci_dry_run_governance.py --lifecycle-artifact-bundle-file /tmp/lifecycle-artifact-integrity-baseline.json --go-no-go-gate-report-file /tmp/go-no-go-gate-report.json --threshold-file fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env --strategy-doc docs/ci/strategy.md --ops-doc docs/ops/configuration.md --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --output-json /tmp/lifecycle-ci-dry-run-governance-report.json"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test lifecycle_ci_dry_run_governance_contract -- --nocapture"
    ));
    assert!(DOC.contains(&format!(
        "lifecycle_ci_dry_run_reason_taxonomy_version={LIFECYCLE_CI_DRY_RUN_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "lifecycle_ci_dry_run_reason_codes_csv={LIFECYCLE_CI_DRY_RUN_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(
        "lifecycle_ci_dry_run_threshold_fixture_path=fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env"
    ));
    assert!(DOC.contains("lifecycle_ci_dry_run_max_seconds=120"));
    assert!(DOC.contains(
        "lifecycle_ci_dry_run_fast_mode_required_entry=cargo test -p kamn-core --test lifecycle_ci_dry_run_governance_contract -- --nocapture"
    ));
    assert!(DOC.contains(
        "lifecycle_ci_dry_run_fast_mode_forbidden_entry=bash \"$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh\" --mode run"
    ));
    assert!(DOC.contains(
        "lifecycle_ci_dry_run_workflow_forbidden_entry=bash scripts/runtime/run_go_no_go_gate_lane.sh --mode run"
    ));
    assert!(DOC.contains("lifecycle_ci_dry_run_remediation_map_version=v1"));
    for reason_code in lifecycle_ci_dry_run_reason_codes() {
        assert!(
            DOC.contains(reason_code),
            "missing lifecycle ci dry-run reason marker {reason_code}"
        );
        assert!(
            DOC.contains(&format!("lifecycle_ci_dry_run_remediation.{reason_code}=")),
            "missing lifecycle ci dry-run remediation marker {reason_code}"
        );
    }
    assert!(DOC.contains("cargo test -p kamn-core --test quota_policy_checker_contract"));
    assert!(DOC.contains("cargo test -p kamn-core --test quota_policy_fixture_parser_contract"));
    assert!(DOC.contains("Regression: #4091"));
}

#[test]
fn doc_contains_retention_policy_checker_taxonomy_contract_markers() {
    assert!(DOC.contains("### Retention Policy Checker Taxonomy Contract"));
    assert!(DOC.contains(
        "retention_policy_checker_reason_taxonomy_version=kamn.runtime.retention-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "retention_policy_checker_reason_codes_csv=retention_domain_unknown,retention_window_non_positive,retention_record_expired"
    ));
    assert!(DOC.contains(
        "retention_policy_checker_fixture_schema_version=kamn.runtime.retention-policy-fixture-matrix.v1"
    ));
    assert!(DOC.contains(
        "retention_policy_checker_fixture_path=fixtures/runtime/retention_policy_fixture_matrix.txt"
    ));
    assert!(DOC.contains("cargo test -p kamn-core --test retention_policy_checker_contract"));
    assert!(DOC.contains("cargo test -p kamn-core --test retention_policy_fixture_parser_contract"));
    assert!(DOC.contains("reason_codes=public_api_surface_fail_threshold_exceeded_unwaived"));
    assert!(DOC.contains("reason_codes=waiver_cap_exceeded"));
    assert!(DOC.contains("set `mitigation_issue=#<issue-id>` and a bounded `max_total_delta`"));
}
