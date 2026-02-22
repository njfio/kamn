use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r56_verify_manifest_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r56-verify-manifest-hardening.md"),
    )
    .expect("r56 verify-manifest docs marker artifact should exist");
    assert!(doc.contains("r56_verify_manifest_nested_field_contract_status_before=partial"));
    assert!(doc.contains("r56_verify_manifest_infrastructure_marker_enforcement=implemented"));
    assert!(doc.contains("r56_verify_manifest_summary_marker_enforcement=implemented"));
    assert!(doc.contains("r56_verify_manifest_nested_field_contract_status_after=implemented"));
}

#[test]
fn spec_c02_r56_milestone_index_references_issue_5637() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r56-e2e-verify-manifest-contract-hardening/index.md"),
    )
    .expect("r56 milestone index should exist");
    assert!(milestone_index.contains("#5637"));
}
