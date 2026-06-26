pub(crate) use kamn_core::{
    build_p2p_swarm_deterministic_config, resolve_libp2p_live_runtime_backend,
    Libp2pLivePeerLifecycleTransport, Libp2pLiveRuntimeBackend, NodeConfig, NodeRole,
    P2pSwarmDeterministicConfig, P2pSwarmHarnessMode, P2pTransportError, PeerDiscoveryRecord,
    PeerGossipFrame, PeerLifecycleState, PeerLifecycleTransport, PeerLifecycleTransportCoordinator,
    RuntimeTransportProfile, SyncMode,
};
pub(crate) use std::sync::atomic::{AtomicU16, Ordering};
pub(crate) use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

static NEXT_TEST_PORT: AtomicU16 = AtomicU16::new(22_000);

pub(crate) fn config_for(role: NodeRole, gossip_enabled: bool) -> NodeConfig {
    NodeConfig {
        chain_id: "kamn-devnet".to_owned(),
        chain_version: "v0.1.0".to_owned(),
        role,
        storage_dir: "/tmp/kamn".to_owned(),
        enable_gossip: gossip_enabled,
        sync_mode: SyncMode::Fast,
    }
}

pub(crate) fn unique_bootstrap_seed() -> String {
    unique_listen_address()
}

pub(crate) fn unique_listen_address() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .subsec_nanos() as u16;
    let base_port = NEXT_TEST_PORT.fetch_add(1, Ordering::Relaxed);
    let port = base_port.wrapping_add(nanos % 1000);
    format!("/ip4/127.0.0.1/tcp/{port}")
}

pub(crate) fn live_swarm_config_for_peer(
    peer_id: &str,
    listen_address: &str,
    bootstrap_seed: &str,
) -> P2pSwarmDeterministicConfig {
    live_swarm_config_for_peer_with_bootstrap(
        peer_id,
        listen_address,
        vec![bootstrap_seed.to_owned()],
    )
}

pub(crate) fn live_swarm_config_for_peer_with_bootstrap(
    peer_id: &str,
    listen_address: &str,
    bootstrap_peers: Vec<String>,
) -> P2pSwarmDeterministicConfig {
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

pub(crate) fn new_transport(
    peer_id: &str,
    listen_address: &str,
    bootstrap_seed: &str,
) -> Libp2pLivePeerLifecycleTransport {
    Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(peer_id, listen_address, bootstrap_seed),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("native transport should initialize")
}

pub(crate) fn new_transport_with_bootstrap(
    peer_id: &str,
    listen_address: &str,
    bootstrap_peers: Vec<String>,
) -> Libp2pLivePeerLifecycleTransport {
    Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer_with_bootstrap(peer_id, listen_address, bootstrap_peers),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("native transport should initialize")
}

pub(crate) fn advertise_messages(
    transport: &Libp2pLivePeerLifecycleTransport,
    peer_id: &str,
    role: NodeRole,
) {
    transport
        .advertise(
            PeerDiscoveryRecord::new(peer_id, role, vec!["messages".to_owned()])
                .expect("peer record should build"),
        )
        .expect("peer advertise should pass");
}

pub(crate) fn message_frame(
    sender_peer_id: &str,
    recipient_peer_id: &str,
    payload: &str,
) -> PeerGossipFrame {
    PeerGossipFrame::new("messages", sender_peer_id, recipient_peer_id, payload)
        .expect("frame should build")
}

pub(crate) fn send_with_retry(
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

pub(crate) fn drain_until_count(
    transport: &Libp2pLivePeerLifecycleTransport,
    recipient_peer_id: &str,
    expected: usize,
    timeout: Duration,
) -> Vec<PeerGossipFrame> {
    let started = Instant::now();
    let mut frames = Vec::new();
    loop {
        append_drained_frames(transport, recipient_peer_id, &mut frames);
        if frames.len() >= expected {
            return frames;
        }
        assert_drain_deadline(started, timeout, expected, frames.len());
    }
}

pub(crate) fn settle_mesh() {
    std::thread::sleep(Duration::from_millis(250));
}

pub(crate) fn disconnected_transport_pair(
    sender_peer_id: &str,
    recipient_peer_id: &str,
) -> (
    Libp2pLivePeerLifecycleTransport,
    Libp2pLivePeerLifecycleTransport,
) {
    let bootstrap_seed = unique_bootstrap_seed();
    let sender = new_transport(
        sender_peer_id,
        unique_listen_address().as_str(),
        bootstrap_seed.as_str(),
    );
    let recipient = new_transport(
        recipient_peer_id,
        unique_listen_address().as_str(),
        bootstrap_seed.as_str(),
    );
    (sender, recipient)
}

fn append_drained_frames(
    transport: &Libp2pLivePeerLifecycleTransport,
    recipient_peer_id: &str,
    frames: &mut Vec<PeerGossipFrame>,
) {
    let mut drained = transport
        .drain_inbox(recipient_peer_id)
        .expect("recipient inbox should drain");
    if !drained.is_empty() {
        frames.append(&mut drained);
    }
}

fn assert_drain_deadline(
    started: Instant,
    timeout: Duration,
    expected: usize,
    current_count: usize,
) {
    assert!(
        started.elapsed() < timeout,
        "expected {expected} frames but only received {current_count} within {:?}",
        timeout
    );
    std::thread::sleep(Duration::from_millis(25));
}
