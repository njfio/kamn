use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/data_layer_m10_partition_archival/phase6.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "src/data_layer_m10_partition_archival/phase6/adapters.rs",
    "src/data_layer_m10_partition_archival/phase6/runtime_evidence.rs",
    "src/data_layer_m10_partition_archival/phase6/policy_mapping.rs",
    "src/data_layer_m10_partition_archival/phase6/scheduler.rs",
    "src/data_layer_m10_partition_archival/phase6/models.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod adapters;",
    "mod runtime_evidence;",
    "mod policy_mapping;",
    "mod scheduler;",
    "mod models;",
];
const MOVED_MARKERS: &[&str] = &[
    "struct M8Phase6CompliancePortAdapter<'a> {",
    "struct Phase6ProjectionPortBridge<'a, T: DataLayerM10Phase6CompliancePort> {",
    "fn map_phase6_runtime_evidence_bundle_from_policy(",
    "fn project_phase6_scheduler_cycle_report(",
    "impl DataLayerM10Phase6SchedulerPolicy {",
    "impl DataLayerM10Phase6SchedulerRuntime {",
];

#[test]
fn data_layer_m10_phase6_root_is_extracted() {
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
