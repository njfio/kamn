use super::*;

#[test]
fn doc_contains_service_api_tenant_isolation_matrix_docs_parity_markers() {
    assert_tenant_isolation_doc_headers();
    assert_tenant_isolation_doc_paths();
    assert_tenant_isolation_doc_commands();
}

#[test]
fn doc_enforces_service_api_tenant_isolation_matrix_docs_parity_matches_source_taxonomy() {
    assert_tenant_isolation_source_markers();
    assert_tenant_isolation_strategy_markers();
    assert_tenant_isolation_ops_markers();
}

#[test]
fn doc_enforces_service_api_tenant_isolation_matrix_reason_codes_non_empty() {
    for reason_code in service_api_tenant_isolation_reason_codes() {
        assert_tenant_isolation_reason_code_present(reason_code);
    }
}

fn assert_tenant_isolation_doc_headers() {
    assert!(DOC.contains("### Service API Tenant-Isolation Matrix Contract"));
    assert!(DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_reason_taxonomy_version={SERVICE_API_TENANT_ISOLATION_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_reason_codes_csv={SERVICE_API_TENANT_ISOLATION_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_matrix_schema_version={SERVICE_API_TENANT_ISOLATION_MATRIX_SCHEMA_VERSION}"
    )));
}

fn assert_tenant_isolation_doc_paths() {
    assert!(DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_required_row_ids_csv={SERVICE_API_TENANT_ISOLATION_REQUIRED_ROW_IDS_CSV}"
    )));
    assert!(DOC.contains("service_api_tenant_isolation_matrix_ops_doc_path=docs/ops/configuration.md"));
    assert!(DOC.contains("service_api_tenant_isolation_matrix_strategy_doc_path=docs/ci/strategy.md"));
}

fn assert_tenant_isolation_doc_commands() {
    assert!(DOC.contains(
        "cargo test -p kamn-core --test service_api_tenant_isolation_matrix_contract integration_tenant_isolation_matrix_contract_lane_composes_lane_policy_and_docs_parity -- --exact"
    ));
    assert!(DOC.contains("Regression: #4058"));
}

fn assert_tenant_isolation_source_markers() {
    assert!(SERVICE_API_TENANT_ISOLATION_CONTRACT_SOURCE.contains(&format!(
        "REASON_TAXONOMY_VERSION = \"{SERVICE_API_TENANT_ISOLATION_REASON_TAXONOMY_VERSION}\""
    )));
    assert!(SERVICE_API_TENANT_ISOLATION_CONTRACT_SOURCE.contains("REASON_CODES_CSV = \",\".join("));
    assert!(SERVICE_API_TENANT_ISOLATION_CONTRACT_SOURCE.contains(&format!(
        "MATRIX_SCHEMA = \"{SERVICE_API_TENANT_ISOLATION_MATRIX_SCHEMA_VERSION}\""
    )));
}

fn assert_tenant_isolation_strategy_markers() {
    assert!(DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_reason_taxonomy_version={SERVICE_API_TENANT_ISOLATION_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_reason_codes_csv={SERVICE_API_TENANT_ISOLATION_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_matrix_schema_version={SERVICE_API_TENANT_ISOLATION_MATRIX_SCHEMA_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_required_row_ids_csv={SERVICE_API_TENANT_ISOLATION_REQUIRED_ROW_IDS_CSV}"
    )));
}

fn assert_tenant_isolation_ops_markers() {
    assert!(OPS_DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_reason_taxonomy_version={SERVICE_API_TENANT_ISOLATION_REASON_TAXONOMY_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_reason_codes_csv={SERVICE_API_TENANT_ISOLATION_REASON_CODES_CSV}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_matrix_schema_version={SERVICE_API_TENANT_ISOLATION_MATRIX_SCHEMA_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "service_api_tenant_isolation_matrix_required_row_ids_csv={SERVICE_API_TENANT_ISOLATION_REQUIRED_ROW_IDS_CSV}"
    )));
}

fn assert_tenant_isolation_reason_code_present(reason_code: &str) {
    assert!(!reason_code.trim().is_empty(), "reason code entries must stay non-empty");
    assert!(DOC.contains(reason_code), "ci strategy docs missing tenant-isolation reason code marker: {reason_code}");
    assert!(OPS_DOC.contains(reason_code), "ops docs missing tenant-isolation reason code marker: {reason_code}");
}
