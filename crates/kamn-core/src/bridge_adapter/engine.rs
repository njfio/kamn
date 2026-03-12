mod envelope;
mod replay;

use std::cell::RefCell;
use std::collections::BTreeSet;

use crate::CanonicalMessageEnvelope;

use super::{
    BridgeAdapter, BridgeAdapterError, BridgeInboundEnvelope, BridgeOutboundEnvelope,
    BridgeOutboundRequest, BridgePolicyHook, NormalizedInboundMessage,
};
use crate::bridge_adapter::engine::envelope::build_canonical_envelope;
use crate::bridge_adapter::engine::replay::{
    ensure_fresh, record_inbound, record_outbound, validate_envelope_inputs,
    validate_translated_outbound,
};
use crate::bridge_adapter::support::parse_agent_did;
use crate::bridge_adapter::support::{
    BRIDGE_ADAPTER_INVALID_BRIDGE_AGENT_DID_REASON_CODE, DEFAULT_MAX_INBOUND_AGE_SECS,
    validate_inbound_envelope, validate_normalized_inbound, validate_outbound_request,
    validate_timestamp,
};

/// Bridge orchestration engine that enforces policy, freshness, and replay guards.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeAdapterEngine<A, P> {
    adapter: A,
    policy: P,
    max_inbound_age_secs: u64,
    seen_inbound_message_ids: RefCell<BTreeSet<String>>,
    seen_outbound_request_ids: RefCell<BTreeSet<String>>,
}

impl<A, P> BridgeAdapterEngine<A, P>
where
    A: BridgeAdapter,
    P: BridgePolicyHook,
{
    pub fn new(adapter: A, policy: P) -> Self {
        Self::with_inbound_freshness_window(adapter, policy, DEFAULT_MAX_INBOUND_AGE_SECS)
    }

    pub fn with_inbound_freshness_window(adapter: A, policy: P, max_inbound_age_secs: u64) -> Self {
        Self {
            adapter,
            policy,
            max_inbound_age_secs,
            seen_inbound_message_ids: RefCell::new(BTreeSet::new()),
            seen_outbound_request_ids: RefCell::new(BTreeSet::new()),
        }
    }

    pub fn process_inbound(
        &self,
        inbound: &BridgeInboundEnvelope,
        observed_at_unix: u64,
    ) -> Result<NormalizedInboundMessage, BridgeAdapterError> {
        parse_agent_did(
            self.adapter.bridge_agent_did(),
            "bridge_agent_did",
            BRIDGE_ADAPTER_INVALID_BRIDGE_AGENT_DID_REASON_CODE,
        )?;
        let _ = validate_inbound_envelope(inbound)?;
        validate_timestamp("observed_at_unix", observed_at_unix)?;
        let normalized = self.adapter.normalize_inbound(inbound)?;
        validate_normalized_inbound(&normalized)?;
        self.policy.authorize_inbound(&normalized)?;
        ensure_fresh(&normalized, observed_at_unix, self.max_inbound_age_secs)?;
        record_inbound(
            &self.seen_inbound_message_ids,
            normalized.bridge_message_id.clone(),
        )?;
        Ok(normalized)
    }

    pub fn process_outbound(
        &self,
        request: &BridgeOutboundRequest,
    ) -> Result<BridgeOutboundEnvelope, BridgeAdapterError> {
        let validated_request = validate_outbound_request(request)?;
        self.policy.authorize_outbound(request)?;
        let translated = self.adapter.translate_outbound(request)?;
        validate_translated_outbound(request, &translated)?;
        record_outbound(
            &self.seen_outbound_request_ids,
            validated_request.request_id,
        )?;
        Ok(translated)
    }

    pub fn process_inbound_to_envelope(
        &self,
        inbound: &BridgeInboundEnvelope,
        observed_at_unix: u64,
        recipient_keys: Vec<String>,
        expires: &str,
        nonce: u64,
    ) -> Result<CanonicalMessageEnvelope, BridgeAdapterError> {
        validate_envelope_inputs(&recipient_keys, expires, nonce)?;
        let normalized = self.process_inbound(inbound, observed_at_unix)?;
        let envelope = build_canonical_envelope(
            self.adapter.bridge_agent_did(),
            normalized,
            recipient_keys,
            expires,
            nonce,
        );
        envelope
            .validate()
            .map_err(|error| BridgeAdapterError::Envelope(error.to_string()))?;
        Ok(envelope)
    }
}
