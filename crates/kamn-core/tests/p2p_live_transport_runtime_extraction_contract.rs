use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "tests/p2p_live_transport_runtime.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "tests/p2p_live_transport_runtime/support.rs",
    "tests/p2p_live_transport_runtime/startup_profile_contract_tests.rs",
    "tests/p2p_live_transport_runtime/exchange_flow_contract_tests.rs",
    "tests/p2p_live_transport_runtime/event_normalization_contract_tests.rs",
    "tests/p2p_live_transport_runtime/fail_closed_regression_contract_tests.rs",
    "tests/p2p_live_transport_runtime/backpressure_contract_tests.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod support;",
    "mod startup_profile_contract_tests;",
    "mod exchange_flow_contract_tests;",
    "mod event_normalization_contract_tests;",
    "mod fail_closed_regression_contract_tests;",
    "mod backpressure_contract_tests;",
];
const MOVED_MARKERS: &[&str] = &[
    "fn config_for(role: NodeRole, gossip_enabled: bool) -> NodeConfig",
    "fn unit_live_transport_adapter_reports_harness_startup_profile()",
    "fn integration_live_transport_data_plane_supports_independent_adapter_exchange()",
    "fn functional_live_transport_emits_normalized_runtime_events()",
    "fn regression_live_transport_data_plane_unknown_recipient_fails_closed()",
    "fn functional_live_transport_dispatch_backpressure_rejects_saturated_inbox()",
];

#[test]
fn p2p_live_transport_runtime_root_is_extracted() {
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
