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

#[path = "ci_strategy_docs/residual_root_contract_tests.rs"]
mod residual_root_contract_tests;
