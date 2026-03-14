use std::fs;
use std::path::{Path, PathBuf};

const DOC: &str = "docs/validation/bridge-finality-slice.md";
const INDEX: &str = "docs/validation/current-proven-runtime-slices.md";
const REQUIRED_DOC_MARKERS: &[&str] = &[
    "# Bridge Finality Slice",
    "ethereum_finalized_receipt_normalizes_to_final",
    "near_final_receipt_normalizes_to_final",
    "integration_service_api_endpoint_persists_bridge_state_across_restart",
    "deterministic receipt-finality normalization",
    "persisted forwarded bridge state",
    "not live chain-backed bridge finality",
    "not external settlement",
];
const REQUIRED_INDEX_MARKERS: &[&str] = &[
    "bridge finality slice: `docs/validation/bridge-finality-slice.md`",
    "proves deterministic receipt-finality normalization and persisted forwarded bridge state",
];

#[test]
fn bridge_finality_validation_doc_exists_and_stays_bounded() {
    let doc = read_workspace_file(DOC);
    assert_contains_all(doc.as_str(), REQUIRED_DOC_MARKERS, "bridge finality validation doc");
}

#[test]
fn runtime_proof_index_includes_bridge_finality_slice() {
    let index = read_workspace_file(INDEX);
    assert_contains_all(index.as_str(), REQUIRED_INDEX_MARKERS, "runtime proof index");
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
