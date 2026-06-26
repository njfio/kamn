use super::support::*;

#[test]
fn integration_live_transport_data_plane_supports_independent_adapter_exchange() {
    let (mut processor, mut listener) = build_live_exchange_pair();
    assert_pair_connects_and_discovers(&mut processor, &mut listener);
    assert_eq!(broadcast_with_retry(&mut processor, "tx-live-001"), 1);
    let listener_frames = drain_listener_frames(&mut listener);
    assert_delivered_payload(&listener_frames[0], "peer-processor-live", "tx-live-001");
}

fn build_live_exchange_pair() -> (
    PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
    PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
) {
    let processor_listen = unique_tcp_listen_address();
    let listener_listen = unique_tcp_listen_address();
    let bootstrap_peers = vec![processor_listen.clone(), listener_listen.clone()];
    let processor_transport = build_live_transport(
        "peer-processor-live",
        processor_listen.as_str(),
        bootstrap_peers.clone(),
    );
    pause_for_live_handshake();
    let listener_transport = build_live_transport(
        "peer-listener-live",
        listener_listen.as_str(),
        bootstrap_peers,
    );
    (
        build_coordinator(
            "peer-processor-live",
            NodeRole::Processor,
            processor_transport,
        ),
        build_coordinator("peer-listener-live", NodeRole::Listener, listener_transport),
    )
}

fn build_coordinator(
    peer_id: &str,
    role: NodeRole,
    transport: Libp2pLivePeerLifecycleTransport,
) -> PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport> {
    PeerLifecycleTransportCoordinator::new(peer_id, role, transport)
        .expect("coordinator should initialize")
}

fn assert_pair_connects_and_discovers(
    processor: &mut PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
    listener: &mut PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
) {
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
}

fn broadcast_with_retry(
    processor: &mut PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
    payload: &str,
) -> usize {
    let started = Instant::now();
    loop {
        match processor.broadcast("messages", payload) {
            Ok(count) => return count,
            Err(P2pTransportError::LiveSocketSendFailed)
                if started.elapsed() < Duration::from_secs(5) =>
            {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("live broadcast should succeed: {error:?}"),
        }
    }
}

fn drain_listener_frames(
    listener: &mut PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
) -> Vec<PeerGossipFrame> {
    let started = Instant::now();
    loop {
        let frames = listener
            .drain_inbox()
            .expect("listener live inbox drain should succeed");
        if !frames.is_empty() {
            return frames;
        }
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "listener did not receive frame within timeout"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_delivered_payload(frame: &PeerGossipFrame, sender_peer_id: &str, payload: &str) {
    assert_eq!(frame.sender_peer_id, sender_peer_id);
    assert_eq!(frame.topic, "messages");
    assert_eq!(frame.payload, payload);
}
