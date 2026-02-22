use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r53_live_status_alignment_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r53-live-status-alignment.md"),
    )
    .expect("r53 live-status docs marker artifact should exist");
    assert!(doc.contains("r53_live_status_alignment_status_before=static-pass"));
    assert!(doc.contains("r53_live_status_alignment_contract=implemented"));
    assert!(doc.contains("r53_live_status_alignment_status_after=active"));
}

#[test]
fn spec_c02_r53_milestone_index_references_active_issue_5622() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r53-e2e-scenario-execution-activation/index.md"),
    )
    .expect("r53 milestone index should exist");
    assert!(milestone_index.contains("#5622"));
}
