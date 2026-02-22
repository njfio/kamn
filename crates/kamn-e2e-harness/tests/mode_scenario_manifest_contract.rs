use kamn_e2e_harness::evidence::MANIFEST_SCHEMA_VERSION;
use kamn_e2e_harness::scenarios::core_scenarios;
use kamn_e2e_harness::verify::verify_manifest;
use kamn_e2e_harness::{all_execution_modes, ExecutionMode};

#[test]
fn spec_c03_execution_mode_registry_contains_required_modes() {
    let modes = all_execution_modes();
    assert_eq!(modes.len(), 4);
    assert!(modes.contains(&ExecutionMode::SdkDirect));
    assert!(modes.contains(&ExecutionMode::CliScripted));
    assert!(modes.contains(&ExecutionMode::McpTau));
    assert!(modes.contains(&ExecutionMode::McpAny));
}

#[test]
fn spec_c04_core_scenario_registry_contains_required_ids() {
    let scenarios = core_scenarios();
    let required = ["S-01", "S-02", "S-03", "S-04", "S-05", "S-06", "S-08"];
    assert_eq!(scenarios.len(), required.len());
    for id in required {
        assert!(
            scenarios.iter().any(|scenario| scenario.id == id),
            "missing scenario id {id}"
        );
    }
}

#[test]
fn spec_c05_manifest_schema_version_marker_is_stable() {
    assert_eq!(MANIFEST_SCHEMA_VERSION, "kamn.e2e.evidence-manifest.v3");
}

#[test]
fn spec_c06_verify_contract_accepts_minimal_valid_manifest() {
    let manifest = r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","execution_mode":"sdk-direct","scenarios":[]}"#;
    verify_manifest(manifest).expect("manifest should verify");
}
