use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r62_verification_hash_format_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r62-verification-hash-format-contract.md"),
    )
    .expect("r62 hash-format docs marker artifact should exist");
    assert!(doc.contains("r62_verification_hash_format_contract_status_before=missing"));
    assert!(doc.contains("r62_verify_artifact_hash_format_enforcement=implemented"));
    assert!(doc.contains("r62_verification_hash_format_contract_status_after=implemented"));
}

#[test]
fn spec_c02_r62_milestone_index_references_issue_5655() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r62-e2e-verification-hash-format-contract/index.md"),
    )
    .expect("r62 milestone index should exist");
    assert!(milestone_index.contains("#5655"));
}
