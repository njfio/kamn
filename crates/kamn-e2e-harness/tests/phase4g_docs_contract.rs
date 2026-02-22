use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c07_phase4g_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-phase4g-gap-analysis.md"),
    )
    .expect("phase-4g docs marker artifact should exist");
    assert!(doc.contains("phase4g_status_before=partial"));
    assert!(doc.contains("phase4g_lifecycle_summary=implemented"));
    assert!(doc.contains("phase4g_fail_path_summary=implemented"));
    assert!(doc.contains("phase4g_status_after=implemented"));
}

#[test]
fn spec_c08_milestone_index_references_active_phase4g_issue() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
    )
    .expect("milestone index should exist");
    assert!(milestone_index.contains("#5576"));
}
