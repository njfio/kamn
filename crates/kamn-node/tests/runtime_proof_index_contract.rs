use std::fs;
use std::path::{Path, PathBuf};

const INDEX_DOC_PATH: &str = "docs/validation/current-proven-runtime-slices.md";
const AUDIT_RESPONSE_PATH: &str = "docs/review/corrected-audit-response-2026-03-14.md";
const INDEX_MARKERS: &[&str] = &[
    "# Current Proven Runtime Slices",
    "docs/validation/working-vertical-slice.md",
    "docs/validation/sdk-tcp-vertical-slice.md",
    "docs/validation/durable-cross-node-relay-slice.md",
    "What Is Currently Proven",
    "What Remains Unproven",
    "service-api vertical slice",
    "TCP signed-relay vertical slice",
    "durable cross-node relay slice",
];

#[test]
fn runtime_proof_index_exists_with_required_markers() {
    assert_contains_all(read_workspace_file(INDEX_DOC_PATH).as_str(), INDEX_MARKERS);
}

#[test]
fn corrected_audit_response_links_runtime_proof_index() {
    let doc = read_workspace_file(AUDIT_RESPONSE_PATH);
    assert!(
        doc.contains("docs/validation/current-proven-runtime-slices.md"),
        "corrected audit response must link the runtime proof index"
    );
}

fn assert_contains_all(doc: &str, markers: &[&str]) {
    for marker in markers {
        assert!(
            doc.contains(marker),
            "runtime proof index missing marker: {}",
            marker
        );
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
