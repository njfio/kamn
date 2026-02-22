use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c16_r52_preflight_absolute_path_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r52-preflight-absolute-path-diagnostics.md"),
    )
    .expect("r52 preflight absolute-path docs marker artifact should exist");
    assert!(doc.contains("r52_preflight_absolute_path_status_before=partial"));
    assert!(doc.contains("r52_preflight_absolute_path_contract=implemented"));
    assert!(doc.contains("r52_preflight_absolute_path_status_after=implemented"));
}

#[test]
fn spec_c17_r52_milestone_index_references_active_issue() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md"),
    )
    .expect("r52 milestone index should exist");
    assert!(milestone_index.contains("#5615"));
}
