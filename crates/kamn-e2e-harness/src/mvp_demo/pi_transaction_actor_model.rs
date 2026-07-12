use serde::Deserialize;

use super::pi_transaction_public_result::{validate_public_result, PublicResult};

const SCHEMA: &str = "kamn.mvp.pi-transaction-actor.v1";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Actor {
    schema_version: String,
    pub(super) actor: String,
    pub(super) pi_process_id: u64,
    pub(super) did: String,
    pub(super) mcp_child_process_id: u64,
    first_request_id: u64,
    last_request_id: u64,
    runtime_response_digests: Vec<String>,
    pub(super) runtime_response_receipts: Vec<RuntimeReceipt>,
    runtime_projection_digest: String,
    pub(super) task_id: String,
    pub(super) transaction_id: String,
    pub(super) escrow_id: String,
    pub(super) amount_lamports: u64,
    pub(super) network: String,
    pub(super) settlement_tx_signature: String,
    pub(super) settlement_commitment: String,
    pub(super) public_commitment: String,
    view_scope: String,
    participant_role: Option<String>,
    private_receipt_digest: Option<String>,
    source_handoff_digest: String,
    handoff_authorized: bool,
    artifact_digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RuntimeReceipt {
    pub(super) request_id: u64,
    pub(super) tool: String,
    pub(super) outcome: Outcome,
    pub(super) digest: String,
    pub(super) public_result: PublicResult,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(super) enum Outcome {
    Success,
    Error,
}

pub(super) fn parse_and_validate_actor(raw: &str, expected_role: &str) -> Result<Actor, String> {
    let actor: Actor = serde_json::from_str(raw).map_err(|_| mismatch())?;
    validate_actor(&actor, expected_role)?;
    Ok(actor)
}

fn validate_actor(actor: &Actor, expected_role: &str) -> Result<(), String> {
    if actor.schema_version != SCHEMA || actor.actor != expected_role || actor.did.is_empty() {
        return Err("PI_ACTOR_IDENTITY_INVALID".to_owned());
    }
    let count = actor.last_request_id.saturating_sub(actor.first_request_id) + 1;
    if actor.first_request_id == 0 || count != actor.runtime_response_digests.len() as u64 {
        return Err("PI_ACTOR_NONCE_STREAM_INVALID".to_owned());
    }
    validate_runtime_receipts(actor)?;
    if actor.handoff_authorized {
        return Err("PI_HANDOFF_AUTHORIZATION_FORBIDDEN".to_owned());
    }
    if !is_sha256(actor.source_handoff_digest.as_str())
        || !is_sha256(actor.artifact_digest.as_str())
    {
        return Err(mismatch());
    }
    validate_scope(actor)
}

fn validate_runtime_receipts(actor: &Actor) -> Result<(), String> {
    if actor.runtime_response_receipts.len() != actor.runtime_response_digests.len()
        || !is_sha256(actor.runtime_projection_digest.as_str())
    {
        return Err(mismatch());
    }
    for (index, receipt) in actor.runtime_response_receipts.iter().enumerate() {
        if receipt.request_id != actor.first_request_id + index as u64
            || receipt.digest != actor.runtime_response_digests[index]
            || !is_sha256(receipt.digest.as_str())
        {
            return Err(mismatch());
        }
        validate_public_result(&receipt.public_result, receipt.outcome == Outcome::Error)?;
    }
    require_operations(actor)?;
    validate_projection_receipt(actor)
}

fn validate_projection_receipt(actor: &Actor) -> Result<(), String> {
    let projection_tool = if actor.actor == "agent_c" {
        "query_verifier_task_projection"
    } else {
        "query_participant_task_projection"
    };
    if has_success(
        actor,
        projection_tool,
        Some(actor.runtime_projection_digest.as_str()),
    ) {
        return Ok(());
    }
    Err(mismatch())
}

fn require_operations(actor: &Actor) -> Result<(), String> {
    let required: &[&str] = match actor.actor.as_str() {
        "agent_a" => &["register", "create_task", "fund_escrow", "release_escrow"],
        "agent_b" => &["register", "accept_task", "complete_task"],
        "agent_c" => &["register"],
        _ => return Err("PI_ACTOR_IDENTITY_INVALID".to_owned()),
    };
    if required.iter().all(|tool| has_success(actor, tool, None)) {
        return Ok(());
    }
    Err(mismatch())
}

fn has_success(actor: &Actor, tool: &str, digest: Option<&str>) -> bool {
    actor
        .runtime_response_receipts
        .iter()
        .filter(|receipt| receipt.tool == tool && receipt.outcome == Outcome::Success)
        .filter(|receipt| digest.is_none_or(|expected| receipt.digest == expected))
        .count()
        == 1
}

fn validate_scope(actor: &Actor) -> Result<(), String> {
    if actor.actor == "agent_c" {
        return validate_verifier_scope(actor);
    }
    validate_participant_scope(actor)
}

fn validate_verifier_scope(actor: &Actor) -> Result<(), String> {
    if actor.view_scope != "restricted-public" {
        return Err("PI_VERIFIER_PROJECTION_MISSING".to_owned());
    }
    if actor.private_receipt_digest.is_some() || actor.participant_role.is_some() {
        return Err("PI_VERIFIER_PRIVATE_LEAK".to_owned());
    }
    Ok(())
}

fn validate_participant_scope(actor: &Actor) -> Result<(), String> {
    let expected_role = if actor.actor == "agent_a" {
        "creator"
    } else {
        "provider"
    };
    if actor.view_scope != "participant-private"
        || actor.participant_role.as_deref() != Some(expected_role)
        || !actor
            .private_receipt_digest
            .as_deref()
            .is_some_and(is_sha256)
    {
        return Err(mismatch());
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn mismatch() -> String {
    "PI_RUNTIME_RECEIPT_MISMATCH".to_owned()
}
