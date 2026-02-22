use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r58_chain_dump_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r58-chain-dump-verification-hardening.md"),
    )
    .expect("r58 chain dump docs marker artifact should exist");
    assert!(doc.contains("r58_chain_dump_marker_contract_status_before=missing"));
    assert!(doc.contains("r58_verify_chain_dump_marker_enforcement=implemented"));
    assert!(doc.contains("r58_chain_dump_marker_contract_status_after=implemented"));
}

#[test]
fn spec_c02_r58_milestone_index_references_issue_5643() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r58-e2e-chain-dump-verification-contract-hardening/index.md"),
    )
    .expect("r58 milestone index should exist");
    assert!(milestone_index.contains("#5643"));
}
