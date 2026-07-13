use serde_json::Value;
use std::path::{Path, PathBuf};

use crate::generated_receipt_fixture::{artifact_digest, Fixture as BundleFixture};

pub(crate) struct Fixture {
    bundle: BundleFixture,
}

impl Fixture {
    pub(crate) fn new(stem: &str) -> Self {
        Self {
            bundle: BundleFixture::new(stem),
        }
    }

    pub(crate) fn remove_view(&self, agent: &str) {
        std::fs::remove_file(self.view_path(agent)).expect("view should remove");
    }

    pub(crate) fn replace_view_field(&self, agent: &str, field: &str, value: &str) {
        let path = self.view_path(agent);
        let mut view = read_json(path.as_path());
        view[field] = Value::String(value.to_owned());
        let refreshed = artifact_digest::with_digest(
            serde_json::to_string(&view).expect("view JSON"),
            "view_digest",
        );
        std::fs::write(path, &refreshed).expect("refreshed view");
        let digest = artifact_digest::digest_field(&refreshed, "view_digest");
        self.bundle
            .replace_claim_field(view_digest_field(agent), digest.as_str());
        self.refresh_receipt_view(agent, digest.as_str());
    }

    pub(crate) fn verify(&self) -> Result<String, String> {
        self.bundle.verify()
    }

    fn refresh_receipt_view(&self, agent: &str, view_digest: &str) {
        let path = self.receipt_path(agent);
        let mut receipt = read_json(path.as_path());
        receipt["view_digest"] = Value::String(view_digest.to_owned());
        let refreshed = artifact_digest::with_digest(
            serde_json::to_string(&receipt).expect("receipt JSON"),
            "receipt_digest",
        );
        std::fs::write(path, &refreshed).expect("refreshed receipt");
        let digest = artifact_digest::digest_field(&refreshed, "receipt_digest");
        self.bundle
            .replace_claim_field(receipt_digest_field(agent), digest.as_str());
    }

    fn view_path(&self, agent: &str) -> PathBuf {
        self.bundle
            .run_dir()
            .join("proof")
            .join(format!("{}-view.json", agent.replace('_', "-")))
    }

    fn receipt_path(&self, agent: &str) -> PathBuf {
        self.bundle.run_dir().join("proof").join(format!(
            "{}-observation-receipt.json",
            agent.replace('_', "-")
        ))
    }
}

fn view_digest_field(agent: &str) -> &str {
    match agent {
        "agent_a" => "agent_a_view_digest",
        "agent_b" => "agent_b_view_digest",
        _ => "agent_c_verifier_view_digest",
    }
}

fn receipt_digest_field(agent: &str) -> &str {
    match agent {
        "agent_a" => "agent_a_observation_receipt_digest",
        "agent_b" => "agent_b_observation_receipt_digest",
        _ => "agent_c_verifier_observation_receipt_digest",
    }
}

fn read_json(path: &Path) -> Value {
    let raw = std::fs::read_to_string(path).expect("fixture JSON should read");
    serde_json::from_str(raw.as_str()).expect("fixture JSON should parse")
}
