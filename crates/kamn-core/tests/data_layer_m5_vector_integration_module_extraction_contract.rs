use std::fs;
use std::path::Path;

const ROOT: &str = "crates/kamn-core/src/data_layer_m5_vector_integration.rs";
const ROOT_BUDGET: usize = 180;
const CHILD_BUDGET: usize = 200;
const EXPECTED_MODULES: &[&str] = &[
    "crates/kamn-core/src/data_layer_m5_vector_integration/models.rs",
    "crates/kamn-core/src/data_layer_m5_vector_integration/registry.rs",
    "crates/kamn-core/src/data_layer_m5_vector_integration/query.rs",
    "crates/kamn-core/src/data_layer_m5_vector_integration/analytics.rs",
    "crates/kamn-core/src/data_layer_m5_vector_integration/support.rs",
    "crates/kamn-core/src/data_layer_m5_vector_integration/tests.rs",
];
const ROOT_MARKERS: &[&str] = &[
    "mod analytics;",
    "mod models;",
    "mod query;",
    "mod registry;",
    "mod support;",
    "#[cfg(test)]",
    "mod tests;",
];
const MOVED_MARKERS: &[&str] = &[
    "pub struct DataLayerM5EmbeddingRecordInput",
    "impl DataLayerM5EmbeddingRegistry",
    "fn compute_embedding_record_hash",
    "fn unit_data_layer_m5_append_and_semantic_query_rank_results",
];

#[test]
fn data_layer_m5_vector_integration_root_is_extracted() {
    let root = fs::read_to_string(ROOT).expect("root should exist");
    let root_lines = root.lines().count();
    assert!(
        root_lines <= ROOT_BUDGET,
        "expected {ROOT} to be <= {ROOT_BUDGET} lines, got {root_lines}"
    );

    for marker in ROOT_MARKERS {
        assert!(root.contains(marker), "missing root marker: {marker}");
    }

    for marker in MOVED_MARKERS {
        assert!(
            !root.contains(marker),
            "root still contains moved marker: {marker}"
        );
    }

    for module in EXPECTED_MODULES {
        let path = Path::new(module);
        assert!(path.exists(), "expected module to exist: {module}");
        let content = fs::read_to_string(path).expect("module should be readable");
        let lines = content.lines().count();
        assert!(
            lines <= CHILD_BUDGET,
            "expected {module} to be <= {CHILD_BUDGET} lines, got {lines}"
        );
    }
}
