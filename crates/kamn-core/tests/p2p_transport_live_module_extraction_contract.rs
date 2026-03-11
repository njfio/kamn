use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/p2p_transport/p2p_transport_live.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "src/p2p_transport/p2p_transport_live/runtime_inbox.rs",
    "src/p2p_transport/p2p_transport_live/peer_lifecycle_transport.rs",
    "src/p2p_transport/p2p_transport_live/native_runtime_loop.rs",
    "src/p2p_transport/p2p_transport_live/swarm_runtime.rs",
    "src/p2p_transport/p2p_transport_live/deterministic_config.rs",
    "src/p2p_transport/p2p_transport_live/regression_harness.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod runtime_inbox;",
    "mod peer_lifecycle_transport;",
    "mod native_runtime_loop;",
    "mod swarm_runtime;",
    "mod deterministic_config;",
    "mod regression_harness;",
];
const MOVED_MARKERS: &[&str] = &[
    "fn build_live_runtime_inbox_backpressure_controller(",
    "pub struct Libp2pLivePeerLifecycleTransport {",
    "struct Libp2pLiveDataPlane {",
    "enum Libp2pNativeRuntimeAdapterLoopCommand {",
    "pub struct P2pSwarmDeterministicConfig",
    "pub fn build_libp2p_lifecycle_regression_corpus()",
];

#[test]
fn p2p_transport_live_root_is_extracted() {
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
