use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_r55_evidence_step_inventory_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r55-evidence-step-inventory.md"),
    )
    .expect("r55 evidence-step docs marker artifact should exist");
    assert!(doc.contains("r55_evidence_step_inventory_status_before=single-step"));
    assert!(doc.contains("r55_evidence_step_inventory_contract=implemented"));
    assert!(doc.contains("r55_evidence_step_inventory_status_after=active"));
}

#[test]
fn spec_c02_r55_milestone_index_references_issue_5634() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r55-e2e-evidence-step-inventory-parity/index.md"),
    )
    .expect("r55 milestone index should exist");
    assert!(milestone_index.contains("#5634"));
}
