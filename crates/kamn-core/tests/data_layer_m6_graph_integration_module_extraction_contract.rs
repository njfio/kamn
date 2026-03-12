use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/data_layer_m6_graph_integration.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "src/data_layer_m6_graph_integration/models.rs",
    "src/data_layer_m6_graph_integration/registry.rs",
    "src/data_layer_m6_graph_integration/trust_query.rs",
    "src/data_layer_m6_graph_integration/export.rs",
    "src/data_layer_m6_graph_integration/support.rs",
    "src/data_layer_m6_graph_integration/tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod export;",
    "mod models;",
    "mod registry;",
    "mod support;",
    "mod tests;",
    "mod trust_query;",
];
const MOVED_MARKERS: &[&str] = &[
    "pub struct DataLayerM6GraphNodeInput {",
    "pub struct DataLayerM6GraphRegistry {",
    "pub fn register_node(",
    "pub fn query_trust_propagation(",
    "pub fn export_portable_edge_projection_scoped(",
    "pub enum DataLayerM6GraphIntegrationError {",
    "fn validate_weight(",
    "mod tests {",
];

#[test]
fn data_layer_m6_graph_integration_root_is_extracted() {
    let root = fs::read_to_string(repo_path(ROOT)).expect("read root");
    assert_root_shell_budget(&root);
    assert_required_markers(&root);
    assert_moved_markers_removed(&root);
    assert_module_files_exist_and_fit_budget();
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_root_shell_budget(root: &str) {
    let lines = root.lines().count();
    assert!(
        lines <= ROOT_CAP,
        "expected {ROOT} <= {ROOT_CAP} lines after extraction, found {lines}"
    );
}

fn assert_required_markers(root: &str) {
    for marker in REQUIRED_MARKERS {
        assert!(root.contains(marker), "missing root module marker: {marker}");
    }
}

fn assert_moved_markers_removed(root: &str) {
    for marker in MOVED_MARKERS {
        assert!(
            !root.contains(marker),
            "moved marker still present in root: {marker}"
        );
    }
}

fn assert_module_files_exist_and_fit_budget() {
    for path in MODULE_FILES {
        let full = repo_path(path);
        assert!(full.exists(), "missing extracted module: {}", full.display());
        let lines = fs::read_to_string(&full)
            .expect("read module")
            .lines()
            .count();
        assert!(
            lines <= MODULE_CAP,
            "extracted module exceeds {MODULE_CAP} lines: {}",
            full.display()
        );
    }
}
