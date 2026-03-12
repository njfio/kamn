use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/data_layer_m7_timeseries_telemetry.rs";
const MODULES: &[&str] = &[
    "src/data_layer_m7_timeseries_telemetry/aggregate_queries.rs",
    "src/data_layer_m7_timeseries_telemetry/aggregates.rs",
    "src/data_layer_m7_timeseries_telemetry/models.rs",
    "src/data_layer_m7_timeseries_telemetry/registry.rs",
    "src/data_layer_m7_timeseries_telemetry/billing.rs",
    "src/data_layer_m7_timeseries_telemetry/errors.rs",
    "src/data_layer_m7_timeseries_telemetry/observability.rs",
    "src/data_layer_m7_timeseries_telemetry/owner_reports.rs",
    "src/data_layer_m7_timeseries_telemetry/support.rs",
    "src/data_layer_m7_timeseries_telemetry/tests.rs",
];
const ROOT_MARKERS: &[&str] = &[
    "mod aggregate_queries;",
    "mod aggregates;",
    "mod billing;",
    "mod errors;",
    "mod models;",
    "mod observability;",
    "mod owner_reports;",
    "mod registry;",
    "mod support;",
    "#[cfg(test)]",
    "mod tests;",
];
const ROOT_EXCLUDES: &[&str] = &[
    "pub struct DataLayerM7TelemetryRegistry",
    "pub enum DataLayerM7TimeseriesError",
    "fn project_m7_owner_billing_daily_projection(",
    "fn project_m7_observability_projection(",
];
const ROOT_MAX_LINES: usize = 180;
const MODULE_MAX_LINES: usize = 200;

#[test]
fn data_layer_m7_timeseries_telemetry_root_is_extracted() {
    let root = read_root();
    assert_root_budget(&root);
    assert_root_markers(&root);
    assert_root_excludes(&root);
    assert_module_files_exist_and_fit();
}

fn read_root() -> String {
    fs::read_to_string(repo_path(ROOT)).expect("read root")
}

fn assert_root_budget(root: &str) {
    let count = line_count(root);
    assert!(
        count <= ROOT_MAX_LINES,
        "expected {ROOT} <= {ROOT_MAX_LINES} lines, found {count}"
    );
}

fn assert_root_markers(root: &str) {
    for marker in ROOT_MARKERS {
        assert!(root.contains(marker), "missing root marker: {marker}");
    }
}

fn assert_root_excludes(root: &str) {
    for marker in ROOT_EXCLUDES {
        assert!(
            !root.contains(marker),
            "root still contains moved marker: {marker}"
        );
    }
}

fn assert_module_files_exist_and_fit() {
    for rel in MODULES {
        let path = repo_path(rel);
        assert!(path.exists(), "missing module file: {}", path.display());
        let content = fs::read_to_string(&path).expect("read module");
        let count = line_count(&content);
        assert!(
            count <= MODULE_MAX_LINES,
            "expected {} <= {} lines, found {}",
            path.display(),
            MODULE_MAX_LINES,
            count
        );
    }
}

fn repo_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn line_count(content: &str) -> usize {
    content.lines().count()
}
