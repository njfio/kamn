use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c09_phase5b_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-phase5b-gap-analysis.md"),
    )
    .expect("phase-5b docs marker artifact should exist");
    assert!(doc.contains("phase5b_status_before=partial"));
    assert!(doc.contains("phase5b_process_lifecycle_contract=implemented"));
    assert!(doc.contains("phase5b_status_after=implemented"));
}

#[test]
fn spec_c10_milestone_index_references_active_phase5b_issue() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
    )
    .expect("milestone index should exist");
    assert!(milestone_index.contains("#5586"));
}
