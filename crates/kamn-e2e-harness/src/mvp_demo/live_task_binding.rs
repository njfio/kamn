use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

use super::artifact_digest::{attach_json_digest, digest_json_without_field};
use super::command_config::LiveTaskEvidencePaths;
use super::report::escape_json;
use super::verify_support::{extract_bool, extract_string, extract_u64, validate_json_delimiters};

const BINDING_FILE: &str = "live-task-settlement-binding.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LiveTaskBinding {
    pub(crate) artifact_path: String,
    pub(crate) digest: String,
    pub(crate) task_id: String,
    pub(crate) agent_a_pid: u64,
    pub(crate) agent_b_pid: u64,
    pub(crate) agent_c_pid: u64,
}

struct Sources {
    handoff: String,
    agent_a: String,
    agent_b: String,
    agent_c: String,
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
    Ok(LiveTaskBinding {
        artifact_path: artifact_path.display().to_string(),
        digest: json.digest,
        task_id: validated.task_id,
        agent_a_pid: validated.agent_a_pid,
        agent_b_pid: validated.agent_b_pid,
        agent_c_pid: validated.agent_c_pid,
    })
}

pub(crate) struct ValidatedSources {
    pub(crate) task_id: String,
    pub(crate) agent_a_pid: u64,
    pub(crate) agent_b_pid: u64,
    pub(crate) agent_c_pid: u64,
    pub(crate) handoff_digest: String,
    pub(crate) agent_a_digest: String,
    pub(crate) agent_b_digest: String,
    pub(crate) agent_c_digest: String,
    pub(crate) public_commitment: String,
}

pub(crate) fn validate_live_task_evidence(
    paths: &LiveTaskEvidencePaths,
) -> Result<ValidatedSources, String> {
    validate_sources(&read_sources(paths)?)
}

fn read_sources(paths: &LiveTaskEvidencePaths) -> Result<Sources, String> {
    Ok(Sources {
        handoff: read_source(paths.handoff.as_str())?,
        agent_a: read_source(paths.agent_a_receipt.as_str())?,
        agent_b: read_source(paths.agent_b_receipt.as_str())?,
        agent_c: read_source(paths.agent_c_observation.as_str())?,
    })
}

fn read_source(path: &str) -> Result<String, String> {
    reject_secret_path(path)?;
    std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read live task evidence {path}: {error}"))
}

fn validate_sources(sources: &Sources) -> Result<ValidatedSources, String> {
    validate_shape(&sources.handoff, "kamn.mvp.live-task-handoff.v1", 3)?;
    validate_shape(&sources.agent_a, "kamn.mvp.live-task-actor-receipt.v1", 6)?;
    validate_shape(&sources.agent_b, "kamn.mvp.live-task-actor-receipt.v1", 6)?;
    validate_shape(
        &sources.agent_c,
        "kamn.mvp.live-task-restricted-observation.v1",
        12,
    )?;
    let task_id = extract_string(&sources.handoff, "task_id")?;
    validate_task_id(task_id.as_str())?;
    let validated = source_fields(sources, task_id)?;
    validate_source_agreement(sources, &validated)?;
    Ok(validated)
}

fn source_fields(sources: &Sources, task_id: String) -> Result<ValidatedSources, String> {
    Ok(ValidatedSources {
        task_id,
        agent_a_pid: receipt_pid(&sources.agent_a, "agent_a")?,
        agent_b_pid: receipt_pid(&sources.agent_b, "agent_b")?,
        agent_c_pid: extract_u64(&sources.agent_c, "agent_c_pi_process_id")?,
        handoff_digest: artifact_digest(&sources.handoff)?,
        agent_a_digest: artifact_digest(&sources.agent_a)?,
        agent_b_digest: artifact_digest(&sources.agent_b)?,
        agent_c_digest: artifact_digest(&sources.agent_c)?,
        public_commitment: extract_string(&sources.agent_c, "public_commitment")?,
    })
}

fn validate_source_agreement(sources: &Sources, found: &ValidatedSources) -> Result<(), String> {
    require_task_state(&sources.agent_a, found.task_id.as_str())?;
    require_task_state(&sources.agent_b, found.task_id.as_str())?;
    require_agent_c_policy(&sources.agent_c, found)?;
    if found.agent_a_pid == found.agent_b_pid
        || found.agent_a_pid == found.agent_c_pid
        || found.agent_b_pid == found.agent_c_pid
    {
        return Err("live task settlement binding requires three distinct Pi processes".to_owned());
    }
    Ok(())
}

