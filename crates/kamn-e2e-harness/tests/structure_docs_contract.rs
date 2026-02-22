use std::path::{Path, PathBuf};

const DOC: &str =
    include_str!("../../../docs/research/e2e-live-testing-prd-phase3-gap-analysis.md");

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .to_path_buf()
}

#[test]
fn spec_c01_harness_required_paths_exist() {
    let root = repo_root();
    let required_paths = [
        "crates/kamn-e2e-harness/Cargo.toml",
        "crates/kamn-e2e-harness/src/main.rs",
        "crates/kamn-e2e-harness/src/infrastructure.rs",
        "crates/kamn-e2e-harness/src/kolme_devnet.rs",
        "crates/kamn-e2e-harness/src/identity.rs",
        "crates/kamn-e2e-harness/src/drivers/mod.rs",
        "crates/kamn-e2e-harness/src/drivers/sdk_direct.rs",
        "crates/kamn-e2e-harness/src/drivers/cli_scripted.rs",
        "crates/kamn-e2e-harness/src/drivers/mcp_agent.rs",
        "crates/kamn-e2e-harness/src/scenarios/mod.rs",
        "crates/kamn-e2e-harness/src/scenarios/s01_discovery.rs",
        "crates/kamn-e2e-harness/src/scenarios/s02_message.rs",
        "crates/kamn-e2e-harness/src/scenarios/s03_group.rs",
        "crates/kamn-e2e-harness/src/scenarios/s04_task.rs",
        "crates/kamn-e2e-harness/src/scenarios/s05_escrow.rs",
        "crates/kamn-e2e-harness/src/scenarios/s06_kolme_verify.rs",
        "crates/kamn-e2e-harness/src/scenarios/s08_crash_recovery.rs",
        "crates/kamn-e2e-harness/src/evidence.rs",
        "crates/kamn-e2e-harness/src/verify.rs",
    ];

    for path in required_paths {
        assert!(root.join(path).is_file(), "required path missing: {path}");
    }
}

#[test]
fn spec_c08_phase3_docs_markers_present() {
    assert!(DOC.contains("phase3_required_paths_total=20"));
    assert!(DOC.contains("phase3_required_paths_present_before=0"));
    assert!(DOC.contains("phase3_required_paths_missing_before=20"));
    assert!(DOC.contains("phase3_required_paths_present_after=20"));
    assert!(DOC.contains("phase3_required_paths_missing_after=0"));
    assert!(DOC.contains("phase3_execution_mode_inventory_count=4"));
    assert!(DOC.contains("phase3_core_scenario_inventory_count=7"));
    assert!(DOC.contains("phase3_manifest_schema_version=kamn.e2e.evidence-manifest.v3"));
    assert!(DOC.contains("phase3_status_after=implemented"));
}
