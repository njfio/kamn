use std::path::{Path, PathBuf};

use kamn_e2e_harness::VerifyMvpDemoCommandConfig;

use super::{canonical_bundle_fixture, mvp_local_artifacts, write_file};

pub(crate) fn temp_root(stem: &str) -> PathBuf {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_millis();
    std::env::temp_dir().join(format!("kamn-7047-{stem}-{}-{millis}", std::process::id()))
}

pub(crate) fn canonical_run_root(stem: &str, run_id: &str) -> PathBuf {
    temp_root(stem).join(run_id)
}

pub(crate) fn write_canonical_report(root: &Path, report: String) -> PathBuf {
    canonical_bundle_fixture::write(root, report)
}

pub(crate) fn config(report: &Path) -> VerifyMvpDemoCommandConfig {
    VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
        agent_harness_evidence_path: None,
        pi_transaction_actor_paths: None,
    }
}

pub(crate) fn write_report(root: &Path, report: String) -> PathBuf {
    let path = root.join("proof/report.json");
    mvp_local_artifacts::write_valid_local_artifacts(root);
    write_file(
        &root.join("proof/report.md"),
        "# MVP demo proof\n".to_owned(),
    );
    write_file(path.as_path(), report);
    path
}

pub(crate) fn write_artifact(root: &Path, artifact: String) -> PathBuf {
    let path = root.join("proof/agent-harness-evidence.json");
    write_file(path.as_path(), artifact);
    path
}

pub(crate) fn write_latest_artifact(root: &Path, artifact: String) -> PathBuf {
    let path = root.join("agent-harness-evidence.json");
    write_file(path.as_path(), artifact);
    path
}
