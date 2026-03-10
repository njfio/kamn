use super::support::*;

#[test]
fn unit_live_transport_adapter_reports_harness_startup_profile() {
    let config = live_swarm_config();
    let expected_listen_address = config.listen_address().to_owned();
    let transport = Libp2pLivePeerLifecycleTransport::new(config, P2pSwarmHarnessMode::Run)
        .expect("live transport should initialize");

    assert_eq!(
        transport.transport_profile(),
        RuntimeTransportProfile::Libp2pLive
    );
    assert!(transport.harness_report().started());
    assert_eq!(transport.harness_report().executed_ticks(), 3);
    assert_eq!(transport.listen_address(), expected_listen_address);
}

#[test]
fn unit_libp2p_runtime_protocol_and_topic_ids_are_deterministic() {
    assert_eq!(
        canonical_libp2p_identify_protocol_id(),
        "/kamn/libp2p-live/1.0.0"
    );
    assert_eq!(
        canonical_libp2p_topic_id("messages").expect("topic id should normalize"),
        "kamn/v1/messages"
    );
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
    assert_eq!(
        coordinator.apply_live_transport_signal(PeerLifecycleEvent::Rejoin),
        Ok(PeerLifecycleState::Connecting)
    );
    assert_eq!(
        coordinator.apply_live_transport_signal(PeerLifecycleEvent::HandshakeSucceeded),
        Ok(PeerLifecycleState::Active)
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
