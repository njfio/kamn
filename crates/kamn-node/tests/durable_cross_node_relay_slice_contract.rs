use std::fs;
use std::path::{Path, PathBuf};

const DOC_PATH: &str = "docs/validation/durable-cross-node-relay-slice.md";
const DOC_MARKERS: &[&str] = &[
    "# Durable Cross-Node Relay Slice",
    "sender enqueue to relay spool",
    "fail-closed pending spool preservation",
    "later successful relay projection",
    "recipient-visible delivered state",
    "What This Proves",
    "What This Does Not Prove",
    "integration_service_api_endpoint_cross_node_relay_delivery_contract",
    "integration_runtime_daemon_without_route_map_preserves_relay_spool_entries",
    "regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart",
    "relayed",
    "delivered",
    "relay spool",
];

fn read_workspace_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    assert!(path.exists(), "expected path to exist: {}", path.display());
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {}", path.display(), error))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}

#[test]
fn regression_durable_cross_node_relay_doc_exists_with_required_markers() {
    let doc = read_workspace_file(DOC_PATH);
    for marker in DOC_MARKERS {
        assert!(
            doc.contains(marker),
            "durable cross-node relay doc missing required marker: {}",
            marker
        );
    }
}
