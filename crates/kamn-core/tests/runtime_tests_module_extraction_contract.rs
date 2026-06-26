use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/runtime_tests.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "src/runtime_tests/runtime_wiring_contract_tests.rs",
    "src/runtime_tests/lifecycle_backpressure_contract_tests.rs",
    "src/runtime_tests/peer_frame_contract_tests.rs",
    "src/runtime_tests/planner_recovery_lock_contract_tests.rs",
    "src/runtime_tests/quorum_watchdog_contract_tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "#[path = \"runtime_tests_snapshot_store.rs\"]",
    "mod runtime_tests_snapshot_store;",
    "#[path = \"runtime_tests_network_fault.rs\"]",
    "mod runtime_tests_network_fault;",
    "#[path = \"runtime_tests/runtime_wiring_contract_tests.rs\"]",
    "mod runtime_wiring_contract_tests;",
    "#[path = \"runtime_tests/lifecycle_backpressure_contract_tests.rs\"]",
    "mod lifecycle_backpressure_contract_tests;",
    "#[path = \"runtime_tests/peer_frame_contract_tests.rs\"]",
    "mod peer_frame_contract_tests;",
    "#[path = \"runtime_tests/planner_recovery_lock_contract_tests.rs\"]",
    "mod planner_recovery_lock_contract_tests;",
    "#[path = \"runtime_tests/quorum_watchdog_contract_tests.rs\"]",
    "mod quorum_watchdog_contract_tests;",
];
const MOVED_MARKERS: &[&str] = &[
    "fn processor_wiring_contains_block_producer()",
    "fn functional_peer_lifecycle_allows_connect_heartbeat_recover_disconnect_flow()",
    "fn unit_authenticated_peer_frame_rejects_invalid_wire_format()",
    "fn functional_planner_orders_candidates_deterministically()",
    "fn functional_listener_quorum_accepts_canonical_sufficient_attestations()",
];

#[test]
fn runtime_tests_root_is_extracted() {
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
        assert!(
            root.contains(marker),
            "missing root module marker: {marker}"
        );
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
        assert!(
            full.exists(),
            "missing extracted module: {}",
            full.display()
        );
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
