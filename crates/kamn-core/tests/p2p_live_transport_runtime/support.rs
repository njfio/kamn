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

pub(super) fn pause_for_live_handshake() {
    std::thread::sleep(Duration::from_millis(250));
}

pub(super) fn build_live_transport(
    peer_id: &str,
    listen_address: &str,
    bootstrap_peers: Vec<String>,
) -> Libp2pLivePeerLifecycleTransport {
    Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(peer_id, listen_address, bootstrap_peers),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("live transport should initialize")
}

pub(super) fn build_seeded_live_transport(
    peer_id: &str,
    bootstrap_seed: &str,
) -> Libp2pLivePeerLifecycleTransport {
    Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            peer_id,
            unique_tcp_listen_address().as_str(),
            bootstrap_seed,
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("live transport should initialize")
}

pub(super) fn advertise_messages_peer(
    transport: &Libp2pLivePeerLifecycleTransport,
    peer_id: &str,
    role: NodeRole,
) {
    transport
        .advertise(
            PeerDiscoveryRecord::new(peer_id, role, vec!["messages".to_owned()])
                .expect("discovery record should build"),
        )
        .expect("peer advertise should pass");
}

#[cfg(not(feature = "libp2p-live-transport"))]
pub(super) fn runtime_backpressure_reject(error: P2pTransportError) -> Option<&'static str> {
    match error {
        P2pTransportError::RuntimeBackpressureRejected { reason_code, .. } => Some(reason_code),
        _ => None,
    }
}

#[path = "support/transport_io_support.rs"]
mod transport_io_support;

pub(crate) use transport_io_support::{
    drain_runtime_events_until, drain_until_count, send_with_retry,
};
#[cfg(not(feature = "libp2p-live-transport"))]
pub(crate) use transport_io_support::{send_frames_expect_success, send_frames_until_error};
