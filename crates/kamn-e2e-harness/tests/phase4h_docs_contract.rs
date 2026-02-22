use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c08_phase4h_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-phase4h-gap-analysis.md"),
    )
    .expect("phase-4h docs marker artifact should exist");
    assert!(doc.contains("phase4h_status_before=partial"));
    assert!(doc.contains("phase4h_runtime_binary_contract=implemented"));
    assert!(doc.contains("phase4h_status_after=implemented"));
}

#[test]
fn spec_c09_milestone_index_references_active_phase4h_issue() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
    )
    .expect("milestone index should exist");
    assert!(milestone_index.contains("#5578"));
}
