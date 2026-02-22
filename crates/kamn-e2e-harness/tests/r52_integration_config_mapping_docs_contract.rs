use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c18_r52_integration_config_mapping_docs_markers_present() {
    let root = repo_root();
    let doc = std::fs::read_to_string(
        root.join("docs/research/e2e-live-testing-prd-r52-integration-config-mapping-fix.md"),
    )
    .expect("r52 integration-config mapping docs marker artifact should exist");
    assert!(doc.contains("r52_integration_config_mapping_status_before=buggy"));
    assert!(doc.contains("r52_integration_config_mapping_contract=implemented"));
    assert!(doc.contains("r52_integration_config_mapping_status_after=fixed"));
}

#[test]
fn spec_c19_r52_milestone_index_references_active_issue() {
    let root = repo_root();
    let milestone_index = std::fs::read_to_string(
        root.join("specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md"),
    )
    .expect("r52 milestone index should exist");
    assert!(milestone_index.contains("#5617"));
}
