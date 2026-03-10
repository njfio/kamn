use super::*;

#[test]
fn doc_contains_service_api_scope_policy_docs_parity_markers() {
    assert_scope_policy_doc_headers();
    assert_scope_policy_doc_paths();
    assert_scope_policy_doc_commands();
    assert!(DOC.contains("Regression: #4056"));
}

#[test]
fn doc_enforces_service_api_scope_policy_docs_parity_matches_source_taxonomy() {
    assert_scope_policy_source_markers();
    assert_scope_policy_strategy_markers();
    assert_scope_policy_ops_markers();
}

#[test]
fn doc_enforces_service_api_scope_policy_remediation_markers_cover_reason_codes() {
    for reason_code in service_api_scope_policy_reason_codes() {
        assert_scope_policy_remediation_markers(reason_code);
    }
}

fn assert_scope_policy_doc_headers() {
    assert!(DOC.contains("### Service API Scope Policy Checker Contract"));
    assert!(DOC.contains(&format!(
        "service_api_scope_policy_reason_taxonomy_version={SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_scope_policy_reason_codes_csv={SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_scope_policy_fixture_schema_version={SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION}"
    )));
}

fn assert_scope_policy_doc_paths() {
    assert!(DOC.contains(&format!(
        "service_api_scope_policy_fixture_path={SERVICE_API_SCOPE_POLICY_FIXTURE_PATH}"
    )));
    assert!(DOC.contains("service_api_scope_policy_ops_doc_path=docs/ops/configuration.md"));
    assert!(DOC.contains("service_api_scope_policy_strategy_doc_path=docs/ci/strategy.md"));
    assert!(DOC.contains("service_api_scope_policy_remediation_map_version=v1"));
}

fn assert_scope_policy_doc_commands() {
    assert!(DOC.contains(
        "cargo test -p kamn-node main_tests::service_api_endpoint_tests::unit_service_api_scope_policy_fixture_parser_contract -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node main_tests::service_api_endpoint_tests::functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-node main_tests::service_api_endpoint_tests::integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_scope_policy_docs_parity_matches_source_taxonomy -- --exact"
    ));
    assert!(DOC.contains(
        "cargo test -p kamn-core --test ci_strategy_docs doc_enforces_service_api_scope_policy_remediation_markers_cover_reason_codes -- --exact"
    ));
}

fn assert_scope_policy_source_markers() {
    assert!(SERVICE_API_ENDPOINT_SOURCE
        .contains("pub(crate) const SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION: &str ="));
    assert!(SERVICE_API_ENDPOINT_SOURCE.contains("pub(crate) const SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV: &str ="));
    assert!(SERVICE_API_ENDPOINT_SOURCE
        .contains("pub(crate) const SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION: &str ="));
    assert!(SERVICE_API_ENDPOINT_SOURCE.contains(SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION));
    assert!(SERVICE_API_ENDPOINT_SOURCE.contains(SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV));
    assert!(SERVICE_API_ENDPOINT_SOURCE.contains(SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION));
}

fn assert_scope_policy_strategy_markers() {
    assert!(DOC.contains(&format!(
        "service_api_scope_policy_reason_taxonomy_version={SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_scope_policy_reason_codes_csv={SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "service_api_scope_policy_fixture_schema_version={SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION}"
    )));
    assert!(DOC.contains(&format!("service_api_scope_policy_fixture_path={SERVICE_API_SCOPE_POLICY_FIXTURE_PATH}")));
}

fn assert_scope_policy_ops_markers() {
    assert!(OPS_DOC.contains(&format!(
        "service_api_scope_policy_reason_taxonomy_version={SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "service_api_scope_policy_reason_codes_csv={SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "service_api_scope_policy_fixture_schema_version={SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!("service_api_scope_policy_fixture_path={SERVICE_API_SCOPE_POLICY_FIXTURE_PATH}")));
}

fn assert_scope_policy_remediation_markers(reason_code: &str) {
    assert!(
        DOC.contains(&format!("service_api_scope_policy_remediation.{reason_code}=")),
        "missing scope policy remediation marker for {reason_code}"
    );
    assert!(
        OPS_DOC.contains(&format!("service_api_scope_policy_remediation.{reason_code}=")),
        "ops docs missing scope policy remediation marker for {reason_code}"
    );
}
