use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r61_verification_finality_value_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r61-verification-finality-value-contract.md"),
    )
    .expect("r61 finality value docs marker artifact should exist");
    assert!(doc.contains("r61_verification_finality_value_contract_status_before=missing"));
    assert!(doc.contains("r61_verify_artifact_finality_value_enforcement=implemented"));
    assert!(doc.contains("r61_verification_finality_value_contract_status_after=implemented"));
}

#[test]
fn spec_c02_r61_milestone_index_references_issue_5652() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r61-e2e-verification-finality-value-contract/index.md"),
    )
    .expect("r61 milestone index should exist");
    assert!(milestone_index.contains("#5652"));
}
