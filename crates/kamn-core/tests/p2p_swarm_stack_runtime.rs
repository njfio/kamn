use kamn_core::{
    build_p2p_swarm_deterministic_config, compose_libp2p_swarm_behavior_stack,
    InMemoryPeerLifecycleTransport, NodeConfig, NodeRole, P2pSwarmHarnessMode, P2pSwarmHarnessTask,
    P2pTransportError, PeerLifecycleState, PeerLifecycleTransportCoordinator, SyncMode,
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

#[test]
fn unit_swarm_config_rejects_invalid_listen_address() {
    let result = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        "127.0.0.1:9000",
        vec![],
        vec!["messages".to_owned()],
        2,
    );

    assert_eq!(result, Err(P2pTransportError::InvalidSwarmListenAddress));
}

#[test]
fn functional_swarm_behavior_stack_contains_required_protocols() {
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        "/ip4/127.0.0.1/tcp/9000",
        vec!["/ip4/127.0.0.1/tcp/9001/p2p/peer-listener".to_owned()],
        vec!["messages".to_owned(), "blocks".to_owned()],
        3,
    )
    .expect("deterministic config should build");

    let stack = compose_libp2p_swarm_behavior_stack(&config);
    assert_eq!(
        stack.behavior_components(),
        vec!["tcp", "noise", "yamux", "identify", "kad", "gossipsub",]
    );
    assert_eq!(
        stack.gossip_topics(),
        vec!["blocks".to_owned(), "messages".to_owned()]
    );
}

#[test]
fn integration_runtime_can_start_swarm_harness_task() {
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        "/ip4/127.0.0.1/tcp/9000",
        vec![
            "/ip4/127.0.0.1/tcp/9001".to_owned(),
            "/ip4/127.0.0.1/tcp/9002".to_owned(),
        ],
        vec![
            "messages".to_owned(),
            "blocks".to_owned(),
            "reputation-updates".to_owned(),
        ],
        4,
    )
    .expect("deterministic config should build");

    let task = P2pSwarmHarnessTask::new(config);
    let report = task
        .start(P2pSwarmHarnessMode::Run)
        .expect("runtime harness start should pass");
    assert!(report.started());
    assert_eq!(report.executed_ticks(), 4);
    assert_eq!(report.bootstrap_peer_count(), 2);
}

#[cfg(feature = "libp2p-live-transport")]
#[test]
fn integration_swarm_harness_report_includes_native_runtime_marker_when_feature_enabled() {
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-native-marker",
        "/ip4/127.0.0.1/tcp/9050",
        vec!["/ip4/127.0.0.1/tcp/9051".to_owned()],
        vec!["messages".to_owned()],
        2,
    )
    .expect("deterministic config should build");

    let task = P2pSwarmHarnessTask::new(config);
    let report = task
        .start(P2pSwarmHarnessMode::Run)
        .expect("runtime harness start should pass");
    assert!(report.started());
    assert!(
        report
            .behavior_components()
            .contains(&"libp2p-runtime-swarm"),
        "feature-enabled run mode must report native libp2p runtime stack marker"
    );
}

#[test]
fn regression_in_memory_transport_fallback_remains_deterministic() {
    // Regression: #3356
    let transport = InMemoryPeerLifecycleTransport::default();
    let mut processor = PeerLifecycleTransportCoordinator::new(
        "peer-processor",
        NodeRole::Processor,
        transport.clone(),
    )
    .expect("processor coordinator should initialize");
    let mut listener =
        PeerLifecycleTransportCoordinator::new("peer-listener", NodeRole::Listener, transport)
            .expect("listener coordinator should initialize");

    assert_eq!(
        processor.connect_and_advertise(vec!["messages".to_owned()]),
        Ok(PeerLifecycleState::Active)
    );
    assert_eq!(
        listener.connect_and_advertise(vec!["messages".to_owned()]),
        Ok(PeerLifecycleState::Active)
    );

    assert_eq!(processor.broadcast("messages", "tx:001"), Ok(1));
    let listener_frames = listener.drain_inbox().expect("listener drain should pass");
    assert_eq!(listener_frames.len(), 1);
    assert_eq!(listener_frames[0].payload, "tx:001");
}
