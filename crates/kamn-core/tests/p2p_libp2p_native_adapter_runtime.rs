#![cfg(feature = "libp2p-live-transport")]

use kamn_core::{
    build_p2p_swarm_deterministic_config, resolve_libp2p_live_runtime_backend,
    Libp2pLivePeerLifecycleTransport, Libp2pLiveRuntimeBackend, NodeConfig, NodeRole,
    P2pSwarmHarnessMode, P2pTransportError, PeerDiscoveryRecord, PeerGossipFrame,
    PeerLifecycleState, PeerLifecycleTransport, PeerLifecycleTransportCoordinator,
    RuntimeTransportProfile, SyncMode,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

fn unique_bootstrap_seed() -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    let port = 20_000 + (nonce % 20_000) as u16;
    format!("/ip4/127.0.0.1/tcp/{port}")
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
fn unit_libp2p_native_adapter_rejects_invalid_listen_multiaddr() {
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-native-invalid",
        "/ip4/127.0.0.1/tcp/invalid-port",
        vec!["/ip4/127.0.0.1/tcp/9201".to_owned()],
        vec!["messages".to_owned()],
        3,
    )
    .expect("base config should build");

    let error = Libp2pLivePeerLifecycleTransport::new(config, P2pSwarmHarnessMode::DryRun)
        .expect_err("invalid libp2p native listen multiaddr must fail");
    assert_eq!(
        error.reason_code(),
        "p2p_transport_libp2p_runtime_config_invalid"
    );
}

#[test]
fn functional_libp2p_native_backend_selection_marker_is_stable() {
    assert_eq!(
        resolve_libp2p_live_runtime_backend(),
        Libp2pLiveRuntimeBackend::NativeSocket
    );
    assert_eq!(
        resolve_libp2p_live_runtime_backend().marker(),
        "native-libp2p-swarm"
    );
}

#[test]
fn functional_libp2p_native_adapter_loop_marker_is_stable() {
    let transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            "peer-native-loop-marker",
            "/ip4/127.0.0.1/tcp/9560",
            unique_bootstrap_seed().as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("native transport should initialize");
    assert_eq!(
        transport.native_runtime_loop_marker(),
        "libp2p-runtime-adapter-loop"
    );
}

