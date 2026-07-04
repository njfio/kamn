use std::fs;
use std::path::{Path, PathBuf};

const DOC_PATH: &str = "docs/validation/live-escrow-cli-parity-slice.md";
const INDEX_PATH: &str = "docs/validation/current-proven-runtime-slices.md";
const SLICE_LABEL: &str =
    "live escrow CLI parity slice: `docs/validation/live-escrow-cli-parity-slice.md`";
const REQUIRED_DOC_MARKERS: &[&str] = &[
    "CLI-scripted S-05",
    "live_s05_cli_scripted_execution",
    "integration_live_s05_cli_scripted_escrow_settlement_probe_against_local_runtime",
    "KAMN_E2E_CLI_SCRIPTED_LIVE",
    "KAMN_E2E_CLI_BINARY",
    "-- --ignored --exact --nocapture",
    "does not prove Solana-backed settlement",
    "does not prove bridge settlement",
    "does not prove external-chain settlement",
    SLICE_LABEL,
];
const REQUIRED_INDEX_MARKERS: &[&str] = &[
    SLICE_LABEL,
    "proves one bounded live escrow settlement execution lane through CLI-scripted S-05 parity",
];

#[test]
fn live_escrow_cli_parity_doc_exists_and_stays_bounded() {
    let doc = read_workspace_file(DOC_PATH);
    assert_contains_all(
        doc.as_str(),
        REQUIRED_DOC_MARKERS,
        "live escrow cli parity doc",
    );
}

#[test]
fn runtime_proof_index_includes_live_escrow_cli_parity_slice() {
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
