use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[path = "support/artifact_digest.rs"]
#[allow(dead_code)]
mod artifact_digest;
#[path = "support/mvp_demo_command.rs"]
mod mvp_demo_command;

#[test]
fn spec_c11_foreign_signature_fails_after_digest_refresh() {
    assert_mutation_fails("foreign-signature", |evidence| {
        evidence["settlement_tx_signature"] = Value::String("foreign-signature-7103".to_owned());
        evidence["persisted_settlement_tx_signature"] =
            Value::String("foreign-signature-7103".to_owned());
    });
}

#[test]
fn spec_c12_foreign_recipient_fails_after_digest_refresh() {
    assert_mutation_fails("foreign-recipient", |evidence| {
        evidence["recipient_pubkey"] =
            Value::String("ForeignRecipient111111111111111111111111111".to_owned());
    });
}

#[test]
fn spec_c13_amount_drift_fails_after_digest_refresh() {
    assert_mutation_fails("amount-drift", |evidence| {
        evidence["lamports"] = Value::from(999_999_u64);
    });
}

#[test]
fn spec_c14_nonfinal_commitment_fails_after_digest_refresh() {
    assert_mutation_fails("commitment-drift", |evidence| {
        evidence["settlement_commitment"] = Value::String("confirmed".to_owned());
    });
}

#[test]
fn spec_c15_balance_movement_drift_fails_after_digest_refresh() {
    assert_mutation_fails("balance-drift", |evidence| {
        evidence["recipient_balance_after"] = Value::from(2_500_999_999_u64);
    });
}

#[test]
fn spec_c19_payer_drift_fails_after_digest_refresh() {
    assert_mutation_fails("payer-drift", |evidence| {
        evidence["payer_pubkey"] =
            Value::String("ForeignPayer1111111111111111111111111111111".to_owned());
    });
}

#[test]
fn spec_c20_task_binding_drift_fails_after_digest_refresh() {
    assert_mutation_fails("task-binding-drift", |evidence| {
        evidence["task_binding_digest"] = Value::String(format!("sha256:{}", "f".repeat(64)));
    });
}

#[test]
fn spec_c25_report_indexes_raw_authoritative_solana_response() {
    let fixture = Fixture::new("raw-rpc-index");
    let report = std::fs::read_to_string(fixture.report_path()).expect("report");

    assert!(report.contains(r#""solana_confirmation_response":"#));
}

#[test]
fn spec_c26_raw_solana_response_tamper_fails_closed() {
    let fixture = Fixture::new("raw-rpc-tamper");
    let path = fixture.raw_solana_response();
    let raw = std::fs::read_to_string(&path).expect("raw Solana response");
    std::fs::write(path, raw.replace("finalized", "confirmed")).expect("tampered Solana response");

    let error = fixture.verify().expect_err("raw RPC tamper must fail");
    assert_eq!(error, "SETTLEMENT_EVIDENCE_INVALID");
}

fn assert_mutation_fails(stem: &str, mutate: impl FnOnce(&mut Value)) {
    let fixture = Fixture::new(stem);
    fixture.mutate_evidence(mutate);
    let error = fixture
        .verify()
        .expect_err("foreign settlement facts must fail");
    assert_eq!(error, "SETTLEMENT_EVIDENCE_INVALID");
}

struct Fixture {
    root: PathBuf,
    evidence: PathBuf,
}

impl Fixture {
    fn new(stem: &str) -> Self {
        let root = temp_root(stem);
        execute_mvp_demo_contract(&mvp_demo_command::devnet_required_demo_config(&root))
            .expect("valid devnet fixture");
        let evidence = only_run_dir(&root).join("proof/settlement-evidence.json");
        Self { root, evidence }
    }

    fn mutate_evidence(&self, mutate: impl FnOnce(&mut Value)) {
        let raw = std::fs::read_to_string(&self.evidence).expect("settlement evidence");
        let mut value: Value = serde_json::from_str(raw.as_str()).expect("evidence JSON");
        mutate(&mut value);
        let unsigned = serde_json::to_string(&value).expect("mutated evidence JSON");
        let refreshed = artifact_digest::with_digest(unsigned, "evidence_digest");
        std::fs::write(&self.evidence, refreshed).expect("mutated settlement evidence");
    }

    fn report_path(&self) -> PathBuf {
        self.root.join("latest/proof/report.json")
    }

    fn raw_solana_response(&self) -> PathBuf {
        only_run_dir(&self.root).join("proof/solana-confirmation-response.json")
    }

    fn verify(&self) -> Result<String, String> {
        execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
            report: self
                .root
                .join("latest/proof/report.json")
                .display()
                .to_string(),
            agent_harness_evidence_path: None,
            pi_transaction_actor_paths: None,
        })
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
        "kamn-7103-settlement-{stem}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
