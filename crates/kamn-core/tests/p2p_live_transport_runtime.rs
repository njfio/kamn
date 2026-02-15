use kamn_core::{
    build_p2p_swarm_deterministic_config, build_runtime_wiring,
    build_runtime_wiring_with_transport_profile, Libp2pLivePeerLifecycleTransport, NodeConfig,
    NodeRole, P2pSwarmHarnessMode, P2pTransportError, PeerLifecycleEvent, PeerLifecycleState,
    PeerLifecycleTransportCoordinator, RuntimeLifecycleError, RuntimeTransportProfile, SyncMode,
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

fn live_swarm_config() -> kamn_core::P2pSwarmDeterministicConfig {
    build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        "/ip4/127.0.0.1/tcp/9200",
        vec!["/ip4/127.0.0.1/tcp/9201/p2p/peer-listener".to_owned()],
        vec!["messages".to_owned()],
        3,
    )
    .expect("swarm config should build")
}

#[test]
fn unit_live_transport_adapter_reports_harness_startup_profile() {
    let transport =
        Libp2pLivePeerLifecycleTransport::new(live_swarm_config(), P2pSwarmHarnessMode::Run)
            .expect("live transport should initialize");

    assert_eq!(
        transport.transport_profile(),
        RuntimeTransportProfile::Libp2pLive
    );
    assert!(transport.harness_report().started());
    assert_eq!(transport.harness_report().executed_ticks(), 3);
    assert_eq!(transport.listen_address(), "/ip4/127.0.0.1/tcp/9200");
}

#[test]
fn functional_live_transport_signal_bridge_maps_deterministic_lifecycle_states() {
    let transport =
        Libp2pLivePeerLifecycleTransport::new(live_swarm_config(), P2pSwarmHarnessMode::DryRun)
            .expect("live transport should initialize");
    let mut coordinator =
        PeerLifecycleTransportCoordinator::new("peer-processor", NodeRole::Processor, transport)
            .expect("coordinator should initialize");

    assert_eq!(
        coordinator.apply_live_transport_signal(PeerLifecycleEvent::HandshakeSucceeded),
        Ok(PeerLifecycleState::Active)
    );
    assert_eq!(
        coordinator.apply_live_transport_signal(PeerLifecycleEvent::HeartbeatMissed),
        Ok(PeerLifecycleState::Degraded)
    );
    assert_eq!(
        coordinator.apply_live_transport_signal(PeerLifecycleEvent::HeartbeatRestored),
        Ok(PeerLifecycleState::Active)
    );
    assert_eq!(
        coordinator.apply_live_transport_signal(PeerLifecycleEvent::Disconnect),
        Ok(PeerLifecycleState::Disconnected)
    );
}

#[test]
fn integration_runtime_wiring_can_enable_live_transport_profile_markers() {
    let wiring = build_runtime_wiring_with_transport_profile(
        &config_for(NodeRole::Processor, true),
        RuntimeTransportProfile::Libp2pLive,
    );

    assert!(wiring
        .all_components()
        .contains(&"p2p-transport-profile:libp2p-live"));
    assert!(wiring
        .all_components()
        .contains(&"p2p-live-libp2p-provider"));
    assert!(!wiring
        .all_components()
        .contains(&"p2p-in-memory-transport-fallback"));

    let default_wiring = build_runtime_wiring(&config_for(NodeRole::Processor, true));
    assert!(default_wiring
        .all_components()
        .contains(&"p2p-transport-profile:in-memory-deterministic"));
    assert!(default_wiring
        .all_components()
        .contains(&"p2p-in-memory-transport-fallback"));
}

#[test]
fn regression_live_transport_signal_bridge_fails_closed_on_invalid_sequence() {
    // Regression: #3469
    let transport =
        Libp2pLivePeerLifecycleTransport::new(live_swarm_config(), P2pSwarmHarnessMode::DryRun)
            .expect("live transport should initialize");
    let mut coordinator =
        PeerLifecycleTransportCoordinator::new("peer-processor", NodeRole::Processor, transport)
            .expect("coordinator should initialize");

    let result = coordinator.apply_live_transport_signal(PeerLifecycleEvent::HeartbeatRestored);
    assert_eq!(
        result,
        Err(P2pTransportError::Lifecycle(
            RuntimeLifecycleError::InvalidTransition {
                from: PeerLifecycleState::Disconnected,
                event: PeerLifecycleEvent::HeartbeatRestored,
            }
        ))
    );
}
