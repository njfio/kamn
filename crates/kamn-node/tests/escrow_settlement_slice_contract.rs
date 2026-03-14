use std::fs;
use std::path::{Path, PathBuf};

const DOC: &str = "docs/validation/escrow-settlement-slice.md";
const INDEX: &str = "docs/validation/current-proven-runtime-slices.md";
const REQUIRED_DOC_MARKERS: &[&str] = &[
    "# Escrow Settlement Slice",
    "integration_service_api_endpoint_persists_task_and_escrow_state_across_routes",
    "integration_service_api_endpoint_persists_task_and_escrow_state_across_restart",
    "service-api escrow lifecycle persistence",
    "not bridge finality",
    "not external chain settlement",
];
const REQUIRED_INDEX_MARKERS: &[&str] = &[
    "escrow settlement slice: `docs/validation/escrow-settlement-slice.md`",
    "proves service-api escrow lifecycle persistence through fund, release, and restart-visible released state",
];

#[test]
fn escrow_settlement_validation_doc_exists_and_stays_bounded() {
    assert_contains_all(
        read_workspace_file(DOC).as_str(),
        REQUIRED_DOC_MARKERS,
        "escrow settlement validation doc",
    );
}

#[test]
fn runtime_proof_index_includes_escrow_settlement_slice() {
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
