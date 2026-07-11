use std::path::Path;

use kamn_agent_lib::KamnAgentHandle;

use super::super::devnet_settlement_json::json_string_value;

const AGENT_A_FILE: &str = "runtime-agent-a-participant-view.json";
const AGENT_B_FILE: &str = "runtime-agent-b-participant-view.json";
const AGENT_C_FILE: &str = "runtime-agent-c-verifier-view.json";

pub(super) fn capture_runtime_task_projections(
    endpoint: &str,
    run_dir: &Path,
    creator: &KamnAgentHandle,
    provider: &KamnAgentHandle,
    task_id: &str,
) -> Result<(), String> {
    let verifier = super::agent(endpoint, "kamn-mvp-devnet-settlement-verifier")?;
    super::register(&verifier, "verifier")?;
    let agent_a = participant_projection(creator, task_id, "creator")?;
    let agent_b = participant_projection(provider, task_id, "provider")?;
    let agent_c = verifier_projection(&verifier, task_id)?;
    validate_runtime_projection_bodies(&agent_a, &agent_b, &agent_c, task_id)?;
    write_projection(run_dir, AGENT_A_FILE, agent_a.as_str())?;
    write_projection(run_dir, AGENT_B_FILE, agent_b.as_str())?;
    write_projection(run_dir, AGENT_C_FILE, agent_c.as_str())
}

fn participant_projection(
    agent: &KamnAgentHandle,
    task_id: &str,
    role: &str,
) -> Result<String, String> {
    agent
        .query_participant_task_projection(task_id)
        .map_err(|error| format!("failed to query {role} participant projection: {error}"))
}

fn verifier_projection(agent: &KamnAgentHandle, task_id: &str) -> Result<String, String> {
    agent
        .query_verifier_task_projection(task_id)
        .map_err(|error| format!("failed to query verifier projection: {error}"))
}

fn validate_runtime_projection_bodies(
    agent_a: &str,
    agent_b: &str,
    agent_c: &str,
    task_id: &str,
) -> Result<(), String> {
    require_field(agent_a, "task_id", task_id, "Agent A")?;
    require_field(agent_b, "task_id", task_id, "Agent B")?;
    require_field(agent_c, "task_id", task_id, "Agent C")?;
    require_matching_commitments(agent_a, agent_b, agent_c)?;
    require_private_boundary(agent_a, agent_b, agent_c)
}

fn require_field(raw: &str, field: &str, expected: &str, agent: &str) -> Result<(), String> {
    if json_string_value(raw, field).as_deref() == Ok(expected) {
        return Ok(());
    }
    Err(format!("{agent} projection {field} mismatch"))
}

fn require_matching_commitments(a: &str, b: &str, c: &str) -> Result<(), String> {
    let commitment = json_string_value(a, "public_commitment")?;
    if json_string_value(b, "public_commitment").as_deref() == Ok(commitment.as_str())
        && json_string_value(c, "public_commitment").as_deref() == Ok(commitment.as_str())
    {
        return Ok(());
    }
    Err("runtime projection public commitment mismatch".to_owned())
}

fn require_private_boundary(a: &str, b: &str, c: &str) -> Result<(), String> {
    if a.contains("\"task_receipt_ids\":[")
        && b.contains("\"task_receipt_ids\":[")
        && !c.contains("\"task_receipt_ids\"")
        && !c.contains("\"completion_evidence_digest\"")
    {
        return Ok(());
    }
    Err("runtime projection private boundary mismatch".to_owned())
}

fn write_projection(run_dir: &Path, file: &str, raw: &str) -> Result<(), String> {
    let path = run_dir.join("proof").join(file);
    std::fs::write(path.as_path(), raw).map_err(|error| {
        format!(
            "failed to write runtime projection {}: {error}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::validate_runtime_projection_bodies;

    #[test]
    fn runtime_projection_validation_enforces_private_boundary() {
        let a = participant("creator");
        let b = participant("provider");
        let c = r#"{"task_id":"task-1","public_commitment":"fnv1a64:a"}"#;
        validate_runtime_projection_bodies(a.as_str(), b.as_str(), c, "task-1")
            .expect("valid runtime projections");
        assert!(
            validate_runtime_projection_bodies(a.as_str(), b.as_str(), a.as_str(), "task-1")
                .is_err()
        );
    }

    fn participant(role: &str) -> String {
        format!(
            r#"{{"task_id":"task-1","participant_role":"{role}","public_commitment":"fnv1a64:a","task_receipt_ids":["receipt-1"]}}"#
        )
    }
}
