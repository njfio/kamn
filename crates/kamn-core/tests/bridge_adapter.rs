use kamn_core::{
    AllowAllBridgePolicy, BridgeAdapter, BridgeAdapterEngine, BridgeAdapterError, BridgeDirection,
    BridgeInboundEnvelope, BridgeOutboundEnvelope, BridgeOutboundRequest, BridgePlatform,
    NormalizedInboundMessage, PassThroughBridgeAdapter,
};

fn inbound_sample() -> BridgeInboundEnvelope {
    BridgeInboundEnvelope {
        external_message_id: "ext-42".to_owned(),
        external_sender_id: "discord:user-99".to_owned(),
        external_channel_id: "discord:channel:alpha".to_owned(),
        target_agent_did: "kamn:did:agent:planner-1".to_owned(),
        body: "Need a sprint plan".to_owned(),
        received_at: "2026-02-08T10:00:00Z".to_owned(),
        received_at_unix: 1_716_620_000,
    }
}

fn outbound_sample() -> BridgeOutboundRequest {
    BridgeOutboundRequest {
        request_id: "req-7".to_owned(),
        from_agent_did: "kamn:did:agent:planner-1".to_owned(),
        destination_channel_id: "discord:channel:alpha".to_owned(),
        body: "Plan accepted".to_owned(),
        created_at: "2026-02-08T10:01:00Z".to_owned(),
    }
}

#[test]
fn inbound_is_normalized_deterministically() {
    let adapter =
        PassThroughBridgeAdapter::new(BridgePlatform::Discord, "kamn:did:agent:bridge-discord-1")
            .expect("adapter should build");
    let engine = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());

    let normalized = engine
        .process_inbound(&inbound_sample(), 1_716_620_100)
        .expect("inbound should normalize");

    assert_eq!(normalized.bridge_message_id, "discord:ext-42");
    assert_eq!(normalized.target_agent_did, "kamn:did:agent:planner-1");
    assert_eq!(normalized.platform, BridgePlatform::Discord);
}

#[test]
fn outbound_translation_is_deterministic() {
    let adapter =
        PassThroughBridgeAdapter::new(BridgePlatform::Discord, "kamn:did:agent:bridge-discord-1")
            .expect("adapter should build");
    let engine = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());

    let translated = engine
        .process_outbound(&outbound_sample())
        .expect("outbound should translate");

    assert_eq!(translated.request_id, "req-7");
    assert_eq!(translated.destination_channel_id, "discord:channel:alpha");
    assert_eq!(translated.platform, BridgePlatform::Discord);
    assert!(translated.payload.contains("\"message\":\"Plan accepted\""));
}

#[test]
fn inbound_can_be_projected_to_canonical_envelope() {
    let adapter =
        PassThroughBridgeAdapter::new(BridgePlatform::Discord, "kamn:did:agent:bridge-discord-1")
            .expect("adapter should build");
    let engine = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());

    let envelope = engine
        .process_inbound_to_envelope(
            &inbound_sample(),
            1_716_620_100,
            vec!["recipient-key-1".to_owned()],
            "2026-02-08T11:00:00Z",
            91,
        )
        .expect("envelope projection should succeed");

    assert_eq!(envelope.envelope.from, "kamn:did:agent:bridge-discord-1");
    assert_eq!(
        envelope.envelope.to,
        vec!["kamn:did:agent:planner-1".to_owned()]
    );
    assert_eq!(envelope.header.message_type, "Request");
}

#[derive(Debug, Clone, Copy)]
struct DenyOutboundPolicy;

impl kamn_core::BridgePolicyHook for DenyOutboundPolicy {
    fn authorize_inbound(
        &self,
        _normalized: &NormalizedInboundMessage,
    ) -> Result<(), BridgeAdapterError> {
        Ok(())
    }

