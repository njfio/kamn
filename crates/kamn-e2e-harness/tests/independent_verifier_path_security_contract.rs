use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig,
};
use std::path::{Path, PathBuf};

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c21_run_id_cannot_select_its_own_containment_root() {
    let fixture = Fixture::new("run-id-traversal");
    let run_id = fixture.run_id();
    fixture.replace_report(
        format!(r#""run_id":"{run_id}""#).as_str(),
        format!(r#""run_id":"latest/../{run_id}""#).as_str(),
    );

    fixture.assert_error("PROOF_ARTIFACT_PATH_INVALID");
}

#[test]
fn spec_c22_report_markdown_index_must_name_verified_file() {
    let fixture = Fixture::new("markdown-index");
    let foreign = fixture.run_dir.join("proof/foreign-report.md");
    let indexed = fixture.run_dir.join("proof/report.md");
    std::fs::copy(&indexed, &foreign).expect("foreign markdown");
    fixture.replace_report(
        &indexed.display().to_string(),
        &foreign.display().to_string(),
    );

    fixture.assert_error("PROOF_ARTIFACT_PATH_INVALID");
}

#[test]
fn spec_c23_concrete_run_report_verifies_without_latest_alias() {
    let fixture = Fixture::new("concrete-run-report");
    let report = fixture.run_dir.join("proof/report.json");

    let output = execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
        report: report.display().to_string(),
        agent_harness_evidence_path: None,
        pi_transaction_actor_paths: None,
    })
    .expect("concrete run report should verify");
    assert!(output.contains(r#""status":"PASS""#));
}

struct Fixture {
    root: PathBuf,
    run_dir: PathBuf,
}

impl Fixture {
    fn new(stem: &str) -> Self {
        let root = temp_root(stem);
        execute_mvp_demo_contract(&mvp_demo_command::devnet_required_demo_config(&root))
            .expect("valid devnet fixture");
        let run_dir = only_run_dir(&root);
        Self { root, run_dir }
    }

    fn run_id(&self) -> String {
        self.run_dir
            .file_name()
            .and_then(|name| name.to_str())
            .expect("run id")
            .to_owned()
    }

    fn report_path(&self) -> PathBuf {
        self.root.join("latest/proof/report.json")
    }

    fn replace_report(&self, from: &str, to: &str) {
        let path = self.report_path();
        let raw = std::fs::read_to_string(&path).expect("report");
        assert!(raw.contains(from), "report marker");
        std::fs::write(path, raw.replace(from, to)).expect("report mutation");
    }

    fn assert_error(&self, expected: &str) {
        let error = execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
            report: self.report_path().display().to_string(),
            agent_harness_evidence_path: None,
            pi_transaction_actor_paths: None,
        })
        .expect_err("unsafe proof path must fail");
        assert_eq!(error, expected);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn only_run_dir(root: &Path) -> PathBuf {
    std::fs::read_dir(root)
        .expect("fixture root")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir() && path.file_name().is_some_and(|name| name != "latest"))
        .expect("one run directory")
}

fn temp_root(stem: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kamn-7103-path-{stem}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
