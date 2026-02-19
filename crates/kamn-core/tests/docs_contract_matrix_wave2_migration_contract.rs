use std::path::Path;

fn repo_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

#[test]
fn regression_wave2_docs_contract_harness_exists_with_case_matrix() {
    let harness_source =
        std::fs::read_to_string(repo_path("tests/docs_contract_matrix_wave2_harness.rs"))
            .expect("docs_contract_matrix_wave2_harness.rs should exist");
    assert!(
        harness_source.contains("DOC_CONTRACT_CASES"),
        "wave2 docs harness must define DOC_CONTRACT_CASES matrix"
    );

    let required_case_ids = [
        "tls_feature_gate_ci_contract_markers",
        "persistence_live_validation_roadmap_contract_markers",
        "data_governance_retention_contract_markers",
        "key_management_encryption_contract_markers",
        "sdk_example_fixture_drift_contract_markers",
        "group_sender_key_rotation_contract_markers",
        "incident_readiness_contract_markers",
        "python_sdk_beta_contract_markers",
        "typescript_sdk_beta_contract_markers",
        "service_marketplace_contract_markers",
        "shell_surface_governance_pr_template_contract_markers",
        "shell_surface_governance_ci_strategy_contract_markers",
        "testing_structure_contract_markers",
    ];

    for case_id in required_case_ids {
        assert!(
            harness_source.contains(case_id),
            "wave2 docs harness should include migrated case ID: {case_id}"
        );
    }
}

#[test]
fn regression_wave2_legacy_singleton_doc_suites_are_retired() {
    // Regression: #5217
    let retired_files = [
        "tests/tls_feature_gate_ci_docs.rs",
        "tests/persistence_live_validation_roadmap_docs.rs",
        "tests/data_governance_retention_docs.rs",
        "tests/key_management_and_encryption_docs.rs",
        "tests/sdk_example_fixture_drift_docs.rs",
        "tests/group_sender_key_rotation_docs.rs",
        "tests/incident_readiness_docs.rs",
        "tests/python_sdk_beta_docs.rs",
        "tests/typescript_sdk_beta_docs.rs",
        "tests/service_marketplace_docs.rs",
        "tests/shell_surface_governance_docs.rs",
        "tests/testing_structure_docs.rs",
    ];

    for relative in retired_files {
        let absolute = repo_path(relative);
        assert!(
            !Path::new(absolute.as_str()).exists(),
            "legacy docs-contract singleton suite should be retired after wave2 migration: {relative}"
        );
    }
}
