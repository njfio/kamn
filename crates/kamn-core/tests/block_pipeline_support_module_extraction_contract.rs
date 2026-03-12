use std::fs;
use std::path::{Path, PathBuf};

const ROOT: &str = "src/block_pipeline/block_pipeline_support.rs";
const ROOT_CAP: usize = 180;
const MODULE_CAP: usize = 200;
const MODULE_FILES: &[&str] = &[
    "src/block_pipeline/block_pipeline_support/gossip_ingress.rs",
    "src/block_pipeline/block_pipeline_support/transport_feeds.rs",
    "src/block_pipeline/block_pipeline_support/convergence_evidence.rs",
    "src/block_pipeline/block_pipeline_support/commit_store.rs",
    "src/block_pipeline/block_pipeline_support/codec.rs",
    "src/block_pipeline/block_pipeline_support/fork_choice.rs",
];
const REQUIRED_MARKERS: &[&str] = &[
    "mod gossip_ingress;",
    "mod transport_feeds;",
    "mod convergence_evidence;",
    "mod commit_store;",
    "mod codec;",
    "mod fork_choice;",
];
const MOVED_MARKERS: &[&str] = &[
    "pub enum GossipIngressRecord {",
    "pub struct GossipFrameTransportMempoolFeed {",
    "pub fn build_transport_convergence_evidence_bundle(",
    "pub struct InMemoryCanonicalCommitStore {",
    "fn serialize_canonical_commit_record(",
    "pub enum ForkChoiceDecision {",
];

#[test]
fn block_pipeline_support_root_is_extracted() {
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
