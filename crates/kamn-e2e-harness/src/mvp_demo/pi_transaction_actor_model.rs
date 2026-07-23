use serde::{Deserialize, Serialize};

use super::pi_transaction_actor_authority::validate_authority;

const SCHEMA: &str = "kamn.mvp.pi-transaction-actor.v2";
const AUTHORITY_ERROR: &str = "PI_SERVICE_AUTHORITY_MISMATCH";
const TRANSPORT_ERROR: &str = "PI_TRANSPORT_PROVENANCE_INVALID";

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
    transport_response_digests: Vec<String>,
    pub(super) service_profile_commitment: String,
    pub(super) service_receipts: Vec<ServiceReceipt>,
    pub(super) task_id: String,
    pub(super) transaction_id: String,
    pub(super) escrow_id: String,
    pub(super) amount_lamports: u64,
    pub(super) network: String,
    pub(super) settlement_tx_signature: String,
    pub(super) settlement_commitment: String,
    pub(super) receipt_chain_commitment: String,
    pub(super) public_commitment: String,
    view_scope: String,
    participant_role: Option<String>,
    source_handoff_digest: String,
    handoff_authorized: bool,
    artifact_digest: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ServiceReceipt {
    pub(super) actor_did: String,
    pub(super) tool: String,
    pub(super) action: String,
    pub(super) resource_id: String,
    pub(super) resulting_state: String,
    pub(super) service_receipt_id: String,
    pub(super) service_receipt_digest: String,
}

pub(super) fn parse_and_validate_actor(raw: &str, expected_role: &str) -> Result<Actor, String> {
    let actor: Actor = serde_json::from_str(raw).map_err(|_| authority_error())?;
    validate_actor(&actor, expected_role)?;
    Ok(actor)
}

fn validate_actor(actor: &Actor, expected_role: &str) -> Result<(), String> {
    if actor.schema_version != SCHEMA || actor.actor != expected_role || actor.did.is_empty() {
        return Err(authority_error());
    }
    validate_transport(actor)?;
    validate_authority(actor)?;
    if actor.handoff_authorized {
        return Err(authority_error());
    }
    validate_shared_digests(actor)?;
    validate_scope(actor)
}

fn validate_transport(actor: &Actor) -> Result<(), String> {
    let count = actor.last_request_id.saturating_sub(actor.first_request_id) + 1;
    let valid = actor.first_request_id > 0
        && count == actor.transport_response_digests.len() as u64
        && !actor.transport_response_digests.is_empty()
        && actor
            .transport_response_digests
            .iter()
            .all(|digest| is_sha256(digest));
    valid.then_some(()).ok_or_else(transport_error)
}

fn validate_shared_digests(actor: &Actor) -> Result<(), String> {
    let valid = is_sha256(&actor.source_handoff_digest)
        && is_sha256(&actor.receipt_chain_commitment)
        && is_sha256(&actor.public_commitment)
        && is_sha256(&actor.artifact_digest);
    valid.then_some(()).ok_or_else(authority_error)
}

fn validate_scope(actor: &Actor) -> Result<(), String> {
    if actor.actor == "agent_c" {
        let valid = actor.view_scope == "restricted-public" && actor.participant_role.is_none();
        return valid.then_some(()).ok_or_else(authority_error);
    }
    let expected = if actor.actor == "agent_a" {
        "creator"
    } else {
        "provider"
    };
    let valid = actor.view_scope == "participant-private"
        && actor.participant_role.as_deref() == Some(expected);
    valid.then_some(()).ok_or_else(authority_error)
}

pub(super) fn is_sha256(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(super) fn authority_error() -> String {
    AUTHORITY_ERROR.to_owned()
}

fn transport_error() -> String {
    TRANSPORT_ERROR.to_owned()
}