fn require_agent_c_policy(raw: &str, found: &ValidatedSources) -> Result<(), String> {
    require_task_state(raw, found.task_id.as_str())?;
    if extract_string(raw, "view_scope")? != "restricted-public"
        || extract_u64(raw, "private_field_count")? != 0
        || !extract_bool(raw, "private_payload_redacted")?
    {
        return Err("live task Agent C observation violates restricted-public policy".to_owned());
    }
    require_string(raw, "source_handoff_digest", &found.handoff_digest)?;
    require_string(raw, "source_agent_a_receipt_digest", &found.agent_a_digest)?;
    require_string(raw, "source_agent_b_receipt_digest", &found.agent_b_digest)?;
    validate_public_commitment(raw, found)
}

fn validate_public_commitment(raw: &str, found: &ValidatedSources) -> Result<(), String> {
    let canonical = format!(
        "{{\"task_id\":\"{}\",\"state\":\"accepted\",\"source_handoff_digest\":\"{}\",\"source_agent_a_receipt_digest\":\"{}\",\"source_agent_b_receipt_digest\":\"{}\"}}",
        found.task_id, found.handoff_digest, found.agent_a_digest, found.agent_b_digest
    );
    if found.public_commitment == sha256_hex(canonical.as_str()) {
        return Ok(());
    }
    Err(format!(
        "live task Agent C public commitment mismatch: {}",
        extract_string(raw, "public_commitment")?
    ))
}

fn copy_sources(run_dir: &Path, sources: &Sources) -> Result<[PathBuf; 4], String> {
    let proof = run_dir.join("proof");
    let paths = [
        proof.join("live-task-handoff.json"),
        proof.join("live-task-agent-a-receipt.json"),
        proof.join("live-task-agent-b-receipt.json"),
        proof.join("live-task-agent-c-observation.json"),
    ];
    for (path, raw) in paths.iter().zip([
        &sources.handoff,
        &sources.agent_a,
        &sources.agent_b,
        &sources.agent_c,
    ]) {
        std::fs::write(path, raw)
            .map_err(|error| format!("failed to copy live task evidence: {error}"))?;
    }
    Ok(paths)
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

fn validate_shape(raw: &str, schema: &str, field_count: usize) -> Result<(), String> {
    validate_json_delimiters(raw).map_err(|_| "malformed live task evidence JSON".to_owned())?;
    require_string(raw, "schema_version", schema)?;
    if raw.matches("\":").count() != field_count {
        return Err("live task evidence field allowlist mismatch".to_owned());
    }
    artifact_digest(raw).map(|_| ())
}

fn artifact_digest(raw: &str) -> Result<String, String> {
    let expected = extract_string(raw, "artifact_digest")?;
    let actual = digest_json_without_field(raw, "artifact_digest")?;
    if actual.strip_prefix("sha256:") == Some(expected.as_str()) {
        return Ok(expected);
    }
    Err("live task evidence artifact digest mismatch".to_owned())
}

fn receipt_pid(raw: &str, actor: &str) -> Result<u64, String> {
    require_string(raw, "actor", actor)?;
    let pid = extract_u64(raw, "pi_process_id")?;
    if pid == 0 {
        Err("live task receipt Pi process ID must be positive".to_owned())
    } else {
        Ok(pid)
    }
}

fn require_task_state(raw: &str, task_id: &str) -> Result<(), String> {
    require_string(raw, "task_id", task_id)?;
    require_string(raw, "state", "accepted")
}

fn require_string(raw: &str, field: &str, expected: &str) -> Result<(), String> {
    if extract_string(raw, field)? == expected {
        Ok(())
    } else {
        Err(format!("live task evidence {field} mismatch"))
    }
}

fn validate_task_id(value: &str) -> Result<(), String> {
    if !value.is_empty()
        && value.len() <= 200
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:-".contains(&byte))
    {
        Ok(())
    } else {
        Err("live task evidence task ID is invalid".to_owned())
    }
}

fn reject_secret_path(path: &str) -> Result<(), String> {
    let lower = path.to_ascii_lowercase();
    if [
        ".kamn/devnet",
        "auth.json",
        ".env",
        "keypair",
        "id_rsa",
        "oauth",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        Err("refusing secret-like live task evidence path".to_owned())
    } else {
        Ok(())
    }
}

fn sha256_hex(value: &str) -> String {
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
