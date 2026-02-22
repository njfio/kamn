use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r60_chain_genesis_anchor_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r60-chain-genesis-anchor-verification.md"),
    )
    .expect("r60 chain genesis anchor docs marker artifact should exist");
    assert!(doc.contains("r60_chain_genesis_anchor_contract_status_before=missing"));
    assert!(doc.contains("r60_verify_chain_genesis_anchor_enforcement=implemented"));
    assert!(doc.contains("r60_chain_genesis_anchor_contract_status_after=implemented"));
}

#[test]
fn spec_c02_r60_milestone_index_references_issue_5649() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r60-e2e-chain-genesis-anchor-verification-contract/index.md"),
    )
    .expect("r60 milestone index should exist");
    assert!(milestone_index.contains("#5649"));
}
