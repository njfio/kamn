use kamn_core::{
    build_p2p_swarm_deterministic_config, build_runtime_wiring,
    build_runtime_wiring_with_transport_profile, canonical_libp2p_identify_protocol_id,
    canonical_libp2p_topic_id, Libp2pLivePeerLifecycleTransport, Libp2pRuntimeEventKind,
    NodeConfig, NodeRole, P2pSwarmHarnessMode, P2pTransportError, PeerDiscoveryRecord,
    PeerGossipFrame, PeerLifecycleEvent, PeerLifecycleState, PeerLifecycleTransport,
    PeerLifecycleTransportCoordinator, RuntimeLifecycleError, RuntimeTransportProfile, SyncMode,
};
use std::sync::atomic::{AtomicU16, Ordering};
use std::time::{Duration, Instant};

static NEXT_TEST_PORT: AtomicU16 = AtomicU16::new(21_000);

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
    let listen_address = unique_tcp_listen_address();
    let bootstrap_address = unique_tcp_listen_address();
    build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-processor",
        listen_address.as_str(),
        vec![bootstrap_address],
        vec!["messages".to_owned()],
        3,
    )
    .expect("swarm config should build")
}

fn live_swarm_config_for_peer(
    peer_id: &str,
    listen_address: &str,
    bootstrap_seed: &str,
) -> kamn_core::P2pSwarmDeterministicConfig {
    live_swarm_config_for_peer_with_bootstrap(
        peer_id,
        listen_address,
        vec![bootstrap_seed.to_owned()],
    )
}

fn live_swarm_config_for_peer_with_bootstrap(
    peer_id: &str,
    listen_address: &str,
    bootstrap_peers: Vec<String>,
) -> kamn_core::P2pSwarmDeterministicConfig {
    build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        peer_id,
        listen_address,
        bootstrap_peers,
        vec!["messages".to_owned()],
        3,
    )
    .expect("swarm config should build")
}

fn unique_tcp_listen_address() -> String {
    let port = NEXT_TEST_PORT.fetch_add(1, Ordering::Relaxed);
    format!("/ip4/127.0.0.1/tcp/{port}")
}

fn send_with_retry(
    transport: &Libp2pLivePeerLifecycleTransport,
    frame: &PeerGossipFrame,
    timeout: Duration,
) -> Result<(), P2pTransportError> {
    let started = Instant::now();
    loop {
        match transport.send(frame.clone()) {
            Ok(()) => return Ok(()),
            Err(P2pTransportError::LiveSocketSendFailed) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => return Err(error),
        }
    }
}

