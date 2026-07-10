use super::command_config::LiveTaskEvidencePaths;
use super::live_task_source_format::{
    artifact_digest, receipt_pid, reject_secret_path, require_string, require_task_state,
    sha256_hex, validate_shape, validate_task_id,
};
use super::verify_support::{extract_bool, extract_string, extract_u64};

pub(super) struct Sources {
    pub(super) handoff: String,
    pub(super) agent_a: String,
    pub(super) agent_b: String,
    pub(super) agent_c: String,
}

impl Sources {
    pub(super) fn values(&self) -> [&str; 4] {
        [&self.handoff, &self.agent_a, &self.agent_b, &self.agent_c]
    }
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

pub(super) fn read_sources(paths: &LiveTaskEvidencePaths) -> Result<Sources, String> {
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

pub(super) fn validate_sources(sources: &Sources) -> Result<ValidatedSources, String> {
    validate_source_shapes(sources)?;
    let task_id = extract_string(&sources.handoff, "task_id")?;
    validate_task_id(task_id.as_str())?;
    let validated = source_fields(sources, task_id)?;
    validate_source_agreement(sources, &validated)?;
    Ok(validated)
}

fn validate_source_shapes(sources: &Sources) -> Result<(), String> {
    validate_shape(&sources.handoff, "kamn.mvp.live-task-handoff.v1", 3)?;
    validate_shape(&sources.agent_a, "kamn.mvp.live-task-actor-receipt.v1", 6)?;
    validate_shape(&sources.agent_b, "kamn.mvp.live-task-actor-receipt.v1", 6)?;
    validate_shape(
        &sources.agent_c,
        "kamn.mvp.live-task-restricted-observation.v1",
        12,
    )
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
    require_distinct_processes(found)
}

fn require_distinct_processes(found: &ValidatedSources) -> Result<(), String> {
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
    require_restricted_public_policy(raw)?;
    require_string(raw, "source_handoff_digest", &found.handoff_digest)?;
    require_string(raw, "source_agent_a_receipt_digest", &found.agent_a_digest)?;
    require_string(raw, "source_agent_b_receipt_digest", &found.agent_b_digest)?;
    validate_public_commitment(raw, found)
}

fn require_restricted_public_policy(raw: &str) -> Result<(), String> {
    let valid = extract_string(raw, "view_scope")? == "restricted-public"
        && extract_u64(raw, "private_field_count")? == 0
        && extract_bool(raw, "private_payload_redacted")?;
    if valid {
        Ok(())
    } else {
        Err("live task Agent C observation violates restricted-public policy".to_owned())
    }
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
