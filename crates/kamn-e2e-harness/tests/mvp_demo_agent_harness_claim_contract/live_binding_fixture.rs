use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use super::live_task_evidence;
use super::three_agent::{digest_field, with_digest};

pub(crate) const TASK_ID: &str = "tx-three-agent-7045";

pub(crate) struct BindingFixture {
    pub(crate) path: PathBuf,
    pub(crate) digest: String,
}

pub(crate) fn binding_fixture() -> &'static BindingFixture {
    static FIXTURE: OnceLock<BindingFixture> = OnceLock::new();
    FIXTURE.get_or_init(create_binding_fixture)
}

fn create_binding_fixture() -> BindingFixture {
    let root = std::env::temp_dir().join(format!(
        "kamn-7086-agent-harness-binding-{}",
        std::process::id()
    ));
    let sources = live_task_evidence::write_for_task(root.as_path(), TASK_ID);
    let path = root.join("live-task-settlement-binding.json");
    let raw = binding_json(&sources);
    let content = with_digest(raw, "binding_digest");
    std::fs::write(&path, &content).expect("binding fixture should be written");
    BindingFixture {
        path,
        digest: digest_field(content.as_str(), "binding_digest"),
    }
}

fn binding_json(sources: &kamn_e2e_harness::LiveTaskEvidencePaths) -> String {
    let handoff = read(sources.handoff.as_str());
    let agent_a = read(sources.agent_a_receipt.as_str());
    let agent_b = read(sources.agent_b_receipt.as_str());
    let agent_c = read(sources.agent_c_observation.as_str());
    format!(
        r#"{{"schema_version":"kamn.mvp.live-task-settlement-binding.v1","task_id":"{TASK_ID}","state":"accepted","agent_a_pi_process_id":101,"agent_b_pi_process_id":202,"agent_c_pi_process_id":303,"handoff_artifact":"{}","agent_a_receipt_artifact":"{}","agent_b_receipt_artifact":"{}","agent_c_observation_artifact":"{}","source_handoff_digest":"{}","source_agent_a_receipt_digest":"{}","source_agent_b_receipt_digest":"{}","source_agent_c_observation_digest":"{}","agent_c_public_commitment":"{}","binding_digest":""}}"#,
        sources.handoff,
        sources.agent_a_receipt,
        sources.agent_b_receipt,
        sources.agent_c_observation,
        digest_field(&handoff, "artifact_digest"),
        digest_field(&agent_a, "artifact_digest"),
        digest_field(&agent_b, "artifact_digest"),
        digest_field(&agent_c, "artifact_digest"),
        string_field(&agent_c, "public_commitment"),
    )
}

fn read(path: &str) -> String {
    std::fs::read_to_string(Path::new(path)).expect("live task fixture should be readable")
}

fn string_field(raw: &str, field: &str) -> String {
    digest_field(raw, field)
}
