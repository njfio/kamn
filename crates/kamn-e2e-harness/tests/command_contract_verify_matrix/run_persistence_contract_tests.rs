use super::support_helpers::*;

#[test]
fn spec_c110_run_command_persists_manifest_chain_dump_and_scenario_artifact_on_pass() {
    let evidence_dir = temp_path("evidence-persist-pass");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    let output = execute_run_contract(&persistence_run_config(&evidence_dir))
        .expect("run output should render");
    assert!(output.contains("\"evidence_contract\":{\"expected_artifacts\":4,\"recorded_artifacts\":4,\"status\":\"PASS\"}"));
    assert_pass_persistence_outputs(&evidence_dir);
    cleanup_path(&evidence_dir);
}

#[test]
fn spec_c111_run_command_evidence_fail_path_omits_chain_dump_and_scenario_artifacts() {
    let evidence_dir = temp_path("evidence-fail-persist");
    std::fs::create_dir_all(&evidence_dir).expect("evidence dir should be created");
    let stale_chain_dump = evidence_dir.join("kolme_chain_dump.json");
    std::fs::write(&stale_chain_dump, valid_chain_dump_json())
        .expect("stale chain dump should be written");
    let output = execute_run_contract(&persistence_run_config(&evidence_dir))
        .expect("run output should render");
    assert!(output.contains("\"evidence_contract\":{\"expected_artifacts\":4,\"recorded_artifacts\":3,\"status\":\"FAIL\"}"));
    assert_fail_persistence_outputs(&evidence_dir);
    cleanup_path(&evidence_dir);
}

fn assert_pass_persistence_outputs(evidence_dir: &Path) {
    let manifest_path = evidence_dir.join("manifest.json");
    let chain_dump_path = evidence_dir.join("kolme_chain_dump.json");
    let scenario_artifact_path = evidence_dir.join("scenario-s01").join("artifact.json");
    assert!(manifest_path.is_file(), "manifest should be persisted");
    assert!(chain_dump_path.is_file(), "chain dump should be persisted");
    assert!(
        scenario_artifact_path.is_file(),
        "scenario artifact should be persisted"
    );
    assert_pass_artifact_contents(&manifest_path, &scenario_artifact_path);
}

fn assert_pass_artifact_contents(manifest_path: &Path, scenario_artifact_path: &Path) {
    let manifest =
        std::fs::read_to_string(manifest_path).expect("manifest content should be readable");
    assert!(manifest.contains("\"schema_version\":\"kamn.e2e.evidence-manifest.v3\""));
    assert!(manifest.contains("\"id\":\"S-01\""));
    assert!(manifest.contains("scenario-s01/artifact.json"));
    let scenario_artifact = std::fs::read_to_string(scenario_artifact_path)
        .expect("scenario artifact content should be readable");
    assert!(scenario_artifact.contains("\"_verification\":"));
    assert!(scenario_artifact.contains("\"finality\":\"FINAL\""));
}

fn assert_fail_persistence_outputs(evidence_dir: &Path) {
    let manifest_path = evidence_dir.join("manifest.json");
    let chain_dump_path = evidence_dir.join("kolme_chain_dump.json");
    let scenario_artifact_path = evidence_dir.join("scenario-s01").join("artifact.json");
    assert!(
        manifest_path.is_file(),
        "manifest should still be persisted"
    );
    assert!(
        !chain_dump_path.exists(),
        "chain dump should be removed on fail path"
    );
    assert!(
        !scenario_artifact_path.exists(),
        "scenario artifacts should not be persisted on fail path"
    );
}
