use std::fs;
use std::path::{Path, PathBuf};

const INDEX_PATH: &str = "docs/validation/current-proven-runtime-slices.md";
const EXTERNAL_SECTION: &str = "## External Evidence That Is Not a Proven Runtime Slice";
const PROVEN_MARKERS: &[&str] = &[
    "docs/validation/live-chain-backed-bridge-finality-slice.md",
    "docs/validation/bridge-authorized-escrow-settlement-slice.md",
    "reuses the finalized bridge transfer",
    "docs/validation/authoritative-live-settlement-driver-parity-slice.md",
    "composes live finality with deterministic service and adapter contracts",
];
const EXTERNAL_MARKERS: &[&str] = &[
    EXTERNAL_SECTION,
    "docs/validation/external-a2a-x402-receipt-authority-probe.md",
    "FAIL",
    "BLOCKED",
    "no approval response",
    "no settlement response",
];

#[test]
fn authority_closeout_artifacts_are_indexed_with_bounded_claims() {
    let index = read_workspace_file(INDEX_PATH);
    assert_contains_all(&index, PROVEN_MARKERS);
    assert_contains_all(&index, EXTERNAL_MARKERS);
}

#[test]
fn external_probe_is_outside_the_proven_runtime_section() {
    let index = read_workspace_file(INDEX_PATH);
    assert_marker_follows(&index, EXTERNAL_SECTION, "## What Remains Unproven");
}

fn assert_marker_follows(doc: &str, marker: &str, predecessor: &str) {
    assert!(
        marker_position(doc, marker) > marker_position(doc, predecessor),
        "{marker} must follow {predecessor}"
    );
}

fn marker_position(doc: &str, marker: &str) -> usize {
    doc.find(marker)
        .unwrap_or_else(|| panic!("runtime proof index missing marker: {marker}"))
}

fn assert_contains_all(doc: &str, markers: &[&str]) {
    for marker in markers {
        assert!(
            doc.contains(marker),
            "runtime proof index missing marker: {marker}"
        );
    }
}

fn read_workspace_file(relative_path: &str) -> String {
    let path = workspace_root().join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root should resolve")
}
