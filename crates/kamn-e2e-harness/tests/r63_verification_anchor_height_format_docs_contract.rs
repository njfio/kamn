use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r63_verification_anchor_height_format_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(root.join(
        "docs/research/e2e-live-testing-prd-r63-verification-anchor-height-format-contract.md",
    ))
    .expect("r63 anchor-height format docs marker artifact should exist");
    assert!(doc.contains("r63_verification_anchor_height_format_contract_status_before=missing"));
    assert!(doc.contains("r63_verify_anchor_block_height_format_enforcement=implemented"));
    assert!(doc.contains("r63_verification_anchor_height_format_contract_status_after=implemented"));
}

#[test]
fn spec_c02_r63_milestone_index_references_issue_5658() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r63-e2e-verification-anchor-height-format-contract/index.md"),
    )
    .expect("r63 milestone index should exist");
    assert!(milestone_index.contains("#5658"));
}
