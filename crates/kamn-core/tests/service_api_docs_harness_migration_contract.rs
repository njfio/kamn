use std::path::Path;

fn repo_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

#[test]
fn regression_service_api_docs_harness_declares_case_matrix() {
    let harness_source =
        std::fs::read_to_string(repo_path("tests/service_api_docs_contract_harness.rs"))
            .expect("service_api_docs_contract_harness.rs should exist");
    assert!(
        harness_source.contains("DOC_CONTRACT_CASES"),
        "service API docs harness must declare DOC_CONTRACT_CASES matrix"
    );
    let required_case_ids = [
        "service_api_invalid_frame_handling_matrix_markers",
        "service_api_async_lifecycle_rejection_taxonomy_markers",
    ];
    for case_id in required_case_ids {
        assert!(
            harness_source.contains(case_id),
            "service API docs harness should include migrated case ID: {case_id}"
        );
    }
}

#[test]
fn regression_service_api_legacy_doc_suites_are_retired() {
    let retired_files = [
        "tests/service_api_contract_docs.rs",
        "tests/service_api_lifecycle_contract_docs.rs",
    ];
    for relative in retired_files {
        let absolute = repo_path(relative);
        assert!(
            !Path::new(absolute.as_str()).exists(),
            "legacy docs-contract suite should be retired after migration: {relative}"
        );
    }
}
