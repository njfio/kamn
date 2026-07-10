use kamn_e2e_harness::LiveTaskEvidencePaths;
use sha2::{Digest, Sha256};
use std::path::Path;

#[allow(dead_code)]
pub(crate) const TASK_ID: &str = "task-local-bound-7086";

#[allow(dead_code)]
pub(crate) fn write(root: &Path) -> LiveTaskEvidencePaths {
    write_for_task(root, TASK_ID)
}

pub(crate) fn write_for_task(root: &Path, task_id: &str) -> LiveTaskEvidencePaths {
    std::fs::create_dir_all(root).expect("task evidence root should be created");
    let handoff = root.join("live-task-handoff.json");
    let agent_a = root.join("live-task-agent-a.json");
    let agent_b = root.join("live-task-agent-b.json");
    let agent_c = root.join("live-task-agent-c.json");
    let handoff_digest = write_handoff(handoff.as_path(), task_id);
    let agent_a_digest = write_receipt(agent_a.as_path(), "agent_a", 101, task_id);
    let agent_b_digest = write_receipt(agent_b.as_path(), "agent_b", 202, task_id);
    write_observation(
        agent_c.as_path(),
        task_id,
        handoff_digest.as_str(),
        agent_a_digest.as_str(),
        agent_b_digest.as_str(),
    );
    LiveTaskEvidencePaths {
        handoff: handoff.display().to_string(),
        agent_a_receipt: agent_a.display().to_string(),
        agent_b_receipt: agent_b.display().to_string(),
        agent_c_observation: agent_c.display().to_string(),
    }
}

fn write_handoff(path: &Path, task_id: &str) -> String {
    let raw =
        format!(r#"{{"schema_version":"kamn.mvp.live-task-handoff.v1","task_id":"{task_id}"}}"#);
    write_with_digest(path, raw)
}

fn write_receipt(path: &Path, actor: &str, pid: u64, task_id: &str) -> String {
    let raw = format!(
        r#"{{"schema_version":"kamn.mvp.live-task-actor-receipt.v1","actor":"{actor}","task_id":"{task_id}","state":"accepted","pi_process_id":{pid}}}"#
    );
    write_with_digest(path, raw)
}

fn write_observation(path: &Path, task_id: &str, handoff: &str, agent_a: &str, agent_b: &str) {
    let public = format!(
        r#"{{"task_id":"{task_id}","state":"accepted","source_handoff_digest":"{handoff}","source_agent_a_receipt_digest":"{agent_a}","source_agent_b_receipt_digest":"{agent_b}"}}"#
    );
    let raw = format!(
        r#"{{"schema_version":"kamn.mvp.live-task-restricted-observation.v1","task_id":"{task_id}","state":"accepted","source_handoff_digest":"{handoff}","source_agent_a_receipt_digest":"{agent_a}","source_agent_b_receipt_digest":"{agent_b}","view_scope":"restricted-public","private_field_count":0,"private_payload_redacted":true,"agent_c_pi_process_id":303,"public_commitment":"{}"}}"#,
        sha256(public.as_str())
    );
    write_with_digest(path, raw);
}

fn write_with_digest(path: &Path, raw: String) -> String {
    let digest = sha256(raw.as_str());
    let json = format!(
        "{},\"artifact_digest\":\"{digest}\"}}",
        raw.trim_end_matches('}')
    );
    std::fs::write(path, json).expect("task evidence should be written");
    digest
}

fn sha256(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
