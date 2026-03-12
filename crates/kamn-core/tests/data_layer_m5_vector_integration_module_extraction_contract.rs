use std::fs;
use std::path::{Path, PathBuf};

const ROOT_MAX_LINES: usize = 180;
const CHILD_MAX_LINES: usize = 200;

#[test]
fn data_layer_m5_vector_integration_root_is_extracted() {
    let src_root = source_root();
    let root = src_root.join("data_layer_m5_vector_integration.rs");
    let root_source = fs::read_to_string(&root).expect("read root source");

    assert_root_budget(&root, &root_source);
    assert_module_layout(&src_root);
    assert_root_markers(&root_source);
    assert_moved_markers_removed(&root_source);
}

fn source_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn assert_root_budget(root: &Path, root_source: &str) {
    let root_lines = root_source.lines().count();
    assert!(
        root_lines <= ROOT_MAX_LINES,
        "expected {} <= {} lines after extraction, found {}",
        root.display(),
        ROOT_MAX_LINES,
        root_lines
    );
}

fn assert_module_layout(src_root: &Path) {
    for relative in expected_modules() {
        let path = src_root.join(relative);
        assert!(path.exists(), "missing extracted module: {}", path.display());
        let line_count = fs::read_to_string(&path).expect("read extracted module").lines().count();
        assert!(
            line_count <= CHILD_MAX_LINES,
            "expected {} <= {} lines, found {}",
            path.display(),
            CHILD_MAX_LINES,
            line_count
        );
    }
}

fn assert_root_markers(root_source: &str) {
    for marker in expected_markers() {
        assert!(root_source.contains(marker), "expected root shell marker: {marker}");
    }
}

fn assert_moved_markers_removed(root_source: &str) {
    for marker in moved_markers() {
        assert!(!root_source.contains(marker), "root still contains moved marker: {marker}");
    }
}

fn expected_modules() -> &'static [&'static str] {
    &[
        "data_layer_m5_vector_integration/models.rs",
        "data_layer_m5_vector_integration/registry.rs",
        "data_layer_m5_vector_integration/query.rs",
        "data_layer_m5_vector_integration/analytics.rs",
        "data_layer_m5_vector_integration/support.rs",
        "data_layer_m5_vector_integration/tests.rs",
    ]
}

fn expected_markers() -> &'static [&'static str] {
    &[
        "mod analytics;",
        "mod models;",
        "mod query;",
        "mod registry;",
        "mod support;",
        "#[cfg(test)]",
        "mod tests;",
    ]
}

fn moved_markers() -> &'static [&'static str] {
    &[
        "pub struct DataLayerM5EmbeddingRecordInput",
        "impl DataLayerM5EmbeddingRegistry",
        "fn compute_embedding_record_hash",
        "fn unit_data_layer_m5_append_and_semantic_query_rank_results",
    ]
}
