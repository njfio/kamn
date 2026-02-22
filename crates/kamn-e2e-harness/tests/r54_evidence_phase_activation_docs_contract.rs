use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r54_evidence_phase_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r54-evidence-phase-activation.md"),
    )
    .expect("r54 evidence-phase docs marker artifact should exist");
    assert!(doc.contains("r54_evidence_phase_status_before=static-skip"));
    assert!(doc.contains("r54_evidence_phase_contract=implemented"));
    assert!(doc.contains("r54_evidence_phase_status_after=active"));
}

#[test]
fn spec_c02_r54_milestone_index_references_active_issue_5629() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r54-e2e-evidence-phase-activation/index.md"),
    )
    .expect("r54 milestone index should exist");
    assert!(milestone_index.contains("#5629"));
}
