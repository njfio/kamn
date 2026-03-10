use super::support::assert_runtime_lane_contract_markers;

#[test]
fn doc_contains_runtime_service_api_validation_negative_matrix_contract_lane_ci_mode_markers() {
    assert_runtime_lane_contract_markers(
        "## Runtime Service API Validation Negative-Matrix Contract Lane",
        &[
            "validate_service_api_validation_negative_matrix_live.sh --mode dry-run --output-json /tmp/service-api-validation-negative-matrix-live-summary.json",
            "KAMN_LOCAL_VALIDATION_NEGATIVE_MATRIX_OPT_IN=1 bash scripts/runtime/validate_service_api_validation_negative_matrix_live.sh --mode run --output-json /tmp/service-api-validation-negative-matrix-live-summary.json",
            "check_service_api_validation_negative_matrix_live_policy.sh --report-file /tmp/service-api-validation-negative-matrix-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-validation-negative-matrix-policy.json",
            "validate_service_api_validation_negative_matrix_live_contract_lane.sh --output-json /tmp/service-api-validation-negative-matrix-contract-lane-report.json --policy-output-json /tmp/service-api-validation-negative-matrix-policy.json",
            "test_validate_service_api_validation_negative_matrix_live_contract_lane.sh",
            "test_check_service_api_validation_negative_matrix_live_policy.sh",
        ],
        "service api validation negative-matrix contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &["service_api_validation_negative_matrix_policy_marker_missing:replay_guard_status"],
        "service api validation negative matrix",
    );
}

#[test]
fn doc_contains_runtime_service_api_tenant_isolation_matrix_contract_lane_ci_mode_markers() {
    assert_runtime_lane_contract_markers(
        "## Runtime Service API Tenant-Isolation Matrix Contract Lane",
        &[
            "validate_service_api_tenant_isolation_matrix_live.sh --mode dry-run --output-json /tmp/service-api-tenant-isolation-matrix-live-summary.json",
            "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_OPT_IN=1 bash scripts/runtime/validate_service_api_tenant_isolation_matrix_live.sh --mode run --output-json /tmp/service-api-tenant-isolation-matrix-live-summary.json",
            "check_service_api_tenant_isolation_matrix_live_policy.sh --report-file /tmp/service-api-tenant-isolation-matrix-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-tenant-isolation-matrix-policy.json",
            "validate_service_api_tenant_isolation_matrix_live_contract_lane.sh --output-json /tmp/service-api-tenant-isolation-matrix-contract-lane-report.json --policy-output-json /tmp/service-api-tenant-isolation-matrix-policy.json",
            "cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract unit_tenant_isolation_matrix_lane_dry_run_emits_deterministic_schema_and_markers -- --exact",
            "cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract integration_tenant_isolation_matrix_contract_lane_composes_lane_policy_and_docs_parity -- --exact",
        ],
        "service api tenant-isolation matrix run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &["service_api_tenant_isolation_policy_matrix_row_status_mismatch"],
        "service api tenant isolation",
    );
}

#[test]
fn doc_contains_runtime_api_version_policy_contract_lane_ci_mode_markers() {
    assert_runtime_lane_contract_markers(
        "## Runtime API Version-Policy Contract Lane",
        &[
            "validate_api_version_policy_live.sh --mode dry-run --output-json /tmp/api-version-policy-live-summary.json",
            "KAMN_API_VERSION_POLICY_OPT_IN=1 bash scripts/runtime/validate_api_version_policy_live.sh --mode run --output-json /tmp/api-version-policy-live-summary.json",
            "check_api_version_policy_live_policy.sh --report-file /tmp/api-version-policy-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/api-version-policy-live-policy.json",
            "validate_api_version_policy_live_contract_lane.sh --output-json /tmp/api-version-policy-contract-lane-report.json --policy-output-json /tmp/api-version-policy-live-policy.json",
            "cargo test -p kamn-core --test api_version_policy_contract unit_api_version_policy_lane_dry_run_emits_deterministic_markers -- --exact",
            "cargo test -p kamn-core --test api_version_policy_contract integration_api_version_policy_contract_lane_composes_policy_and_docs_parity -- --exact",
        ],
        "api version-policy run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &["api_version_policy_fixture_row_status_mismatch"],
        "api version policy",
    );
}
