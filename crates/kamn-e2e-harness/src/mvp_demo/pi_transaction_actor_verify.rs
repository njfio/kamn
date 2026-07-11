use sha2::{Digest, Sha256};

use super::verify_support::{
    extract_bool, extract_optional_string, extract_string, extract_u64, validate_json_delimiters,
};
const SCHEMA: &str = "kamn.mvp.pi-transaction-actor.v1";
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
    Ok(format!(
        "{{\"task_id\":\"{}\",\"escrow_id\":\"{}\",\"settlement_tx_signature\":\"{}\"}}",
        actors[0].task_id, actors[0].escrow_id, actors[0].settlement_tx_signature
    ))
}
struct Actor {
    role: String,
    pi_process_id: u64,
    did: String,
    mcp_child_process_id: u64,
    first_request_id: u64,
    last_request_id: u64,
    response_digests: Vec<String>,
    projection_digest: String,
    task_id: String,
    transaction_id: String,
    escrow_id: String,
    amount_lamports: u64,
    network: String,
    settlement_tx_signature: String,
    settlement_commitment: String,
    public_commitment: String,
    view_scope: String,
    private_receipt_digest: Option<String>,
}
fn read_actor(path: &str, expected_role: &str) -> Result<Actor, String> {
    let raw = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read Pi transaction actor {path}: {error}"))?;
    validate_json_delimiters(raw.as_str()).map_err(|_| "PI_RUNTIME_RECEIPT_MISMATCH".to_owned())?;
    verify_artifact_digest(raw.as_str())?;
    let actor = parse_actor(raw.as_str())?;
    validate_actor(&actor, raw.as_str(), expected_role)?;
    Ok(actor)
}
fn parse_actor(raw: &str) -> Result<Actor, String> {
    if extract_string(raw, "schema_version")? != SCHEMA {
        return Err("PI_RUNTIME_RECEIPT_MISMATCH".to_owned());
    }
    Ok(Actor {
        role: extract_string(raw, "actor")?,
        pi_process_id: extract_u64(raw, "pi_process_id")?,
        did: extract_string(raw, "did")?,
        mcp_child_process_id: extract_u64(raw, "mcp_child_process_id")?,
        first_request_id: extract_u64(raw, "first_request_id")?,
        last_request_id: extract_u64(raw, "last_request_id")?,
        response_digests: extract_string_array(raw, "runtime_response_digests")?,
        projection_digest: extract_string(raw, "runtime_projection_digest")?,
        task_id: extract_string(raw, "task_id")?,
        transaction_id: extract_string(raw, "transaction_id")?,
        escrow_id: extract_string(raw, "escrow_id")?,
        amount_lamports: extract_u64(raw, "amount_lamports")?,
        network: extract_string(raw, "network")?,
        settlement_tx_signature: extract_string(raw, "settlement_tx_signature")?,
        settlement_commitment: extract_string(raw, "settlement_commitment")?,
        public_commitment: extract_string(raw, "public_commitment")?,
        view_scope: extract_string(raw, "view_scope")?,
        private_receipt_digest: extract_optional_string(raw, "private_receipt_digest"),
    })
}
fn validate_actor(actor: &Actor, raw: &str, expected_role: &str) -> Result<(), String> {
    if actor.role != expected_role || actor.did.is_empty() {
        return Err("PI_ACTOR_IDENTITY_INVALID".to_owned());
    }
    let count = actor.last_request_id.saturating_sub(actor.first_request_id) + 1;
    if actor.first_request_id == 0 || count != actor.response_digests.len() as u64 {
        return Err("PI_ACTOR_NONCE_STREAM_INVALID".to_owned());
    }
    validate_runtime_digests(actor)?;
    if extract_bool(raw, "handoff_authorized")? {
        return Err("PI_HANDOFF_AUTHORIZATION_FORBIDDEN".to_owned());
    }
    validate_scope(actor)
}
fn validate_runtime_digests(actor: &Actor) -> Result<(), String> {
    if !is_sha256(actor.projection_digest.as_str())
        || !actor.response_digests.iter().all(|value| is_sha256(value))
        || !actor.response_digests.contains(&actor.projection_digest)
    {
        return Err("PI_RUNTIME_RECEIPT_MISMATCH".to_owned());
    }
    Ok(())
}
fn validate_scope(actor: &Actor) -> Result<(), String> {
    if actor.role == "agent_c" {
        if actor.view_scope != "restricted-public" {
            return Err("PI_VERIFIER_PROJECTION_MISSING".to_owned());
        }
        if actor.private_receipt_digest.is_some() {
            return Err("PI_VERIFIER_PRIVATE_LEAK".to_owned());
        }
        return Ok(());
    }
    if actor.view_scope != "participant-private"
        || !actor
            .private_receipt_digest
            .as_deref()
            .is_some_and(is_sha256)
    {
        return Err("PI_RUNTIME_RECEIPT_MISMATCH".to_owned());
    }
    Ok(())
}

fn verify_artifact_digest(raw: &str) -> Result<(), String> {
    let trimmed = raw.trim();
    let marker = ",\"artifact_digest\":\"";
    let start = trimmed
        .rfind(marker)
        .ok_or_else(|| "PI_RUNTIME_RECEIPT_MISMATCH".to_owned())?;
    let unsigned = format!("{}}}", &trimmed[..start]);
    let expected = extract_string(trimmed, "artifact_digest")?;
    let actual = format!("sha256:{:x}", Sha256::digest(unsigned.as_bytes()));
    if expected == actual {
        return Ok(());
    }
    Err("PI_RUNTIME_RECEIPT_MISMATCH".to_owned())
}

fn extract_string_array(raw: &str, field: &str) -> Result<Vec<String>, String> {
    let marker = format!("\"{field}\":[\"");
    let start = raw
        .find(marker.as_str())
        .ok_or_else(|| "PI_RUNTIME_RECEIPT_MISMATCH".to_owned())?
        + marker.len();
    let end = raw[start..]
        .find("\"]")
        .ok_or_else(|| "PI_RUNTIME_RECEIPT_MISMATCH".to_owned())?;
    Ok(raw[start..start + end]
        .split("\",\"")
        .map(str::to_owned)
        .collect())
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

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
