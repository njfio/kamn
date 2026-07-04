use std::fs;
use std::path::{Path, PathBuf};

const DOC_PATH: &str = "docs/validation/live-solana-devnet-proof-slice.md";
const INDEX_PATH: &str = "docs/validation/current-proven-runtime-slices.md";
const REQUIRED_DOC_MARKERS: &[&str] = &[
    "# Live Solana Devnet Proof Slice",
    "scripts/runtime/run_live_solana_devnet_proof.py",
    "scripts/runtime/validate_live_solana_devnet_proof.py",
    "scripts/runtime/check_live_solana_devnet_proof_policy.py",
    "cargo test -p kamn-core --test live_solana_devnet_receipt_normalization -- --nocapture",
    "normalize_cross_chain_receipt",
    "What This Slice Proves",
    "What This Slice Does Not Prove",
    "not live bridge settlement",
    "not live message relay",
    "Solana devnet JSON-RPC",
];
const REQUIRED_INDEX_MARKERS: &[&str] = &[
    "live solana devnet proof slice: `docs/validation/live-solana-devnet-proof-slice.md`",
    "proves bounded live Solana devnet JSON-RPC evidence normalized through the public receipt surface",
];

#[test]
fn live_solana_devnet_proof_doc_exists_and_stays_bounded() {
    let doc = read_workspace_file(DOC_PATH);
    assert_contains_all(
        doc.as_str(),
        REQUIRED_DOC_MARKERS,
        "live Solana devnet proof doc",
    );
}

#[test]
fn runtime_proof_index_includes_live_solana_devnet_proof_slice() {
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
