use std::fs;
use std::path::{Path, PathBuf};

const DOC_PATH: &str = "docs/validation/live-solana-bridge-websocket-slice.md";
const INDEX_PATH: &str = "docs/validation/current-proven-runtime-slices.md";

const REQUIRED_DOC_MARKERS: &[&str] = &[
    "service-api.bridge.forwarded",
    "websocket event stream",
    "KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL",
    "not live on-chain settlement",
    "integration_service_api_endpoint_websocket_streams_live_bridge_forwarded_event_after_upgrade",
];

const REQUIRED_INDEX_MARKERS: &[&str] = &[
    "live solana bridge websocket slice: `docs/validation/live-solana-bridge-websocket-slice.md`",
    "proves the live Solana-backed bridge evidence lane reaches the websocket event stream",
];

#[test]
fn live_solana_bridge_websocket_doc_exists_and_stays_bounded() {
    let doc = read_workspace_file(DOC_PATH);
    assert_contains_all(
        doc.as_str(),
        REQUIRED_DOC_MARKERS,
        "live Solana bridge websocket doc",
    );
}

#[test]
fn runtime_proof_index_includes_live_solana_bridge_websocket_slice() {
    let index = read_workspace_file(INDEX_PATH);
    assert_contains_all(
        index.as_str(),
        REQUIRED_INDEX_MARKERS,
        "runtime proof index",
    );
}

fn assert_contains_all(doc: &str, markers: &[&str], label: &str) {
    for marker in markers {
        assert!(doc.contains(marker), "{label} missing marker: {marker}");
    }
}

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
