#![allow(dead_code)]

use kamn_e2e_harness::{
    execute_mvp_demo_contract, execute_verify_mvp_demo_contract, VerifyMvpDemoCommandConfig,
};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[path = "artifact_digest.rs"]
#[allow(dead_code)]
mod artifact_digest;

use crate::mvp_demo_command;

pub(crate) struct Fixture {
    output_root: PathBuf,
    report: PathBuf,
}

impl Fixture {
    pub(crate) fn new(stem: &str) -> Self {
        let output_root = temp_root(stem);
        execute_mvp_demo_contract(&mvp_demo_command::devnet_required_demo_config(&output_root))
            .expect("canonical receipt fixture should generate");
        let report = only_run_dir(&output_root).join("proof/report.json");
        Self {
            output_root,
            report,
        }
    }

    pub(crate) fn remove_receipt_references(&self) {
        let mut raw = std::fs::read_to_string(&self.report).expect("report should read");
        let report = read_json(self.report.as_path());
        for agent in ["agent_a", "agent_b", "agent_c_verifier"] {
            raw = remove_report_field(raw, &report["artifacts"], receipt_entry(agent));
            raw = remove_report_field(raw, transaction_claim(&report), receipt_artifact(agent));
            raw = remove_report_field(raw, transaction_claim(&report), receipt_digest(agent));
        }
        std::fs::write(&self.report, raw).expect("report should write");
    }

    pub(crate) fn tamper_agent_a_receipt(&self) {
        let path = self.receipt_path("agent_a");
        let mut receipt = read_json(path.as_path());
        receipt["tamper_marker"] = Value::String("agent-a-receipt-tamper".to_owned());
        write_json(path.as_path(), &receipt);
    }

    pub(crate) fn remove_transcript(&self) {
        std::fs::remove_file(self.transcript_path()).expect("transcript should remove");
    }

    pub(crate) fn tamper_transcript(&self) {
        let path = self.transcript_path();
        let mut transcript = read_json(path.as_path());
        transcript["tamper_marker"] = Value::String("transcript-tampered".to_owned());
        write_json(path.as_path(), &transcript);
    }

    pub(crate) fn replace_transcript_field(&self, field: &str, value: &str) {
        let path = self.transcript_path();
        let mut transcript = read_json(path.as_path());
        transcript[field] = Value::String(value.to_owned());
        let refreshed = artifact_digest::with_digest(
            serde_json::to_string(&transcript).expect("transcript JSON"),
            "transcript_digest",
        );
        std::fs::write(path, &refreshed).expect("refreshed transcript");
        let digest = artifact_digest::digest_field(&refreshed, "transcript_digest");
        self.replace_claim_field("three_agent_transcript_digest", digest.as_str());
    }

    pub(crate) fn replace_receipt_field(&self, agent: &str, field: &str, value: &str) {
        let path = self.receipt_path(agent);
        let mut receipt = read_json(path.as_path());
        receipt[field] = Value::String(value.to_owned());
        let refreshed = artifact_digest::with_digest(
            serde_json::to_string(&receipt).expect("receipt JSON"),
            "receipt_digest",
        );
        std::fs::write(&path, &refreshed).expect("refreshed receipt");
        let digest = artifact_digest::digest_field(&refreshed, "receipt_digest");
        self.replace_claim_digest(agent, digest.as_str());
    }

    pub(crate) fn verify(&self) -> Result<String, String> {
        execute_verify_mvp_demo_contract(&VerifyMvpDemoCommandConfig {
            report: self.report.display().to_string(),
            agent_harness_evidence_path: None,
            pi_transaction_actor_paths: None,
        })
    }

    fn receipt_path(&self, agent: &str) -> PathBuf {
        only_run_dir(&self.output_root)
            .join("proof")
            .join(format!("{agent}-observation-receipt.json").replace('_', "-"))
    }

    fn transcript_path(&self) -> PathBuf {
        only_run_dir(&self.output_root).join("proof/three-agent-transcript.json")
    }

    fn replace_claim_digest(&self, agent: &str, digest: &str) {
        self.replace_claim_field(receipt_digest(agent).as_str(), digest);
    }

    fn replace_claim_field(&self, field: &str, value: &str) {
        let raw = std::fs::read_to_string(&self.report).expect("report should read");
        let report = read_json(self.report.as_path());
        let old = transaction_claim(&report)[field]
            .as_str()
            .expect("claim string field");
        let updated = raw.replace(
            format!(r#""{field}":"{old}""#).as_str(),
            format!(r#""{field}":"{value}""#).as_str(),
        );
        std::fs::write(&self.report, updated).expect("report should write");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.output_root);
    }
}

fn remove_report_field(raw: String, object: &Value, field: String) -> String {
    let value = object[&field].as_str().expect("string report field");
    raw.replace(format!(r#","{field}":"{value}""#).as_str(), "")
}

fn receipt_entry(agent: &str) -> String {
    format!("{agent}_observation_receipt")
}

fn receipt_artifact(agent: &str) -> String {
    format!("{agent}_observation_receipt_artifact")
}

fn receipt_digest(agent: &str) -> String {
    format!("{agent}_observation_receipt_digest")
}

fn transaction_claim(report: &Value) -> &Value {
    report["claim_matrix"]
        .as_array()
        .and_then(|claims| {
            claims
                .iter()
                .find(|claim| claim["id"] == "three_agent_escrow_verification")
        })
        .expect("three-agent transaction claim")
}

fn read_json(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path).expect("fixture JSON should read");
    serde_json::from_str(raw.as_str()).expect("fixture JSON should parse")
}

fn write_json(path: &Path, value: &Value) {
    let raw = serde_json::to_string(value).expect("fixture JSON should serialize");
    std::fs::write(path, raw).expect("fixture JSON should write");
}

fn only_run_dir(root: &Path) -> PathBuf {
    std::fs::read_dir(root)
        .expect("fixture root")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_dir() && path.file_name().is_some_and(|name| name != "latest"))
        .expect("one immutable run directory")
}

pub(crate) fn temp_root(stem: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "kamn-7114-receipts-{stem}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ))
}