#[test]
fn integration_libp2p_native_adapter_supports_discovery_and_gossip_over_sockets() {
    let processor_listen = "/ip4/127.0.0.1/tcp/9520";
    let listener_listen = "/ip4/127.0.0.1/tcp/9521";
    let bootstrap_peers = vec![processor_listen.to_owned(), listener_listen.to_owned()];
    let processor_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(
            "peer-native-processor",
            processor_listen,
            bootstrap_peers.clone(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("processor transport should initialize");
    std::thread::sleep(Duration::from_millis(250));
    let listener_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(
            "peer-native-listener",
            listener_listen,
            bootstrap_peers,
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("listener transport should initialize");

    assert_eq!(
        processor_transport.transport_profile(),
        RuntimeTransportProfile::Libp2pLive
    );
    assert_eq!(
        resolve_libp2p_live_runtime_backend().marker(),
        "native-libp2p-swarm"
    );

    let mut processor = PeerLifecycleTransportCoordinator::new(
        "peer-native-processor",
        NodeRole::Processor,
        processor_transport,
    )
    .expect("processor coordinator should initialize");
    let mut listener = PeerLifecycleTransportCoordinator::new(
        "peer-native-listener",
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
        .expect("native discovery should succeed");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].peer_id, "peer-native-listener");

    let started = Instant::now();
    let delivered = loop {
        match processor.broadcast("messages", "tx-native-001") {
            Ok(count) => break count,
            Err(P2pTransportError::LiveSocketSendFailed)
                if started.elapsed() < Duration::from_secs(5) =>
            {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("native broadcast should succeed: {error:?}"),
        }
    };
    assert_eq!(delivered, 1);

    let started = Instant::now();
    let listener_frames = loop {
        let frames = listener
            .drain_inbox()
            .expect("listener inbox drain should succeed");
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
    assert_eq!(listener_frames[0].sender_peer_id, "peer-native-processor");
    assert_eq!(listener_frames[0].topic, "messages");
    assert_eq!(listener_frames[0].payload, "tx-native-001");
}

#[test]
fn integration_libp2p_native_adapter_disconnected_publish_fails_closed() {
    let bootstrap_seed = unique_bootstrap_seed();
    let sender_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            "peer-native-disconnected-sender",
            "/ip4/127.0.0.1/tcp/9570",
            bootstrap_seed.as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("sender transport should initialize");
    let recipient_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            "peer-native-disconnected-recipient",
            "/ip4/127.0.0.1/tcp/9571",
            bootstrap_seed.as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("recipient transport should initialize");

    sender_transport
        .advertise(
            PeerDiscoveryRecord::new(
                "peer-native-disconnected-sender",
                NodeRole::Processor,
                vec!["messages".to_owned()],
            )
            .expect("sender record should build"),
        )
        .expect("sender advertise should pass");
    recipient_transport
        .advertise(
            PeerDiscoveryRecord::new(
                "peer-native-disconnected-recipient",
                NodeRole::Listener,
                vec!["messages".to_owned()],
            )
            .expect("recipient record should build"),
        )
        .expect("recipient advertise should pass");

    let result = sender_transport.send(
        PeerGossipFrame::new(
            "messages",
            "peer-native-disconnected-sender",
            "peer-native-disconnected-recipient",
            "tx-native-disconnected-001",
        )
        .expect("frame should build"),
    );
    assert_eq!(
        result,
        Err(kamn_core::P2pTransportError::LiveSocketSendFailed)
    );
}

#[test]
fn unit_libp2p_native_adapter_rejects_invalid_bootstrap_multiaddr() {
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-native-invalid-bootstrap",
        "/ip4/127.0.0.1/tcp/9540",
        vec!["/ip4/127.0.0.1/tcp/9541/invalid-proto".to_owned()],
        vec!["messages".to_owned()],
        3,
    )
    .expect("base config should build");

    let error = Libp2pLivePeerLifecycleTransport::new(config, P2pSwarmHarnessMode::DryRun)
        .expect_err("invalid bootstrap multiaddr must fail");
    assert_eq!(
        error.reason_code(),
        "p2p_transport_libp2p_runtime_config_invalid"
    );
}

#[test]
fn regression_libp2p_native_runtime_config_error_reason_code_stays_stable() {
    // Regression: #3633
    let config = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-native-invalid-regression",
        "/ip4/127.0.0.1/tcp/invalid-port",
        vec!["/ip4/127.0.0.1/tcp/9201".to_owned()],
        vec!["messages".to_owned()],
        3,
    )
    .expect("base config should build");

    let error = Libp2pLivePeerLifecycleTransport::new(config, P2pSwarmHarnessMode::DryRun)
        .expect_err("invalid config should fail");
    assert_eq!(
        error.reason_code(),
        "p2p_transport_libp2p_runtime_config_invalid"
    );
}

#[test]
fn performance_libp2p_native_adapter_stays_within_local_heavy_budget() {
    let sender_listen = "/ip4/127.0.0.1/tcp/9530";
    let recipient_listen = "/ip4/127.0.0.1/tcp/9531";
    let bootstrap_peers = vec![sender_listen.to_owned(), recipient_listen.to_owned()];
    let sender_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(
            "peer-native-perf-sender",
            sender_listen,
            bootstrap_peers.clone(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("sender transport should initialize");
    std::thread::sleep(Duration::from_millis(250));
    let recipient_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(
            "peer-native-perf-recipient",
            recipient_listen,
            bootstrap_peers,
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("recipient transport should initialize");

    sender_transport
        .advertise(
            PeerDiscoveryRecord::new(
                "peer-native-perf-sender",
                NodeRole::Processor,
                vec!["messages".to_owned()],
            )
            .expect("sender record should build"),
        )
        .expect("sender advertise should succeed");
    recipient_transport
        .advertise(
            PeerDiscoveryRecord::new(
                "peer-native-perf-recipient",
                NodeRole::Listener,
                vec!["messages".to_owned()],
            )
            .expect("recipient record should build"),
        )
        .expect("recipient advertise should succeed");

    let warmup_frame = PeerGossipFrame::new(
        "messages",
        "peer-native-perf-sender",
        "peer-native-perf-recipient",
        "tx-native-performance-warmup",
    )
    .expect("warmup frame should build");
    send_with_retry(&sender_transport, &warmup_frame, Duration::from_secs(5))
        .expect("warmup frame should send");
    let _warmup_frames = drain_until_count(
        &recipient_transport,
        "peer-native-perf-recipient",
        1,
        Duration::from_secs(5),
    );

    let started = Instant::now();
    for nonce in 0..64 {
        let frame = PeerGossipFrame::new(
            "messages",
            "peer-native-perf-sender",
            "peer-native-perf-recipient",
            &format!("tx-native-performance-{nonce}"),
        )
        .expect("frame should build");
        send_with_retry(&sender_transport, &frame, Duration::from_secs(2))
            .expect("frame should send");
    }

    let frames = drain_until_count(
        &recipient_transport,
        "peer-native-perf-recipient",
        64,
        Duration::from_secs(2),
    );
    assert_eq!(frames.len(), 64);
    assert!(
        started.elapsed() <= std::time::Duration::from_secs(2),
        "libp2p native adapter exceeded local-heavy runtime budget"
    );
}
