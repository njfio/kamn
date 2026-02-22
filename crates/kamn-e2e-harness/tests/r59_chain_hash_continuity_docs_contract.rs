use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r59_chain_hash_continuity_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r59-chain-hash-continuity-verification.md"),
    )
    .expect("r59 chain hash continuity docs marker artifact should exist");
    assert!(doc.contains("r59_chain_hash_continuity_contract_status_before=missing"));
    assert!(doc.contains("r59_verify_chain_hash_continuity_enforcement=implemented"));
    assert!(doc.contains("r59_chain_hash_continuity_contract_status_after=implemented"));
}

#[test]
fn spec_c02_r59_milestone_index_references_issue_5646() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r59-e2e-chain-hash-continuity-verification-contract/index.md"),
    )
    .expect("r59 milestone index should exist");
    assert!(milestone_index.contains("#5646"));
}
