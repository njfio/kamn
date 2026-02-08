use kamn_core::{
    BridgeInboundEnvelope, BridgeOutboundRequest, BridgePlatform, DiscordBridgeConfig,
    DiscordBridgeEngine, DiscordBridgeError, DiscordInboundRequest,
};
use std::collections::{BTreeMap, BTreeSet};

fn config() -> DiscordBridgeConfig {
    let mut channel_routes = BTreeMap::new();
    channel_routes.insert(
        "discord:channel:ops".to_owned(),
        "kamn:did:agent:listener-target-1".to_owned(),
    );

    DiscordBridgeConfig {
        bridge_agent_did: "kamn:did:agent:bridge-discord-1".to_owned(),
        authorized_listener_dids: ["kamn:did:agent:listener-1".to_owned()]
            .into_iter()
            .collect::<BTreeSet<_>>(),
        authorized_approver_dids: [
            "kamn:did:agent:approver-1".to_owned(),
            "kamn:did:agent:approver-2".to_owned(),
            "kamn:did:agent:approver-3".to_owned(),
        ]
        .into_iter()
        .collect::<BTreeSet<_>>(),
        required_approvals: 2,
        channel_routes,
    }
}

fn inbound(target: &str) -> BridgeInboundEnvelope {
    BridgeInboundEnvelope {
        external_message_id: "discord-msg-1".to_owned(),
        external_sender_id: "discord:user-9".to_owned(),
        external_channel_id: "discord:channel:ops".to_owned(),
        target_agent_did: target.to_owned(),
        body: "run diagnostics".to_owned(),
        received_at: "2026-02-08T04:00:00Z".to_owned(),
    }
}

fn outbound() -> BridgeOutboundRequest {
    BridgeOutboundRequest {
        request_id: "discord-outbound-1".to_owned(),
        from_agent_did: "kamn:did:agent:processor-1".to_owned(),
        destination_channel_id: "discord:channel:ops".to_owned(),
        body: "status: green".to_owned(),
        created_at: "2026-02-08T04:10:00Z".to_owned(),
    }
}

#[test]
fn approver_quorum_dispatches_outbound() {
    let engine = DiscordBridgeEngine::new(config()).expect("engine should build");

    let dispatch = engine
        .process_outbound_with_approvals(
            &outbound(),
            vec![
                "kamn:did:agent:approver-1".to_owned(),
                "kamn:did:agent:approver-2".to_owned(),
            ],
        )
        .expect("quorum approvals should dispatch outbound");

    assert_eq!(dispatch.envelope.request_id, "discord-outbound-1");
    assert_eq!(dispatch.envelope.platform, BridgePlatform::Discord);
    assert_eq!(dispatch.approval.required_approvals, 2);
    assert_eq!(dispatch.approval.approved_by.len(), 2);
}

#[test]
fn outbound_rejects_insufficient_approvals() {
    let engine = DiscordBridgeEngine::new(config()).expect("engine should build");

    assert_eq!(
        engine.process_outbound_with_approvals(
            &outbound(),
            vec!["kamn:did:agent:approver-1".to_owned()],
        ),
        Err(DiscordBridgeError::InsufficientApprovals {
            required: 2,
            provided: 1,
        })
    );
}

#[test]
fn outbound_rejects_unauthorized_approver() {
    let engine = DiscordBridgeEngine::new(config()).expect("engine should build");

    assert_eq!(
        engine.process_outbound_with_approvals(
            &outbound(),
            vec![
                "kamn:did:agent:approver-1".to_owned(),
                "kamn:did:agent:approver-x".to_owned(),
            ],
        ),
        Err(DiscordBridgeError::UnauthorizedApprover(
            "kamn:did:agent:approver-x".to_owned()
        ))
    );
}

#[test]
fn integration_processes_discord_inbound_to_envelope() {
    let engine = DiscordBridgeEngine::new(config()).expect("engine should build");

    let envelope = engine
        .process_inbound_to_envelope(
            &DiscordInboundRequest {
                listener_did: "kamn:did:agent:listener-1".to_owned(),
                inbound: inbound("kamn:did:agent:listener-target-1"),
            },
            vec!["kamn:did:agent:listener-target-1#key-agreement-1".to_owned()],
            "2026-02-08T04:30:00Z",
            81,
        )
        .expect("envelope conversion should succeed");

    assert_eq!(
        envelope.envelope.from,
        "kamn:did:agent:bridge-discord-1".to_owned()
    );
    assert_eq!(
        envelope.envelope.to,
        vec!["kamn:did:agent:listener-target-1".to_owned()]
    );
}

#[test]
fn regression_rejects_replayed_discord_inbound_projection_event() {
    // Regression: #438
    let engine = DiscordBridgeEngine::new(config()).expect("engine should build");
    let request = DiscordInboundRequest {
        listener_did: "kamn:did:agent:listener-1".to_owned(),
        inbound: inbound("kamn:did:agent:listener-target-1"),
    };

    engine
        .process_inbound_to_envelope(
            &request,
            vec!["kamn:did:agent:listener-target-1#key-agreement-1".to_owned()],
            "2026-02-08T04:30:00Z",
            82,
        )
        .expect("first envelope projection should succeed");
    assert_eq!(
        engine.process_inbound_to_envelope(
            &request,
            vec!["kamn:did:agent:listener-target-1#key-agreement-1".to_owned()],
            "2026-02-08T04:30:00Z",
            83,
        ),
        Err(DiscordBridgeError::Bridge(
            "duplicate inbound message id: discord:discord-msg-1".to_owned()
        ))
    );
}

#[test]
fn regression_duplicate_approval_is_rejected() {
    let engine = DiscordBridgeEngine::new(config()).expect("engine should build");

    // Regression: #221
    assert_eq!(
        engine.process_outbound_with_approvals(
            &outbound(),
            vec![
                "kamn:did:agent:approver-1".to_owned(),
                "kamn:did:agent:approver-1".to_owned(),
            ],
        ),
        Err(DiscordBridgeError::DuplicateApproval(
            "kamn:did:agent:approver-1".to_owned()
        ))
    );
}
