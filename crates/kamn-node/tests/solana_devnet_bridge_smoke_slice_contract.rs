use std::fs;
use std::path::PathBuf;

const DOC_PATH: &str = "docs/validation/solana-devnet-bridge-smoke-slice.md";
const INDEX_PATH: &str = "docs/validation/current-proven-runtime-slices.md";
const REQUIRED_DOC_MARKERS: [&str; 9] = [
    "# Solana Devnet Bridge Smoke Slice",
    "solana:devnet:program:task-v1",
    "solana_quorum_dispatches_outbound",
    "integration_projects_solana_inbound_to_envelope",
    "regression_rejects_replayed_solana_inbound_projection_event",
    "integration_outbound_quorum_rejections_are_explicit_and_fail_closed",
    "not live Solana RPC/devnet-backed proof",
    "What This Slice Proves",
    "What This Slice Does Not Prove",
];
const REQUIRED_INDEX_MARKERS: [&str; 2] = [
    "solana devnet bridge smoke slice: `docs/validation/solana-devnet-bridge-smoke-slice.md`",
    "proves a bounded Solana devnet-addressed bridge smoke path on the public bridge surface",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn read_text(path: &str) -> String {
    let full_path = repo_root().join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", full_path.display()))
}

#[test]
fn solana_devnet_bridge_smoke_doc_exists_with_bounded_markers() {
    let doc = read_text(DOC_PATH);
    for marker in REQUIRED_DOC_MARKERS {
        assert!(
            doc.contains(marker),
            "solana devnet bridge smoke doc missing marker: {marker}"
        );
    }
}

#[test]
fn runtime_proof_index_includes_solana_devnet_bridge_smoke_slice() {
    let index = read_text(INDEX_PATH);
    for marker in REQUIRED_INDEX_MARKERS {
        assert!(
            index.contains(marker),
            "runtime proof index missing marker: {marker}"
        );
    }
}
