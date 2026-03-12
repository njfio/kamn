use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/runtime_phase_coordination.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "src/runtime_phase_coordination/construct_lock.rs",
    "src/runtime_phase_coordination/listener_quorum.rs",
    "src/runtime_phase_coordination/approver_quorum.rs",
    "src/runtime_phase_coordination/did_validation.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod construct_lock;",
    "mod listener_quorum;",
    "mod approver_quorum;",
    "mod did_validation;",
];
const MOVED_MARKERS: &[&str] = &[
    "pub struct ConstructLockLease {",
    "pub enum ConstructLockError {",
    "pub struct ConstructLockGuard {",
    "pub struct ListenerAttestation {",
    "pub struct ListenerQuorumEvaluator {",
    "pub struct ApproverAttestation {",
    "pub struct ApproverQuorumEvaluator {",
    "fn parse_listener_did(",
    "fn parse_approver_did(",
];

#[test]
fn runtime_phase_coordination_root_is_extracted() {
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
