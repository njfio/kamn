pub(super) use kamn_core::{
    build_p2p_swarm_deterministic_config, build_runtime_wiring,
    build_runtime_wiring_with_transport_profile, canonical_libp2p_identify_protocol_id,
    canonical_libp2p_topic_id, Libp2pLivePeerLifecycleTransport, Libp2pRuntimeEventKind,
    NodeConfig, NodeRole, P2pSwarmHarnessMode, P2pTransportError, PeerDiscoveryRecord,
    PeerGossipFrame, PeerLifecycleEvent, PeerLifecycleState, PeerLifecycleTransport,
    PeerLifecycleTransportCoordinator, RuntimeLifecycleError, RuntimeTransportProfile, SyncMode,
};
pub(super) use std::sync::atomic::{AtomicU16, Ordering};
pub(super) use std::time::{Duration, Instant};

static NEXT_TEST_PORT: AtomicU16 = AtomicU16::new(21_000);

pub(super) fn config_for(role: NodeRole, gossip_enabled: bool) -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role,
        storage_dir: "/tmp/kamn".to_owned(),
        enable_gossip: gossip_enabled,
        sync_mode: SyncMode::Fast,
    }
}

pub(super) fn live_swarm_config() -> kamn_core::P2pSwarmDeterministicConfig {
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

pub(super) fn live_swarm_config_for_peer(
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

pub(super) fn live_swarm_config_for_peer_with_bootstrap(
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

pub(super) fn unique_tcp_listen_address() -> String {
    let port = NEXT_TEST_PORT.fetch_add(1, Ordering::Relaxed);
    format!("/ip4/127.0.0.1/tcp/{port}")
}

pub(super) fn send_with_retry(
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

pub(super) fn drain_until_count(
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
