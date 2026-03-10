use super::*;
use super::service_api_policy_support::assert_reason_code_present_in_docs_and_ops;

#[test]
fn doc_contains_runtime_request_response_schema_compatibility_contract_lane_ci_mode_markers() {
    assert_request_response_doc_headers();
    assert_request_response_doc_paths();
    assert_request_response_doc_command();
    assert!(DOC.contains("Regression: #4042"));
}

#[test]
fn doc_enforces_request_response_schema_compatibility_docs_parity_matches_source_taxonomy() {
    assert_request_response_source_markers();
    assert_request_response_strategy_markers();
    assert_request_response_ops_markers();
}

#[test]
fn doc_enforces_request_response_schema_compatibility_reason_codes_non_empty() {
    for reason_code in request_response_schema_compatibility_reason_codes() {
        assert_request_response_reason_code_present(reason_code);
    }
}

fn assert_request_response_doc_headers() {
    assert!(DOC.contains("### Request-Response Schema Compatibility Contract"));
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_reason_taxonomy_version={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_reason_codes_csv={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_fixture_schema_version={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_FIXTURE_SCHEMA_VERSION}"
    )));
}

fn assert_request_response_doc_paths() {
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_fixture_path={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_FIXTURE_PATH}"
    )));
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_required_row_ids_csv={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REQUIRED_ROW_IDS_CSV}"
    )));
    assert!(DOC.contains("request_response_schema_compatibility_ops_doc_path=docs/ops/configuration.md"));
    assert!(DOC.contains("request_response_schema_compatibility_strategy_doc_path=docs/ci/strategy.md"));
}

fn assert_request_response_doc_command() {
    assert!(DOC.contains(
        "cargo test -p kamn-core --test request_response_schema_compatibility_contract integration_request_response_schema_compatibility_contract_lane_composes_policy_and_docs_parity -- --exact"
    ));
}

fn assert_request_response_source_markers() {
    assert!(REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_CONTRACT_SOURCE.contains(
        "REASON_TAXONOMY_VERSION = \"kamn.runtime.request-response-schema-compatibility-reason-taxonomy.v1\""
    ));
    assert!(REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_CONTRACT_SOURCE.contains("REASON_CODES_CSV = \",\".join("));
    assert!(REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_CONTRACT_SOURCE.contains(
        "FIXTURE_SCHEMA = \"kamn.runtime.request-response-schema-compatibility-fixture-matrix.v1\""
    ));
}

fn assert_request_response_strategy_markers() {
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_reason_taxonomy_version={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_reason_codes_csv={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REASON_CODES_CSV}"
    )));
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_fixture_schema_version={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_FIXTURE_SCHEMA_VERSION}"
    )));
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_fixture_path={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_FIXTURE_PATH}"
    )));
    assert!(DOC.contains(&format!(
        "request_response_schema_compatibility_required_row_ids_csv={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REQUIRED_ROW_IDS_CSV}"
    )));
}

fn assert_request_response_ops_markers() {
    assert!(OPS_DOC.contains(&format!(
        "request_response_schema_compatibility_reason_taxonomy_version={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REASON_TAXONOMY_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "request_response_schema_compatibility_reason_codes_csv={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REASON_CODES_CSV}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "request_response_schema_compatibility_fixture_schema_version={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_FIXTURE_SCHEMA_VERSION}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "request_response_schema_compatibility_fixture_path={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_FIXTURE_PATH}"
    )));
    assert!(OPS_DOC.contains(&format!(
        "request_response_schema_compatibility_required_row_ids_csv={REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_REQUIRED_ROW_IDS_CSV}"
    )));
}

fn assert_request_response_reason_code_present(reason_code: &str) {
    assert_reason_code_present_in_docs_and_ops(reason_code, "schema-compatibility");
}
