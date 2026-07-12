use std::path::{Path, PathBuf};

use super::artifact_digest::attach_json_digest;
use super::command_config::LiveTaskEvidencePaths;
use super::live_task_sources::{read_sources, validate_sources, Sources, ValidatedSources};
use super::report::escape_json;

const BINDING_FILE: &str = "live-task-settlement-binding.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveTaskBinding {
    pub(crate) artifact_path: String,
    pub(crate) digest: String,
    pub(crate) task_id: String,
    pub(crate) transaction_id: String,
    pub(crate) terms_digest: String,
    pub(crate) agent_a_pid: u64,
    pub(crate) agent_b_pid: u64,
    pub(crate) agent_c_pid: u64,
}

pub(crate) fn create_live_task_binding(
    paths: &LiveTaskEvidencePaths,
    run_dir: &Path,
) -> Result<LiveTaskBinding, String> {
    let sources = read_sources(paths)?;
    let validated = validate_sources(&sources)?;
    let copied = copy_sources(run_dir, &sources)?;
    let artifact_path = run_dir.join("proof").join(BINDING_FILE);
    let json = binding_json(&validated, &copied)?;
    std::fs::write(&artifact_path, json.json.as_str())
        .map_err(|error| format!("failed to write live task settlement binding: {error}"))?;
    Ok(binding(artifact_path, json.digest, validated))
}

fn binding(path: PathBuf, digest: String, found: ValidatedSources) -> LiveTaskBinding {
    let transaction_id = found
        .transaction_id
        .unwrap_or_else(|| found.task_id.clone());
    let terms_digest = found.terms_digest.unwrap_or_else(|| digest.clone());
    LiveTaskBinding {
        artifact_path: path.display().to_string(),
        digest,
        task_id: found.task_id,
        transaction_id,
        terms_digest,
        agent_a_pid: found.agent_a_pid,
        agent_b_pid: found.agent_b_pid,
        agent_c_pid: found.agent_c_pid,
    }
}

fn copy_sources(run_dir: &Path, sources: &Sources) -> Result<[PathBuf; 4], String> {
    let proof = run_dir.join("proof");
    let paths = source_copy_paths(&proof);
    for (path, raw) in paths.iter().zip(sources.values()) {
        std::fs::write(path, raw)
            .map_err(|error| format!("failed to copy live task evidence: {error}"))?;
    }
    Ok(paths)
}

fn source_copy_paths(proof: &Path) -> [PathBuf; 4] {
    [
        proof.join("live-task-handoff.json"),
        proof.join("live-task-agent-a-receipt.json"),
        proof.join("live-task-agent-b-receipt.json"),
        proof.join("live-task-agent-c-observation.json"),
    ]
}

fn binding_json(
    found: &ValidatedSources,
    paths: &[PathBuf; 4],
) -> Result<super::artifact_digest::ArtifactJson, String> {
    attach_json_digest(format!(
        "{{\"schema_version\":\"kamn.mvp.live-task-settlement-binding.v1\",\"task_id\":\"{}\",\"state\":\"accepted\",\"agent_a_pi_process_id\":{},\"agent_b_pi_process_id\":{},\"agent_c_pi_process_id\":{},\"handoff_artifact\":\"{}\",\"agent_a_receipt_artifact\":\"{}\",\"agent_b_receipt_artifact\":\"{}\",\"agent_c_observation_artifact\":\"{}\",\"source_handoff_digest\":\"{}\",\"source_agent_a_receipt_digest\":\"{}\",\"source_agent_b_receipt_digest\":\"{}\",\"source_agent_c_observation_digest\":\"{}\",\"agent_c_public_commitment\":\"{}\",\"binding_digest\":\"\"}}",
        escape_json(found.task_id.as_str()), found.agent_a_pid, found.agent_b_pid, found.agent_c_pid,
        paths[0].display(), paths[1].display(), paths[2].display(), paths[3].display(),
        found.handoff_digest, found.agent_a_digest, found.agent_b_digest, found.agent_c_digest,
        found.public_commitment,
    ), "binding_digest")
}
