use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c08_phase4d_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-phase4d-gap-analysis.md"),
    )
    .expect("phase-4d docs marker artifact should exist");
    assert!(doc.contains("phase4d_status_before=partial"));
    assert!(doc.contains("phase4d_phase_result_model=implemented"));
    assert!(doc.contains("phase4d_infra_and_agent_placeholders=implemented"));
    assert!(doc.contains("phase4d_status_after=implemented"));
}

#[test]
fn spec_c09_milestone_index_references_active_phase4d_issue() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
    )
    .expect("milestone index should exist");
    assert!(milestone_index.contains("#5570"));
}
