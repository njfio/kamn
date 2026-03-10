use super::support::*;

#[test]
fn integration_live_transport_invalid_event_retries_are_idempotent() {
    let transport =
        Libp2pLivePeerLifecycleTransport::new(live_swarm_config(), P2pSwarmHarnessMode::DryRun)
            .expect("live transport should initialize");
    let mut coordinator =
        PeerLifecycleTransportCoordinator::new("peer-processor", NodeRole::Processor, transport)
            .expect("coordinator should initialize");

    for _ in 0..3 {
        let error = coordinator
            .apply_live_transport_signal(PeerLifecycleEvent::HeartbeatRestored)
            .expect_err("heartbeat restore from disconnected must fail");
        assert_eq!(error.reason_code(), "runtime_peer_transition_invalid");
        assert_eq!(
            coordinator.lifecycle_state(),
            PeerLifecycleState::Disconnected
        );
    }
}

#[test]
fn regression_live_transport_data_plane_unknown_recipient_fails_closed() {
    // Regression: #3574
    let bootstrap_seed = unique_tcp_listen_address();
    let transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            "peer-processor-live-fail-closed",
            unique_tcp_listen_address().as_str(),
            bootstrap_seed.as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("live transport should initialize");

    transport
        .advertise(
            PeerDiscoveryRecord::new(
                "peer-processor-live-fail-closed",
                NodeRole::Processor,
                vec!["messages".to_owned()],
            )
            .expect("discovery record should build"),
        )
        .expect("local peer should advertise");

    let frame = PeerGossipFrame::new(
        "messages",
        "peer-processor-live-fail-closed",
        "peer-missing-live",
        "tx-live-fail-closed",
    )
    .expect("frame should build");
    let result = transport.send(frame);
    assert_eq!(
        result,
        Err(P2pTransportError::UnknownRecipientPeer(
            "peer-missing-live".to_owned()
        ))
    );
}

#[test]
fn regression_live_transport_invalid_transition_reason_code_stable() {
    // Regression: #3575
    let transport =
        Libp2pLivePeerLifecycleTransport::new(live_swarm_config(), P2pSwarmHarnessMode::DryRun)
            .expect("live transport should initialize");
    let mut coordinator =
        PeerLifecycleTransportCoordinator::new("peer-processor", NodeRole::Processor, transport)
            .expect("coordinator should initialize");

    let error = coordinator
        .apply_live_transport_signal(PeerLifecycleEvent::HeartbeatRestored)
        .expect_err("heartbeat restore from disconnected must fail");
    assert_eq!(error.reason_code(), "runtime_peer_transition_invalid");
}

#[test]
fn regression_libp2p_topic_normalization_invalid_topic_reason_code_stable() {
    // Regression: #3668
    let error =
        canonical_libp2p_topic_id("bad|topic").expect_err("wire-delimited topics must fail closed");
    assert_eq!(error.reason_code(), "p2p_transport_invalid_topic");
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
