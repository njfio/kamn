use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig,
};
use std::path::{Path, PathBuf};

#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c01_report_contains_signature_derived_devnet_explorer_link() {
    let fixture = Fixture::new("explorer-link");
    let signature = "devnet-signature-111";

    assert!(fixture
        .markdown()
        .contains(format!("https://explorer.solana.com/tx/{signature}?cluster=devnet").as_str()));
}

#[test]
fn spec_c02_verifier_rejects_artifact_outside_proof_bundle() {
    let fixture = Fixture::new("path-boundary");
    let foreign = fixture.root.join("foreign-devnet-output.txt");
    std::fs::copy(fixture.devnet_log(), &foreign).expect("foreign fixture copy");
    fixture.replace_report_path(fixture.devnet_log().as_path(), foreign.as_path());

    fixture.assert_error("PROOF_ARTIFACT_PATH_INVALID");
}

#[test]
fn spec_c03_verifier_rejects_settlement_artifact_fact_drift() {
    let fixture = Fixture::new("settlement-drift");
    fixture.replace_file(
        fixture.devnet_log().as_path(),
        "recipient_pubkey=FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe",
        "recipient_pubkey=ForeignRecipient111111111111111111111111111",
    );

    fixture.assert_error("SETTLEMENT_EVIDENCE_INVALID");
}

#[test]
fn spec_c04_report_indexes_authoritative_offline_settlement_evidence() {
    let fixture = Fixture::new("offline-evidence");

    assert!(fixture
        .report()
        .contains(r#""devnet_settlement_evidence":"#));
}

#[test]
fn spec_c05_verifier_rejects_downgraded_agent_transaction_claim() {
    let fixture = Fixture::new("claim-downgrade");
    fixture.replace_report(
        r#""id":"three_agent_escrow_verification","label":"devnet-backed""#,
        r#""id":"three_agent_escrow_verification","label":"placeholder""#,
    );

    fixture.assert_error("AGENT_TRANSACTION_CLAIM_INVALID");
}

#[test]
fn spec_c06_verifier_rejects_explorer_link_drift() {
    let fixture = Fixture::new("explorer-drift");
    fixture.replace_file(
        fixture.markdown_path().as_path(),
        "?cluster=devnet",
        "?cluster=mainnet-beta",
    );

    fixture.assert_error("EXPLORER_LINK_INVALID");
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

    fn report(&self) -> String {
        read(self.report_path().as_path())
    }

    fn markdown(&self) -> String {
        read(self.markdown_path().as_path())
    }

    fn report_path(&self) -> PathBuf {
        self.root.join("latest/proof/report.json")
    }

    fn markdown_path(&self) -> PathBuf {
        self.root.join("latest/proof/report.md")
    }

    fn devnet_log(&self) -> PathBuf {
        self.run_dir.join("proof/devnet-settlement-output.txt")
    }

    fn replace_report(&self, from: &str, to: &str) {
        self.replace_file(self.report_path().as_path(), from, to);
    }

    fn replace_report_path(&self, from: &Path, to: &Path) {
        self.replace_report(&from.display().to_string(), &to.display().to_string());
    }

    fn replace_file(&self, path: &Path, from: &str, to: &str) {
        let source = read(path);
        assert!(source.contains(from), "fixture marker must exist: {from}");
        std::fs::write(path, source.replace(from, to)).expect("fixture mutation");
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
        "kamn-7103-{stem}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
