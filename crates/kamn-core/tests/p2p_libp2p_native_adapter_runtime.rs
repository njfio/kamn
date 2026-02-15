#![cfg(feature = "libp2p-live-transport")]

use kamn_core::{
    build_p2p_swarm_deterministic_config, resolve_libp2p_live_runtime_backend,
    Libp2pLivePeerLifecycleTransport, Libp2pLiveRuntimeBackend, NodeConfig, NodeRole,
    P2pSwarmHarnessMode, PeerDiscoveryRecord, PeerGossipFrame, PeerLifecycleState,
    PeerLifecycleTransport, PeerLifecycleTransportCoordinator, RuntimeTransportProfile, SyncMode,
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

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
    build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        peer_id,
        listen_address,
        vec![bootstrap_seed.to_owned()],
        vec!["messages".to_owned()],
        3,
    )
    .expect("swarm config should build")
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
fn integration_libp2p_native_adapter_supports_discovery_and_gossip_over_sockets() {
    let bootstrap_seed = unique_bootstrap_seed();
    let processor_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            "peer-native-processor",
            "/ip4/127.0.0.1/tcp/9520",
            bootstrap_seed.as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("processor transport should initialize");
    let listener_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            "peer-native-listener",
            "/ip4/127.0.0.1/tcp/9521",
            bootstrap_seed.as_str(),
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

    let delivered = processor
        .broadcast("messages", "tx-native-001")
        .expect("native broadcast should succeed");
    assert_eq!(delivered, 1);

    let listener_frames = listener
        .drain_inbox()
        .expect("listener inbox drain should succeed");
    assert_eq!(listener_frames.len(), 1);
    assert_eq!(listener_frames[0].sender_peer_id, "peer-native-processor");
    assert_eq!(listener_frames[0].topic, "messages");
    assert_eq!(listener_frames[0].payload, "tx-native-001");
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
    let started = Instant::now();
    let bootstrap_seed = unique_bootstrap_seed();
    let sender_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            "peer-native-perf-sender",
            "/ip4/127.0.0.1/tcp/9530",
            bootstrap_seed.as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("sender transport should initialize");
    let recipient_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            "peer-native-perf-recipient",
            "/ip4/127.0.0.1/tcp/9531",
            bootstrap_seed.as_str(),
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

    for nonce in 0..64 {
        sender_transport
            .send(
                PeerGossipFrame::new(
                    "messages",
                    "peer-native-perf-sender",
                    "peer-native-perf-recipient",
                    &format!("tx-native-performance-{nonce}"),
                )
                .expect("frame should build"),
            )
            .expect("frame should send");
    }

    let frames = recipient_transport
        .drain_inbox("peer-native-perf-recipient")
        .expect("recipient inbox should drain");
    assert_eq!(frames.len(), 64);
    assert!(
        started.elapsed() <= std::time::Duration::from_secs(2),
        "libp2p native adapter exceeded local-heavy runtime budget"
    );
}
