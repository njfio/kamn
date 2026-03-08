use super::config::{AGENTS_READ_SCOPE, LiveTransportConfig};
use super::task_escrow::{LiveEscrowAlias, LiveTaskAlias, deterministic_u64_tag};
use crate::{
    AgentDid, MessageId, SdkError, ServiceRequestAuth, service_signature_for_fields,
    service_signer_public_key_for_fields,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Default)]
pub(crate) struct LiveTransportState {
    pub(crate) sender_nonces: HashMap<String, u64>,
    pub(crate) artifact_ids: HashMap<u64, String>,
    pub(crate) message_ids: HashMap<u64, String>,
    pub(crate) escrow_aliases: HashMap<u64, LiveEscrowAlias>,
    pub(crate) task_aliases: HashMap<u64, LiveTaskAlias>,
}

pub(crate) fn build_auth(
    state: &Arc<Mutex<LiveTransportState>>,
    config: &LiveTransportConfig,
    sender_did: &AgentDid,
    body: &str,
    scope: Option<&str>,
) -> Result<ServiceRequestAuth, SdkError> {
    let nonce = next_nonce(state, sender_did)?;
    let signature = service_signature_for_fields(
        sender_did,
        nonce,
        config.chain_id.as_str(),
        config.chain_version.as_str(),
        body,
    )?;
    let signer_public_key_hex = service_signer_public_key_for_fields()?;
    ServiceRequestAuth::new_with_signer_public_key_and_scope(
        sender_did.clone(),
        nonce,
        signature,
        Some(signer_public_key_hex.as_str()),
        scope,
    )
}

pub(crate) fn build_agents_read_auth(
    state: &Arc<Mutex<LiveTransportState>>,
    config: &LiveTransportConfig,
) -> Result<ServiceRequestAuth, SdkError> {
    build_agents_read_auth_with_body(state, config, "")
}

pub(crate) fn build_agents_read_auth_with_body(
    state: &Arc<Mutex<LiveTransportState>>,
    config: &LiveTransportConfig,
    body: &str,
) -> Result<ServiceRequestAuth, SdkError> {
    build_auth(
        state,
        config,
        &config.requester_did,
        body,
        Some(AGENTS_READ_SCOPE),
    )
}

pub(crate) fn remember_message_id(
    state: &Arc<Mutex<LiveTransportState>>,
    service_message_id: &str,
) -> Result<MessageId, SdkError> {
    if service_message_id.trim().is_empty() {
        return Err(SdkError::TransportFailure(
            "service returned empty message_id in send response",
        ));
    }

    let numeric_id = deterministic_u64_tag(service_message_id);
    let mut guard = state
        .lock()
        .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
    if let Some(existing) = guard.message_ids.get(&numeric_id) {
        if existing != service_message_id {
            return Err(SdkError::Conflict(
                "service message id collision detected in sdk numeric alias map",
            ));
        }
    } else {
        guard
            .message_ids
            .insert(numeric_id, service_message_id.to_owned());
    }
    Ok(MessageId(numeric_id))
}

fn next_nonce(
    state: &Arc<Mutex<LiveTransportState>>,
    sender_did: &AgentDid,
) -> Result<u64, SdkError> {
    let mut guard = state
        .lock()
        .map_err(|_| SdkError::TransportFailure("live transport state lock poisoned"))?;
    let nonce = guard
        .sender_nonces
        .entry(sender_did.as_str().to_owned())
        .or_insert(0);
    if *nonce == u64::MAX {
        return Err(SdkError::Conflict(
            "live transport nonce exhausted for sender",
        ));
    }
    *nonce += 1;
    Ok(*nonce)
}
