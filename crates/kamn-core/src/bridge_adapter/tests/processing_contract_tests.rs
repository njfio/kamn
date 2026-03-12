use crate::bridge_adapter::{
    AllowAllBridgePolicy, BridgeAdapter, BridgeAdapterEngine, BridgeAdapterError, BridgeDirection,
    BridgeInboundEnvelope, BridgeOutboundEnvelope, BridgeOutboundRequest, BridgePlatform,
    NormalizedInboundMessage, PassThroughBridgeAdapter,
};

#[derive(Debug, Clone, Copy)]
struct BadAdapter;

impl BridgeAdapter for BadAdapter {
    fn platform(&self) -> BridgePlatform {
        BridgePlatform::Telegram
    }

    fn bridge_agent_did(&self) -> &str {
        "kamn:did:agent:bridge-telegram"
    }

    fn normalize_inbound(
        &self,
        inbound: &BridgeInboundEnvelope,
    ) -> Result<NormalizedInboundMessage, BridgeAdapterError> {
        Ok(NormalizedInboundMessage {
            bridge_message_id: format!("telegram:{}", inbound.external_message_id),
            sender_handle: inbound.external_sender_id.clone(),
            source_channel: inbound.external_channel_id.clone(),
            target_agent_did: inbound.target_agent_did.clone(),
            body: inbound.body.clone(),
            received_at: inbound.received_at.clone(),
            received_at_unix: inbound.received_at_unix,
            platform: self.platform(),
        })
    }

    fn translate_outbound(
        &self,
        request: &BridgeOutboundRequest,
    ) -> Result<BridgeOutboundEnvelope, BridgeAdapterError> {
        Ok(BridgeOutboundEnvelope {
            request_id: format!("{}-mutated", request.request_id),
            destination_channel_id: request.destination_channel_id.clone(),
            payload: "{\"message\":\"x\"}".to_owned(),
            platform: self.platform(),
        })
    }
}

#[test]
fn process_outbound_rejects_request_id_mutation() {
    let engine = BridgeAdapterEngine::new(BadAdapter, AllowAllBridgePolicy::new());
    let request = outbound_request("telegram:channel:a", "hi");
    assert_eq!(
        engine.process_outbound(&request),
        Err(BridgeAdapterError::OutboundRequestIdMismatch {
            expected: "req-1".to_owned(),
            actual: "req-1-mutated".to_owned(),
        })
    );
}

#[test]
fn process_inbound_to_envelope_rejects_empty_recipient_keys() {
    let adapter = valid_adapter();
    let engine = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());
    let inbound = inbound_envelope();
    assert_eq!(
        engine.process_inbound_to_envelope(
            &inbound,
            1_707_383_600,
            Vec::new(),
            "2026-02-08T10:00:00Z",
            1,
        ),
        Err(BridgeAdapterError::EmptyField("recipient_keys"))
    );
}

#[test]
fn process_inbound_rejects_duplicate_message_id() {
    let engine = BridgeAdapterEngine::new(valid_adapter(), AllowAllBridgePolicy::new());
    let inbound = inbound_envelope();
    assert!(engine.process_inbound(&inbound, 1_707_383_700).is_ok());
    assert_eq!(
        engine.process_inbound(&inbound, 1_707_383_700),
        Err(BridgeAdapterError::DuplicateInboundMessageId(
            "discord:ext-9".to_owned()
        ))
    );
}

#[test]
fn process_outbound_rejects_duplicate_request_id() {
    let engine = BridgeAdapterEngine::new(valid_adapter(), AllowAllBridgePolicy::new());
    let request = outbound_request("discord:channel:1", "hello");
    assert!(engine.process_outbound(&request).is_ok());
    assert_eq!(
        engine.process_outbound(&request),
        Err(BridgeAdapterError::DuplicateOutboundRequestId(
            "req-1".to_owned()
        ))
    );
}

#[test]
fn display_policy_denied_error_mentions_direction() {
    let error = BridgeAdapterError::PolicyDenied {
        direction: BridgeDirection::Inbound,
        reason: "blocked".to_owned(),
    };
    assert_eq!(error.to_string(), "policy denied Inbound traffic: blocked");
}

fn valid_adapter() -> PassThroughBridgeAdapter {
    PassThroughBridgeAdapter::new(BridgePlatform::Discord, "kamn:did:agent:bridge-discord-1")
        .expect("adapter should be valid")
}

fn inbound_envelope() -> BridgeInboundEnvelope {
    BridgeInboundEnvelope {
        external_message_id: "ext-9".to_owned(),
        external_sender_id: "discord:user-1".to_owned(),
        external_channel_id: "discord:channel:1".to_owned(),
        target_agent_did: "kamn:did:agent:planner-1".to_owned(),
        body: "hello".to_owned(),
        received_at: "2026-02-08T09:00:00Z".to_owned(),
        received_at_unix: 1_707_383_600,
    }
}

fn outbound_request(destination_channel_id: &str, body: &str) -> BridgeOutboundRequest {
    BridgeOutboundRequest {
        request_id: "req-1".to_owned(),
        from_agent_did: "kamn:did:agent:planner-1".to_owned(),
        destination_channel_id: destination_channel_id.to_owned(),
        body: body.to_owned(),
        created_at: "2026-02-08T09:00:00Z".to_owned(),
    }
}
