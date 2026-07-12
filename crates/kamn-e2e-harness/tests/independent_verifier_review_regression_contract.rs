use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig,
};
use std::path::{Path, PathBuf};

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c33_verifier_rejects_conflicting_explorer_link() {
    let fixture = Fixture::new("conflicting-explorer-link");
    fixture.append_markdown(
        "\nhttps://explorer.solana.com/tx/foreign-signature?cluster=devnet\n",
    );

    fixture.assert_error("EXPLORER_LINK_INVALID");
}

#[test]
fn spec_c34_verifier_rejects_claim_and_index_removal() {
    let fixture = Fixture::new("claim-and-index-removal");
    fixture.remove_three_agent_claim();
    fixture.rename_agent_transaction_indexes();

    fixture.assert_error("AGENT_TRANSACTION_CLAIM_INVALID");
}

#[test]
fn spec_c35_view_index_mismatch_uses_public_scope_code() {
    let fixture = Fixture::new("view-index-public-code");
    let view = fixture.run_dir.join("proof/agent-c-verifier-view.json");
    let replacement = fixture.run_dir.join("proof/agent-a-view.json");
    fixture.replace_report_once(&view.display().to_string(), &replacement.display().to_string());

    fixture.assert_error("PROJECTION_SCOPE_INVALID");
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

    fn report_path(&self) -> PathBuf {
        self.root.join("latest/proof/report.json")
    }

    fn markdown_path(&self) -> PathBuf {
        self.root.join("latest/proof/report.md")
    }

    fn append_markdown(&self, suffix: &str) {
        let path = self.markdown_path();
        let mut raw = read(path.as_path());
        raw.push_str(suffix);
        std::fs::write(path, raw).expect("markdown mutation");
    }

    fn replace_report_once(&self, from: &str, to: &str) {
        let path = self.report_path();
        let raw = read(path.as_path());
        assert!(raw.contains(from), "fixture marker must exist: {from}");
        std::fs::write(path, raw.replacen(from, to, 1)).expect("report mutation");
    }

    fn rename_agent_transaction_indexes(&self) {
        for marker in [
            "three_agent_transcript",
            "live_task_settlement_binding",
        ] {
            self.replace_report_once(marker, format!("removed_{marker}").as_str());
        }
    }

    fn remove_three_agent_claim(&self) {
        let path = self.report_path();
        let raw = read(path.as_path());
        let start = raw
            .find(r#",{"id":"three_agent_escrow_verification"#)
            .expect("three-agent claim start");
        let tail = &raw[start + 1..];
        let end = tail
            .find(r#",{"id":"production_readiness"#)
            .expect("three-agent claim end");
        let reduced = format!("{}{}", &raw[..start], &tail[end..]);
        std::fs::write(path, reduced).expect("claim removal");
    }

    fn assert_error(&self, expected: &str) {
        let error = execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
            report: self.report_path().display().to_string(),
            agent_harness_evidence_path: None,
            pi_transaction_actor_paths: None,
        })
        .expect_err("tampered proof bundle must fail");
        assert_eq!(error, expected);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).expect("fixture file")
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
        "kamn-7103-review-{stem}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
