use sha2::{Digest, Sha256};

use super::pi_transaction_actor_model::{parse_and_validate_actor, Actor};

const ROLES: [&str; 3] = ["agent_a", "agent_b", "agent_c"];

/// Verifies three independent Pi actor artifacts and returns shared evidence.
pub fn verify_pi_transaction_actor_paths(paths: &[String; 3]) -> Result<String, String> {
    let actors = [
        read_actor(paths[0].as_str(), ROLES[0])?,
        read_actor(paths[1].as_str(), ROLES[1])?,
        read_actor(paths[2].as_str(), ROLES[2])?,
    ];
    require_distinct_u64(&actors, |actor| actor.pi_process_id)?;
    require_distinct_u64(&actors, |actor| actor.mcp_child_process_id)?;
    require_distinct_dids(&actors)?;
    require_shared_facts(&actors)?;
    serde_json::to_string(&serde_json::json!({
        "task_id": actors[0].task_id,
        "escrow_id": actors[0].escrow_id,
        "settlement_tx_signature": actors[0].settlement_tx_signature,
    }))
    .map_err(|_| mismatch())
}

fn read_actor(path: &str, expected_role: &str) -> Result<Actor, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read Pi transaction actor {path}: {error}"))?;
    verify_artifact_digest(raw.as_str())?;
    parse_and_validate_actor(raw.as_str(), expected_role)
}

fn verify_artifact_digest(raw: &str) -> Result<(), String> {
    let trimmed = raw.trim();
    let marker = ",\"artifact_digest\":\"";
    let start = trimmed.rfind(marker).ok_or_else(mismatch)?;
    if !trimmed.ends_with('}') {
        return Err(mismatch());
    }
    let expected_start = start + marker.len();
    let expected_end = trimmed[expected_start..].find('"').ok_or_else(mismatch)? + expected_start;
    if &trimmed[expected_end..] != "\"}" {
        return Err(mismatch());
    }
    let unsigned = format!("{}}}", &trimmed[..start]);
    let actual = format!("sha256:{:x}", Sha256::digest(unsigned.as_bytes()));
    if trimmed[expected_start..expected_end] == actual {
        return Ok(());
    }
    Err(mismatch())
}

fn require_distinct_u64(actors: &[Actor; 3], field: fn(&Actor) -> u64) -> Result<(), String> {
    let values = actors.iter().map(field).collect::<Vec<_>>();
    if values.iter().all(|value| *value > 0)
        && values[0] != values[1]
        && values[0] != values[2]
        && values[1] != values[2]
    {
        return Ok(());
    }
    Err("PI_ACTOR_PROCESS_REUSED".to_owned())
}

fn require_distinct_dids(actors: &[Actor; 3]) -> Result<(), String> {
    if actors[0].did != actors[1].did
        && actors[0].did != actors[2].did
        && actors[1].did != actors[2].did
    {
        return Ok(());
    }
    Err("PI_ACTOR_IDENTITY_INVALID".to_owned())
}

fn require_shared_facts(actors: &[Actor; 3]) -> Result<(), String> {
    let first = shared_fact_key(&actors[0]);
    if shared_fact_key(&actors[1]) == first && shared_fact_key(&actors[2]) == first {
        return Ok(());
    }
    Err("PI_TRANSACTION_FACT_MISMATCH".to_owned())
}

fn shared_fact_key(actor: &Actor) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}|{}|{}",
        actor.task_id,
        actor.transaction_id,
        actor.escrow_id,
        actor.amount_lamports,
        actor.network,
        actor.settlement_tx_signature,
        actor.settlement_commitment,
        actor.public_commitment
    )
}

fn mismatch() -> String {
    "PI_RUNTIME_RECEIPT_MISMATCH".to_owned()
}
