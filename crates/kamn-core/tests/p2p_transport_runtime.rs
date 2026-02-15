use kamn_core::{
    bootstrap, InMemoryPeerLifecycleTransport, NodeConfig, NodeRole, P2pTransportError,
    PeerGossipFrame, PeerLifecycleState, PeerLifecycleTransportCoordinator, SyncMode,
};

fn config_for(role: NodeRole, gossip_enabled: bool) -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role,
        storage_dir: "/tmp/kamn".to_owned(),
        enable_gossip: gossip_enabled,
        sync_mode: SyncMode::Fast,
    }
}

fn assert_single_frame(
    frames: &[PeerGossipFrame],
    expected_sender: &str,
    expected_topic: &str,
    expected_payload: &str,
) {
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].sender_peer_id, expected_sender);
    assert_eq!(frames[0].topic, expected_topic);
    assert_eq!(frames[0].payload, expected_payload);
}

#[test]
fn functional_p2p_transport_gossip_broadcast_reaches_discovered_roles() {
    let shared_transport = InMemoryPeerLifecycleTransport::default();
    let mut processor = PeerLifecycleTransportCoordinator::new(
        "peer-processor",
        NodeRole::Processor,
        shared_transport.clone(),
    )
    .expect("processor coordinator should initialize");
    let mut listener = PeerLifecycleTransportCoordinator::new(
        "peer-listener",
        NodeRole::Listener,
        shared_transport.clone(),
    )
    .expect("listener coordinator should initialize");
    let mut approver = PeerLifecycleTransportCoordinator::new(
        "peer-approver",
        NodeRole::Approver,
        shared_transport,
    )
    .expect("approver coordinator should initialize");

    assert_eq!(
        processor.connect_and_advertise(vec!["messages".to_owned()]),
        Ok(PeerLifecycleState::Active)
    );
    assert_eq!(
        listener.connect_and_advertise(vec!["messages".to_owned()]),
        Ok(PeerLifecycleState::Active)
    );
    assert_eq!(
        approver.connect_and_advertise(vec!["messages".to_owned()]),
        Ok(PeerLifecycleState::Active)
    );

    let discovered = processor
        .discover("messages")
        .expect("discovery should succeed when active");
    assert_eq!(discovered.len(), 2);

    let delivered = processor
        .broadcast("messages", "tx:001")
        .expect("broadcast should succeed");
    assert_eq!(delivered, 2);

    let listener_frames = listener
        .drain_inbox()
        .expect("listener drain should succeed");
    assert_single_frame(&listener_frames, "peer-processor", "messages", "tx:001");

    let approver_frames = approver
        .drain_inbox()
        .expect("approver drain should succeed");
    assert_single_frame(&approver_frames, "peer-processor", "messages", "tx:001");
}

#[test]
fn integration_bootstrap_wiring_includes_p2p_transport_components_when_gossip_enabled() {
    let gossip_plan =
        bootstrap(config_for(NodeRole::Processor, true)).expect("gossip bootstrap should pass");
    assert!(gossip_plan
        .wiring
        .all_components()
        .contains(&"p2p-discovery"));
    assert!(gossip_plan
        .wiring
        .all_components()
        .contains(&"p2p-gossip-transport"));
    assert!(gossip_plan
        .wiring
        .all_components()
        .contains(&"p2p-libp2p-swarm-stack"));
    assert!(gossip_plan
        .wiring
        .all_components()
        .contains(&"p2p-libp2p-harness-ready"));

    let disabled_plan = bootstrap(config_for(NodeRole::Processor, false))
        .expect("gossip-disabled bootstrap should pass");
    assert!(!disabled_plan
        .wiring
        .all_components()
        .contains(&"p2p-discovery"));
    assert!(disabled_plan
        .wiring
        .all_components()
        .contains(&"gossip-transport-disabled"));
    assert!(!disabled_plan
        .wiring
        .all_components()
        .contains(&"p2p-libp2p-swarm-stack"));
}

#[test]
fn regression_p2p_transport_rejects_broadcast_while_disconnected() {
    // Regression: #2922
    let shared_transport = InMemoryPeerLifecycleTransport::default();
    let mut listener = PeerLifecycleTransportCoordinator::new(
        "peer-listener",
        NodeRole::Listener,
        shared_transport.clone(),
    )
    .expect("listener coordinator should initialize");
    let processor = PeerLifecycleTransportCoordinator::new(
        "peer-processor",
        NodeRole::Processor,
        shared_transport,
    )
    .expect("processor coordinator should initialize");
    listener
        .connect_and_advertise(vec!["messages".to_owned()])
        .expect("listener must be discoverable");

    let result = processor.broadcast("messages", "tx:001");
    assert_eq!(
        result,
        Err(P2pTransportError::InactivePeerLifecycleState(
            PeerLifecycleState::Disconnected
        ))
    );
}
