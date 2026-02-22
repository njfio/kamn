use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c15_phase6_runtime_validation_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-phase6-runtime-validation-gap-analysis.md"),
    )
    .expect("phase-6 runtime validation docs marker artifact should exist");
    assert!(doc.contains("phase6_runtime_validation_status_before=partial"));
    assert!(doc.contains("phase6_runtime_validation_contract=implemented"));
    assert!(doc.contains("phase6_runtime_validation_status_after=implemented"));
}

#[test]
fn spec_c16_milestone_index_references_active_phase6_runtime_validation_issue() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md"),
    )
    .expect("milestone index should exist");
    assert!(milestone_index.contains("#5606"));
}
