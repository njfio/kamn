use super::*;

#[test]
fn doc_contains_api_version_policy_docs_parity_markers() {
    assert_api_version_doc_headers();
    assert_api_version_doc_paths();
    assert_api_version_doc_command();
    assert!(DOC.contains("Regression: #4041"));
}

#[test]
fn doc_enforces_api_version_policy_docs_parity_matches_source_taxonomy() {
    assert_api_version_source_markers();
    assert_api_version_strategy_markers();
    assert_api_version_ops_markers();
}

#[test]
fn doc_enforces_api_version_policy_reason_codes_non_empty() {
    for reason_code in api_version_policy_reason_codes() {
        assert_api_version_reason_code_present(reason_code);
    }
}

fn assert_api_version_doc_headers() {
    assert!(DOC.contains("### API Version-Policy Contract"));
    assert!(DOC.contains(&format!(
        "api_version_policy_reason_taxonomy_version={API_VERSION_POLICY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "api_version_policy_reason_codes_csv={API_VERSION_POLICY_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "api_version_policy_fixture_schema_version={API_VERSION_POLICY_FIXTURE_SCHEMA_VERSION}"
    )));
}

fn assert_api_version_doc_paths() {
    assert!(DOC.contains(&format!("api_version_policy_fixture_path={API_VERSION_POLICY_FIXTURE_PATH}")));
    assert!(DOC.contains(&format!(
        "api_version_policy_required_row_ids_csv={API_VERSION_POLICY_REQUIRED_ROW_IDS_CSV}"
    )));
    assert!(DOC.contains("api_version_policy_ops_doc_path=docs/ops/configuration.md"));
    assert!(DOC.contains("api_version_policy_strategy_doc_path=docs/ci/strategy.md"));
}

fn assert_api_version_doc_command() {
    assert!(DOC.contains(
        "cargo test -p kamn-core --test api_version_policy_contract integration_api_version_policy_contract_lane_composes_policy_and_docs_parity -- --exact"
    ));
}

fn assert_api_version_source_markers() {
    assert!(API_VERSION_POLICY_CONTRACT_SOURCE.contains(
        "REASON_TAXONOMY_VERSION = \"kamn.runtime.api-version-policy-reason-taxonomy.v1\""
    ));
    assert!(API_VERSION_POLICY_CONTRACT_SOURCE.contains("REASON_CODES_CSV = \",\".join("));
    assert!(API_VERSION_POLICY_CONTRACT_SOURCE.contains(
        "FIXTURE_SCHEMA = \"kamn.runtime.api-version-policy-fixture-matrix.v1\""
    ));
}

fn assert_api_version_strategy_markers() {
    assert!(DOC.contains(&format!(
        "api_version_policy_reason_taxonomy_version={API_VERSION_POLICY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "api_version_policy_reason_codes_csv={API_VERSION_POLICY_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "api_version_policy_fixture_schema_version={API_VERSION_POLICY_FIXTURE_SCHEMA_VERSION}"
    )));
    assert!(DOC.contains(&format!("api_version_policy_fixture_path={API_VERSION_POLICY_FIXTURE_PATH}")));
    assert!(DOC.contains(&format!(
        "api_version_policy_required_row_ids_csv={API_VERSION_POLICY_REQUIRED_ROW_IDS_CSV}"
    )));
}

fn assert_api_version_ops_markers() {
    assert!(OPS_DOC.contains(&format!(
        "api_version_policy_reason_taxonomy_version={API_VERSION_POLICY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "api_version_policy_reason_codes_csv={API_VERSION_POLICY_REASON_CODES_CSV}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "api_version_policy_fixture_schema_version={API_VERSION_POLICY_FIXTURE_SCHEMA_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!("api_version_policy_fixture_path={API_VERSION_POLICY_FIXTURE_PATH}")));
    assert!(OPS_DOC.contains(&format!(
        "api_version_policy_required_row_ids_csv={API_VERSION_POLICY_REQUIRED_ROW_IDS_CSV}"
    )));
}

fn assert_api_version_reason_code_present(reason_code: &str) {
    assert!(!reason_code.trim().is_empty(), "reason code entries must stay non-empty");
    assert!(DOC.contains(reason_code), "ci strategy docs missing api version-policy reason code marker: {reason_code}");
    assert!(OPS_DOC.contains(reason_code), "ops docs missing api version-policy reason code marker: {reason_code}");
}
