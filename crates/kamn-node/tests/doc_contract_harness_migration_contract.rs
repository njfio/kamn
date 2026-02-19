use std::path::Path;

const ARCH_DOC_HARNESS_SOURCE: &str = include_str!("./architecture_navigation_docs.rs");

fn repo_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

#[test]
fn regression_doc_contract_harness_declares_data_driven_case_matrix() {
    assert!(
        ARCH_DOC_HARNESS_SOURCE.contains("DOC_CONTRACT_CASES"),
        "consolidated doc harness must declare DOC_CONTRACT_CASES case matrix"
    );
}

#[test]
fn regression_doc_contract_harness_migrates_selected_legacy_doc_tests() {
    let migrated_files = [
        "tests/kolme_devnet_ops_docs.rs",
        "tests/node_module_map_docs.rs",
        "tests/observability_streaming_docs.rs",
    ];

    for relative in migrated_files {
        let absolute = repo_path(relative);
        assert!(
            !Path::new(absolute.as_str()).exists(),
            "legacy docs-contract file should be migrated into harness: {relative}"
        );
    }
}
