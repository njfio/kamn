use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r64_verification_captured_at_format_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(root.join(
        "docs/research/e2e-live-testing-prd-r64-verification-captured-at-format-contract.md",
    ))
    .expect("r64 captured-at format docs marker artifact should exist");
    assert!(doc.contains("r64_verification_captured_at_format_contract_status_before=missing"));
    assert!(doc.contains("r64_verify_captured_at_format_enforcement=implemented"));
    assert!(doc.contains("r64_verification_captured_at_format_contract_status_after=implemented"));
}

#[test]
fn spec_c02_r64_milestone_index_references_issue_5661() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r64-e2e-verification-captured-at-format-contract/index.md"),
    )
    .expect("r64 milestone index should exist");
    assert!(milestone_index.contains("#5661"));
}
