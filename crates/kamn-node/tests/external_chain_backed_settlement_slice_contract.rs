use std::fs;
use std::path::{Path, PathBuf};

const DOC: &str = "docs/validation/external-chain-backed-settlement-slice.md";
const INDEX: &str = "docs/validation/current-proven-runtime-slices.md";
const REQUIRED_DOC_MARKERS: &[&str] = &[
    "# External-Chain-Backed Settlement Slice",
    "integration_service_api_endpoint_live_settlement_release_persists_external_receipt_linkage",
    "What This Proves",
    "What This Does Not Prove",
    "external-chain-backed escrow settlement lane",
    "not external economic settlement",
];
const REQUIRED_INDEX_MARKERS: &[&str] = &[
    "external-chain-backed settlement slice: `docs/validation/external-chain-backed-settlement-slice.md`",
    "proves one bounded external-chain-backed escrow settlement lane",
];

#[test]
fn external_chain_backed_settlement_doc_exists_and_stays_bounded() {
    assert_contains_all(
        read_workspace_file(DOC).as_str(),
        REQUIRED_DOC_MARKERS,
        "external-chain-backed settlement doc",
    );
}

#[test]
fn runtime_proof_index_includes_external_chain_backed_settlement_slice() {
    assert_contains_all(
        read_workspace_file(INDEX).as_str(),
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
