use std::path::{Path, PathBuf};

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
        "crates/kamn-e2e-harness/src/scenarios/s07_replay_protection.rs",
        "crates/kamn-e2e-harness/src/scenarios/s08_crash_recovery.rs",
        "crates/kamn-e2e-harness/src/scenarios/s09_transport_failover.rs",
        "crates/kamn-e2e-harness/src/scenarios/s10_topology_coherence.rs",
        "crates/kamn-e2e-harness/src/scenarios/s11_signer_rotation.rs",
        "crates/kamn-e2e-harness/src/scenarios/s12_retention_deletion.rs",
        "crates/kamn-e2e-harness/src/scenarios/s13_bridge_forwarding.rs",
        "crates/kamn-e2e-harness/src/scenarios/s14_batch_merkle.rs",
        "crates/kamn-e2e-harness/src/scenarios/s15_performance_smoke.rs",
        "crates/kamn-e2e-harness/src/evidence.rs",
        "crates/kamn-e2e-harness/src/verify.rs",
    ];

    for path in required_paths {
        assert!(root.join(path).is_file(), "required path missing: {path}");
    }
}

#[test]
fn spec_c12_phase4a_docs_markers_present() {
    let root = repo_root();
    let doc_path = root.join("docs/research/e2e-live-testing-prd-phase4a-gap-analysis.md");
    let doc = std::fs::read_to_string(&doc_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", doc_path.display()));

    assert!(doc.contains("phase4a_required_paths_total=28"));
    assert!(doc.contains("phase4a_required_paths_present_before=20"));
    assert!(doc.contains("phase4a_required_paths_missing_before=8"));
    assert!(doc.contains("phase4a_required_paths_present_after=28"));
    assert!(doc.contains("phase4a_required_paths_missing_after=0"));
    assert!(doc.contains("phase4a_scenario_inventory_count=15"));
    assert!(doc.contains("phase4a_manifest_schema_version=kamn.e2e.evidence-manifest.v3"));
    assert!(doc.contains("phase4a_verifier_report_markers=schema,proof,chain,content"));
    assert!(doc.contains("phase4a_status_after=implemented"));
}