    fn authorize_outbound(
        &self,
        _request: &BridgeOutboundRequest,
    ) -> Result<(), BridgeAdapterError> {
        Err(BridgeAdapterError::PolicyDenied {
            direction: BridgeDirection::Outbound,
            reason: "blocked by test policy".to_owned(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct MismatchedOutboundAdapter;

impl BridgeAdapter for MismatchedOutboundAdapter {
    fn platform(&self) -> BridgePlatform {
        BridgePlatform::Custom("manual".to_owned())
    }

    fn bridge_agent_did(&self) -> &str {
        "kamn:did:agent:bridge-manual"
    }

    fn normalize_inbound(
        &self,
        inbound: &BridgeInboundEnvelope,
    ) -> Result<NormalizedInboundMessage, BridgeAdapterError> {
        Ok(NormalizedInboundMessage {
            bridge_message_id: format!("manual:{}", inbound.external_message_id),
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
            payload: "{\"message\":\"mutated\"}".to_owned(),
            platform: self.platform(),
        })
    }
}

#[test]
fn regression_rejects_policy_denied_outbound() {
    let adapter =
        PassThroughBridgeAdapter::new(BridgePlatform::Discord, "kamn:did:agent:bridge-discord-1")
            .expect("adapter should build");
    let engine = BridgeAdapterEngine::new(adapter, DenyOutboundPolicy);

    assert_eq!(
        engine.process_outbound(&outbound_sample()),
        Err(BridgeAdapterError::PolicyDenied {
            direction: BridgeDirection::Outbound,
            reason: "blocked by test policy".to_owned(),
        })
    );
}

#[test]
fn regression_rejects_adapter_outbound_request_id_mutation() {
    let engine = BridgeAdapterEngine::new(MismatchedOutboundAdapter, AllowAllBridgePolicy::new());

    assert_eq!(
        engine.process_outbound(&outbound_sample()),
        Err(BridgeAdapterError::OutboundRequestIdMismatch {
            expected: "req-7".to_owned(),
            actual: "req-7-mutated".to_owned(),
        })
    );
}

#[test]
fn regression_rejects_duplicate_inbound_event_replay() {
    // Regression: #423
    let adapter =
        PassThroughBridgeAdapter::new(BridgePlatform::Discord, "kamn:did:agent:bridge-discord-1")
            .expect("adapter should build");
    let engine = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());

    engine
        .process_inbound(&inbound_sample(), 1_716_620_100)
        .expect("first inbound event should normalize");
    assert_eq!(
        engine.process_inbound(&inbound_sample(), 1_716_620_100),
        Err(BridgeAdapterError::DuplicateInboundMessageId(
            "discord:ext-42".to_owned()
        ))
    );
}

#[test]
fn regression_rejects_duplicate_outbound_request_replay() {
    // Regression: #433
    let adapter =
        PassThroughBridgeAdapter::new(BridgePlatform::Discord, "kamn:did:agent:bridge-discord-1")
            .expect("adapter should build");
    let engine = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());

    engine
        .process_outbound(&outbound_sample())
        .expect("first outbound request should translate");
    let replay_error = engine
        .process_outbound(&outbound_sample())
        .expect_err("duplicate outbound request id should be rejected");
    assert_eq!(
        replay_error.to_string(),
        "duplicate outbound request id: req-7"
    );
}

#[test]
fn regression_rejects_stale_inbound_event() {
    // Regression: #546
    let adapter =
        PassThroughBridgeAdapter::new(BridgePlatform::Discord, "kamn:did:agent:bridge-discord-1")
            .expect("adapter should build");
    let engine = BridgeAdapterEngine::new(adapter, AllowAllBridgePolicy::new());

    assert_eq!(
        engine.process_inbound(&inbound_sample(), 1_716_621_000),
        Err(BridgeAdapterError::StaleInboundMessage {
            bridge_message_id: "discord:ext-42".to_owned(),
            received_at_unix: 1_716_620_000,
            observed_at_unix: 1_716_621_000,
            max_age_secs: 300,
        })
    );
}
