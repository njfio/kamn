use kamn_core::{
    BridgeInboundEnvelope, BridgeOutboundRequest, BridgePlatform, CrossChainBridgeConfig,
    CrossChainBridgeEngine, CrossChainBridgeError, CrossChainInboundRequest, CrossChainNetwork,
};
use std::collections::{BTreeMap, BTreeSet};

fn config() -> CrossChainBridgeConfig {
    let mut ethereum_routes = BTreeMap::new();
    ethereum_routes.insert(
        "ethereum:sepolia:contract:escrow-v1".to_owned(),
        "kamn:did:agent:listener-target-eth".to_owned(),
    );

    let mut solana_routes = BTreeMap::new();
    solana_routes.insert(
        "solana:devnet:program:task-v1".to_owned(),
        "kamn:did:agent:listener-target-sol".to_owned(),
    );

    CrossChainBridgeConfig {
        bridge_agent_did: "kamn:did:agent:bridge-crosschain-1".to_owned(),
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
        ethereum_routes,
        solana_routes,
    }
}

fn eth_inbound(target: &str) -> BridgeInboundEnvelope {
    BridgeInboundEnvelope {
        external_message_id: "0xabc123:7".to_owned(),
        external_sender_id: "0xContractEscrow".to_owned(),
        external_channel_id: "ethereum:sepolia:contract:escrow-v1".to_owned(),
        target_agent_did: target.to_owned(),
        body: "event:PaymentOffer".to_owned(),
        received_at: "2026-02-08T09:00:00Z".to_owned(),
    }
}

fn solana_inbound(target: &str) -> BridgeInboundEnvelope {
    BridgeInboundEnvelope {
        external_message_id: "slot-881991:ix-2".to_owned(),
        external_sender_id: "So11111111111111111111111111111111111111112".to_owned(),
        external_channel_id: "solana:devnet:program:task-v1".to_owned(),
        target_agent_did: target.to_owned(),
        body: "event:TaskStateChanged".to_owned(),
        received_at: "2026-02-08T09:05:00Z".to_owned(),
    }
}

fn solana_outbound() -> BridgeOutboundRequest {
    BridgeOutboundRequest {
        request_id: "solana-outbound-1".to_owned(),
        from_agent_did: "kamn:did:agent:processor-1".to_owned(),
        destination_channel_id: "solana:devnet:program:task-v1".to_owned(),
        body: "instruction:acknowledge".to_owned(),
        created_at: "2026-02-08T09:10:00Z".to_owned(),
    }
}

#[test]
fn ethereum_listener_can_process_inbound() {
    let engine = CrossChainBridgeEngine::new(config()).expect("engine should build");

    let normalized = engine
        .process_inbound(&CrossChainInboundRequest {
            listener_did: "kamn:did:agent:listener-1".to_owned(),
            chain: CrossChainNetwork::Ethereum,
            inbound: eth_inbound("kamn:did:agent:listener-target-eth"),
        })
        .expect("ethereum inbound should process");

    assert_eq!(normalized.sender_handle, "0xContractEscrow");
    assert_eq!(normalized.bridge_message_id, "ethereum:0xabc123:7");
}

#[test]
fn solana_quorum_dispatches_outbound() {
    let engine = CrossChainBridgeEngine::new(config()).expect("engine should build");

    let dispatch = engine
        .process_outbound_with_approvals(
            CrossChainNetwork::Solana,
            &solana_outbound(),
            vec![
                "kamn:did:agent:approver-1".to_owned(),
                "kamn:did:agent:approver-2".to_owned(),
            ],
        )
        .expect("solana outbound should dispatch with quorum");

    assert_eq!(dispatch.envelope.request_id, "solana-outbound-1");
    assert_eq!(
        dispatch.envelope.platform,
        BridgePlatform::Custom("solana".to_owned())
    );
    assert_eq!(dispatch.approval.required_approvals, 2);
    assert_eq!(dispatch.approval.approved_by.len(), 2);
}

#[test]
fn outbound_rejects_unauthorized_approver() {
    let engine = CrossChainBridgeEngine::new(config()).expect("engine should build");

    assert_eq!(
        engine.process_outbound_with_approvals(
            CrossChainNetwork::Ethereum,
            &BridgeOutboundRequest {
                destination_channel_id: "ethereum:sepolia:contract:escrow-v1".to_owned(),
                ..solana_outbound()
            },
            vec![
                "kamn:did:agent:approver-1".to_owned(),
                "kamn:did:agent:approver-x".to_owned(),
            ],
        ),
        Err(CrossChainBridgeError::UnauthorizedApprover(
            "kamn:did:agent:approver-x".to_owned()
        ))
    );
}

#[test]
fn integration_projects_solana_inbound_to_envelope() {
    let engine = CrossChainBridgeEngine::new(config()).expect("engine should build");

    let envelope = engine
        .process_inbound_to_envelope(
            &CrossChainInboundRequest {
                listener_did: "kamn:did:agent:listener-1".to_owned(),
                chain: CrossChainNetwork::Solana,
                inbound: solana_inbound("kamn:did:agent:listener-target-sol"),
            },
            vec!["kamn:did:agent:listener-target-sol#key-agreement-1".to_owned()],
            "2026-02-08T09:30:00Z",
            13,
        )
        .expect("solana inbound projection should succeed");

    assert_eq!(
        envelope.envelope.from,
        "kamn:did:agent:bridge-crosschain-1".to_owned()
    );
    assert_eq!(
        envelope.envelope.to,
        vec!["kamn:did:agent:listener-target-sol".to_owned()]
    );
}

#[test]
fn regression_rejects_replayed_solana_inbound_projection_event() {
    // Regression: #443
    let engine = CrossChainBridgeEngine::new(config()).expect("engine should build");
    let request = CrossChainInboundRequest {
        listener_did: "kamn:did:agent:listener-1".to_owned(),
        chain: CrossChainNetwork::Solana,
        inbound: solana_inbound("kamn:did:agent:listener-target-sol"),
    };

    engine
        .process_inbound_to_envelope(
            &request,
            vec!["kamn:did:agent:listener-target-sol#key-agreement-1".to_owned()],
            "2026-02-08T09:30:00Z",
            14,
        )
        .expect("first inbound projection should succeed");
    assert_eq!(
        engine.process_inbound_to_envelope(
            &request,
            vec!["kamn:did:agent:listener-target-sol#key-agreement-1".to_owned()],
            "2026-02-08T09:30:00Z",
            15,
        ),
        Err(CrossChainBridgeError::Bridge(
            "duplicate inbound message id: solana:slot-881991:ix-2".to_owned()
        ))
    );
}

#[test]
fn regression_unknown_ethereum_route_is_rejected() {
    let engine = CrossChainBridgeEngine::new(config()).expect("engine should build");

    // Regression: #233
    assert_eq!(
        engine.process_inbound(&CrossChainInboundRequest {
            listener_did: "kamn:did:agent:listener-1".to_owned(),
            chain: CrossChainNetwork::Ethereum,
            inbound: BridgeInboundEnvelope {
                external_channel_id: "ethereum:sepolia:contract:unknown".to_owned(),
                ..eth_inbound("kamn:did:agent:listener-target-eth")
            },
        }),
        Err(CrossChainBridgeError::UnknownRouteChannel {
            chain: CrossChainNetwork::Ethereum,
            channel_id: "ethereum:sepolia:contract:unknown".to_owned(),
        })
    );
}
