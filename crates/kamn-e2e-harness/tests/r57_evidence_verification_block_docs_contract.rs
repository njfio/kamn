use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r57_evidence_verification_block_docs_markers_present() {
    let root = repo_root();
    let doc =
        std::fs::read_to_string(root.join(
            "docs/research/e2e-live-testing-prd-r57-evidence-verification-block-enforcement.md",
        ))
        .expect("r57 evidence verification block docs marker artifact should exist");
    assert!(doc.contains("r57_evidence_verification_block_contract_status_before=missing"));
    assert!(doc.contains("r57_verify_artifact_verification_marker_enforcement=implemented"));
    assert!(doc.contains("r57_evidence_verification_block_contract_status_after=implemented"));
}

#[test]
fn spec_c02_r57_milestone_index_references_issue_5640() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r57-e2e-evidence-verification-block-enforcement/index.md"),
    )
    .expect("r57 milestone index should exist");
    assert!(milestone_index.contains("#5640"));
}
