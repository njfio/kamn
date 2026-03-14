use std::fs;
use std::path::{Path, PathBuf};

const DOC_PATH: &str = "docs/validation/solana-receipt-finality-slice.md";
const INDEX_DOC_PATH: &str = "docs/validation/current-proven-runtime-slices.md";
const REQUIRED_DOC_MARKERS: &[&str] = &[
    "# Solana Receipt Finality Slice",
    "solana_finalized_receipt_normalizes_to_final",
    "solana_processed_receipt_normalizes_to_pending",
    "regression_rejects_unknown_solana_finality_label",
    "not live Solana devnet proof",
    "not live chain-backed settlement",
];
const REQUIRED_INDEX_MARKERS: &[&str] = &[
    "solana receipt finality slice: `docs/validation/solana-receipt-finality-slice.md`",
    "proves bounded Solana receipt-finality normalization on the public core surface",
];

#[test]
fn solana_receipt_finality_doc_exists_with_bounded_markers() {
    let doc = read_workspace_file(DOC_PATH);
    assert_contains_all(doc.as_str(), REQUIRED_DOC_MARKERS, "solana receipt finality doc");
}

#[test]
fn runtime_proof_index_includes_solana_receipt_finality_slice() {
    let index = read_workspace_file(INDEX_DOC_PATH);
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