fn drain_until_count(
    transport: &Libp2pLivePeerLifecycleTransport,
    recipient_peer_id: &str,
    expected: usize,
    timeout: Duration,
) -> Vec<PeerGossipFrame> {
    let started = Instant::now();
    let mut frames = Vec::new();
    loop {
        let mut drained = transport
            .drain_inbox(recipient_peer_id)
            .expect("recipient inbox should drain");
        if !drained.is_empty() {
            frames.append(&mut drained);
        }
        if frames.len() >= expected {
            return frames;
        }
        assert!(
            started.elapsed() < timeout,
            "expected {expected} frames but only received {} within {:?}",
            frames.len(),
            timeout
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

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

#[test]
fn integration_live_transport_data_plane_supports_independent_adapter_exchange() {
    let processor_listen = unique_tcp_listen_address();
    let listener_listen = unique_tcp_listen_address();
    let bootstrap_peers = vec![processor_listen.clone(), listener_listen.clone()];
    let processor_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(
            "peer-processor-live",
            processor_listen.as_str(),
            bootstrap_peers.clone(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("processor live transport should initialize");
    std::thread::sleep(Duration::from_millis(250));
    let listener_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(
            "peer-listener-live",
            listener_listen.as_str(),
            bootstrap_peers,
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("listener live transport should initialize");

    let mut processor = PeerLifecycleTransportCoordinator::new(
        "peer-processor-live",
        NodeRole::Processor,
        processor_transport,
    )
    .expect("processor coordinator should initialize");
    let mut listener = PeerLifecycleTransportCoordinator::new(
        "peer-listener-live",
        NodeRole::Listener,
        listener_transport,
    )
    .expect("listener coordinator should initialize");

    assert_eq!(
        processor.connect_and_advertise(vec!["messages".to_owned()]),
        Ok(PeerLifecycleState::Active)
    );
    assert_eq!(
        listener.connect_and_advertise(vec!["messages".to_owned()]),
        Ok(PeerLifecycleState::Active)
    );

    let discovered = processor
        .discover("messages")
        .expect("live discovery should succeed");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].peer_id, "peer-listener-live");

    let started = Instant::now();
    let delivered = loop {
        match processor.broadcast("messages", "tx-live-001") {
            Ok(count) => break count,
            Err(P2pTransportError::LiveSocketSendFailed)
                if started.elapsed() < Duration::from_secs(5) =>
            {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("live broadcast should succeed: {error:?}"),
        }
    };
    assert_eq!(delivered, 1);

    let started = Instant::now();
    let listener_frames = loop {
        let frames = listener
            .drain_inbox()
            .expect("listener live inbox drain should succeed");
        if !frames.is_empty() {
            break frames;
        }
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "listener did not receive frame within timeout"
        );
        std::thread::sleep(Duration::from_millis(25));
    };
    assert_eq!(listener_frames.len(), 1);
    assert_eq!(listener_frames[0].sender_peer_id, "peer-processor-live");
    assert_eq!(listener_frames[0].topic, "messages");
    assert_eq!(listener_frames[0].payload, "tx-live-001");
}

#[test]
fn functional_live_transport_emits_normalized_runtime_events() {
    let processor_listen = unique_tcp_listen_address();
    let listener_listen = unique_tcp_listen_address();
    let bootstrap_peers = vec![processor_listen.clone(), listener_listen.clone()];
    let processor_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(
            "peer-processor-live-events",
            processor_listen.as_str(),
            bootstrap_peers.clone(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("processor live transport should initialize");
    std::thread::sleep(Duration::from_millis(250));
    let listener_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(
            "peer-listener-live-events",
            listener_listen.as_str(),
            bootstrap_peers,
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("listener live transport should initialize");

    processor_transport
        .advertise(
            PeerDiscoveryRecord::new(
                "peer-processor-live-events",
                NodeRole::Processor,
                vec!["messages".to_owned()],
            )
            .expect("processor record should build"),
        )
        .expect("processor advertise should pass");
    listener_transport
        .advertise(
            PeerDiscoveryRecord::new(
                "peer-listener-live-events",
                NodeRole::Listener,
                vec!["messages".to_owned()],
            )
            .expect("listener record should build"),
        )
        .expect("listener advertise should pass");

    let discovered = processor_transport
        .discover("peer-processor-live-events", "messages")
        .expect("discovery should pass");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].peer_id, "peer-listener-live-events");

    let frame = PeerGossipFrame::new(
        "messages",
        "peer-processor-live-events",
        "peer-listener-live-events",
        "tx-live-events-001",
    )
    .expect("gossip frame should build");
    send_with_retry(&processor_transport, &frame, Duration::from_secs(5))
        .expect("gossip send should pass");
    let _frames = drain_until_count(
        &listener_transport,
        "peer-listener-live-events",
        1,
        Duration::from_secs(5),
    );

    let started = Instant::now();
    let events = loop {
        let drained = processor_transport
            .drain_runtime_events()
            .expect("runtime events should drain");
        if drained.len() >= 5 {
            break drained;
        }
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "runtime events did not reach expected count within timeout"
        );
        std::thread::sleep(Duration::from_millis(25));
    };
    assert_eq!(events.len(), 5);
    assert_eq!(
        events
            .iter()
            .map(|event| event.kind())
            .collect::<Vec<Libp2pRuntimeEventKind>>(),
        vec![
            Libp2pRuntimeEventKind::PeerAdvertised,
            Libp2pRuntimeEventKind::PeerAdvertised,
            Libp2pRuntimeEventKind::PeerDiscovered,
            Libp2pRuntimeEventKind::GossipPublished,
            Libp2pRuntimeEventKind::GossipReceived,
        ]
    );
    assert!(events
        .iter()
        .all(|event| event.schema_marker() == "kamn.libp2p.runtime-event.v1"));
}

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
