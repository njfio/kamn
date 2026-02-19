fn repo_path(relative: &str) -> String {
    format!("{}/{}", env!("CARGO_MANIFEST_DIR"), relative)
}

fn read_source(relative: &str) -> String {
    std::fs::read_to_string(repo_path(relative))
        .unwrap_or_else(|error| panic!("failed reading {relative}: {error}"))
}

#[test]
fn regression_wave2_harness_inventory_guards_remain_declared() {
    // Regression: #5217
    let harness = read_source("tests/docs_contract_matrix_wave2_harness.rs");

    assert!(
        harness.contains("assert_eq!(DOC_CONTRACT_CASES.len(), 13);"),
        "wave2 harness must keep explicit case-count guard"
    );
    assert!(
        harness.contains("assert_eq!(total_marker_count, 89);"),
        "wave2 harness must keep explicit marker-count guard"
    );
    assert!(
        harness.contains(".all(|case| !case.required_markers.is_empty())"),
        "wave2 harness must keep non-empty marker inventory guard"
    );
}

#[test]
fn regression_wave2_migration_inventory_retains_retired_file_guard() {
    let migration = read_source("tests/docs_contract_matrix_wave2_migration_contract.rs");

    let retired_inventory_count = migration
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("\"tests/") && line.ends_with("_docs.rs\","))
        .count();
    assert_eq!(
        retired_inventory_count, 12,
        "wave2 migration contract should retain 12-file retired singleton inventory"
    );
}
