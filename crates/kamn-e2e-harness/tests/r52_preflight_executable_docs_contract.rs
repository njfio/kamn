use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c12_r52_preflight_executable_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r52-preflight-executable-diagnostics.md"),
    )
    .expect("r52 preflight executable docs marker artifact should exist");
    assert!(doc.contains("r52_preflight_executable_status_before=partial"));
    assert!(doc.contains("r52_preflight_executable_contract=implemented"));
    assert!(doc.contains("r52_preflight_executable_status_after=implemented"));
}

#[test]
fn spec_c13_r52_milestone_index_references_active_issue() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md"),
    )
    .expect("r52 milestone index should exist");
    assert!(milestone_index.contains("#5610"));
}
