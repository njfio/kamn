use std::fs;
use std::path::PathBuf;

const REQUIRED_MARKERS: &[&str] = &[
    "# Runtime Capability Audit R57",
    "## Message Routing",
    "status:",
    "evidence:",
    "## Task Dispatch",
    "## Audit Emission And Export",
    "## Live Transport",
    "implemented_and_wired",
    "gated_or_partial",
    "contract_only",
    "missing",
    "follow_on_issues:",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn audit_doc() -> String {
    fs::read_to_string(repo_root().join("docs/review/runtime-capability-audit-r57.md"))
        .expect("runtime capability audit doc should exist")
}

fn assert_markers_present(doc: &str) {
    for marker in REQUIRED_MARKERS {
        assert!(doc.contains(marker), "missing marker: {marker}");
    }
}

#[test]
fn runtime_capability_audit_declares_required_sections_and_statuses() {
    assert_markers_present(&audit_doc());
}
