use std::path::Path;

pub(super) fn write_live_evidence(root: &Path) -> [String; 4] {
    let staging = root.join("staging");
    std::fs::create_dir_all(&staging).expect("staging root");
    let paths = [
        "handoff.json",
        "task-agent-a.json",
        "task-agent-b.json",
        "task-agent-c.json",
    ]
    .map(|name| staging.join(name));
    let handoff = write_digest(
        &paths[0],
        r#"{"schema_version":"kamn.mvp.live-task-handoff.v1","task_id":"task-local-bound-7086"}"#,
    );
    let agent_a = write_receipt(&paths[1], "agent_a", 101);
    let agent_b = write_receipt(&paths[2], "agent_b", 202);
    write_observation(&paths[3], &handoff, &agent_a, &agent_b);
    paths.map(|path| path.display().to_string())
}

fn write_receipt(path: &Path, actor: &str, pid: u64) -> String {
    write_digest(
        path,
        &format!(
            r#"{{"schema_version":"kamn.mvp.live-task-actor-receipt.v1","actor":"{actor}","task_id":"task-local-bound-7086","state":"accepted","pi_process_id":{pid}}}"#
        ),
    )
}

fn write_observation(path: &Path, handoff: &str, agent_a: &str, agent_b: &str) {
    let public = format!(
        r#"{{"task_id":"task-local-bound-7086","state":"accepted","source_handoff_digest":"{handoff}","source_agent_a_receipt_digest":"{agent_a}","source_agent_b_receipt_digest":"{agent_b}"}}"#
    );
    let raw = format!(
        r#"{{"schema_version":"kamn.mvp.live-task-restricted-observation.v1","task_id":"task-local-bound-7086","state":"accepted","source_handoff_digest":"{handoff}","source_agent_a_receipt_digest":"{agent_a}","source_agent_b_receipt_digest":"{agent_b}","view_scope":"restricted-public","private_field_count":0,"private_payload_redacted":true,"agent_c_pi_process_id":303,"public_commitment":"{}"}}"#,
        sha256(&public)
    );
    write_digest(path, &raw);
}

fn write_digest(path: &Path, raw: &str) -> String {
    let digest = sha256(raw);
    let artifact = format!(
        "{},\"artifact_digest\":\"{digest}\"}}",
        raw.trim_end_matches('}')
    );
    std::fs::write(path, artifact).expect("evidence artifact");
    digest
}

fn sha256(value: &str) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
