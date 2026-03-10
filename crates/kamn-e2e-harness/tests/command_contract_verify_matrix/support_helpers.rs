pub(crate) use kamn_e2e_harness::{
    execute_run_contract, execute_verify_contract, RunCommandConfig, VerifyCommandConfig,
};
pub(crate) use std::path::{Path, PathBuf};
pub(crate) use super::support::command_contract_support::{
    set_executable, temp_path, valid_chain_dump_json, with_external_component_binaries,
    write_failing_stub_binary,
};

pub(crate) const VALID_MANIFEST: &str = r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#;
pub(crate) const MISSING_INFRA_KOLME_VERSION_MANIFEST: &str = r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47,"proofs_verified":47}}"#;
pub(crate) const MISSING_SUMMARY_PROOFS_VERIFIED_MANIFEST: &str = r#"{"schema_version":"kamn.e2e.evidence-manifest.v3","run_id":"e2e-run","started_at":"2026-02-21T14:30:52Z","completed_at":"2026-02-21T14:35:12Z","duration_seconds":260,"execution_mode":"sdk-direct","infrastructure":{"kolme_version":"0.x.y","kamn_version":"0.1.0","kamn_commit":"49efe252","kamn_agent_lib_version":"0.1.0","agent_runtime":"sdk-direct","node_count":3,"agent_count":3,"storage_backend":"sqlite+postgres"},"scenarios":[],"summary":{"total_scenarios":15,"passed":13,"failed":1,"skipped":1,"kolme_blocks_produced":47,"messages_exchanged":128,"proofs_anchored":47}}"#;

pub(crate) struct VerifyPaths {
    pub(crate) evidence_dir: PathBuf,
    pub(crate) output_path: PathBuf,
    pub(crate) chain_dump_path: PathBuf,
}

pub(crate) fn setup_verify_case(case: &str, manifest: &str, chain_dump: &str) -> VerifyPaths {
    let paths = verify_paths(case);
    ensure_dir(&paths.evidence_dir, "evidence dir should be created");
    write_manifest(&paths.evidence_dir, manifest);
    write_chain_dump(&paths.chain_dump_path, chain_dump);
    paths
}

pub(crate) fn setup_verify_case_with_artifact(
    case: &str,
    manifest: &str,
    artifact: &str,
    chain_dump: &str,
) -> VerifyPaths {
    let paths = setup_verify_case(case, manifest, chain_dump);
    let scenario_dir = scenario_dir(&paths.evidence_dir);
    ensure_dir(&scenario_dir, "evidence scenario dir should be created");
    write_artifact(&scenario_dir, artifact);
    paths
}

pub(crate) fn verify_config(paths: &VerifyPaths) -> VerifyCommandConfig {
    VerifyCommandConfig {
        evidence_dir: paths.evidence_dir.display().to_string(),
        kolme_chain_dump: paths.chain_dump_path.display().to_string(),
        output: paths.output_path.display().to_string(),
    }
}

pub(crate) fn expect_verify_failure(
    paths: &VerifyPaths,
    failure_message: &str,
    expected_fragment: &str,
) {
    let err = execute_verify_contract(&verify_config(paths)).expect_err(failure_message);
    assert!(err.contains(expected_fragment));
}

pub(crate) fn expect_verify_success(paths: &VerifyPaths, success_message: &str) -> String {
    execute_verify_contract(&verify_config(paths)).expect(success_message)
}

pub(crate) fn cleanup_verify_case(paths: &VerifyPaths) {
    let _ = std::fs::remove_file(&paths.output_path);
    let _ = std::fs::remove_file(&paths.chain_dump_path);
    let _ = std::fs::remove_dir_all(&paths.evidence_dir);
}

pub(crate) fn probe_run_config(kolme_binary: &Path) -> RunCommandConfig {
    RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: kolme_binary.display().to_string(),
        agent_binary: None,
        external_execution: true,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-01".to_owned()],
    }
}

pub(crate) fn persistence_run_config(evidence_dir: &Path) -> RunCommandConfig {
    RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: evidence_dir.display().to_string(),
        scenario_ids: vec!["S-01".to_owned()],
    }
}

pub(crate) fn ordered_run_config() -> RunCommandConfig {
    RunCommandConfig {
        mode: "sdk-direct".to_owned(),
        kolme_binary: "/tmp/kolme-node".to_owned(),
        agent_binary: None,
        external_execution: false,
        evidence_dir: "/tmp/evidence".to_owned(),
        scenario_ids: vec!["S-03".to_owned(), "S-01".to_owned()],
    }
}

pub(crate) fn with_probe_binaries<R>(callback: impl FnOnce() -> R) -> R {
    with_external_component_binaries(callback)
}

pub(crate) fn failing_stub_binary(name: &str) -> PathBuf {
    let kolme_binary = temp_path(name);
    write_failing_stub_binary(&kolme_binary);
    #[cfg(unix)]
    set_executable(&kolme_binary);
    kolme_binary
}

pub(crate) fn cleanup_path(path: &Path) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_dir_all(path);
}

fn verify_paths(case: &str) -> VerifyPaths {
    VerifyPaths {
        evidence_dir: temp_path(&format!("evidence-{case}")),
        output_path: temp_path(&format!("report-{case}.json")),
        chain_dump_path: temp_path(&format!("kolme_chain_dump_{case}.json")),
    }
}

fn ensure_dir(path: &Path, message: &str) {
    std::fs::create_dir_all(path).expect(message);
}

fn write_manifest(evidence_dir: &Path, manifest: &str) {
    std::fs::write(evidence_dir.join("manifest.json"), manifest)
        .expect("manifest should be written");
}

fn write_chain_dump(path: &Path, contents: &str) {
    std::fs::write(path, contents).expect("chain dump should be written");
}

fn scenario_dir(evidence_dir: &Path) -> PathBuf {
    evidence_dir.join("s01-agent-discovery")
}

fn write_artifact(scenario_dir: &Path, artifact: &str) {
    std::fs::write(scenario_dir.join("alice_registration.json"), artifact)
        .expect("evidence artifact should be written");
}
