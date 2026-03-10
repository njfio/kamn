use super::super::support::*;

#[test]
fn integration_libp2p_native_adapter_supports_discovery_and_gossip_over_sockets() {
    let (mut processor, mut listener) = build_connected_message_coordinators();
    assert_runtime_backend_marker();
    assert_discovery_converges(&mut processor);
    assert_single_broadcast_delivery(&mut processor, &mut listener);
}

fn build_connected_message_coordinators(
) -> (
    PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
    PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
) {
    let processor_listen = unique_listen_address();
    let listener_listen = unique_listen_address();
    let bootstrap_peers = vec![processor_listen.clone(), listener_listen.clone()];
    let processor_transport = new_transport_with_bootstrap(
        "peer-native-processor",
        processor_listen.as_str(),
        bootstrap_peers.clone(),
    );
    settle_mesh();
    let listener_transport = new_transport_with_bootstrap(
        "peer-native-listener",
        listener_listen.as_str(),
        bootstrap_peers,
    );
    (build_processor(processor_transport), build_listener(listener_transport))
}

fn build_processor(
    transport: Libp2pLivePeerLifecycleTransport,
) -> PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport> {
    assert_eq!(transport.transport_profile(), RuntimeTransportProfile::Libp2pLive);
    let mut processor = PeerLifecycleTransportCoordinator::new(
        "peer-native-processor",
        NodeRole::Processor,
        transport,
    )
    .expect("processor coordinator should initialize");
    assert_eq!(
        processor.connect_and_advertise(vec!["messages".to_owned()]),
        Ok(PeerLifecycleState::Active)
    );
    processor
}

fn build_listener(
    transport: Libp2pLivePeerLifecycleTransport,
) -> PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport> {
    let mut listener = PeerLifecycleTransportCoordinator::new(
        "peer-native-listener",
        NodeRole::Listener,
        transport,
    )
    .expect("listener coordinator should initialize");
    assert_eq!(
        listener.connect_and_advertise(vec!["messages".to_owned()]),
        Ok(PeerLifecycleState::Active)
    );
    listener
}

fn assert_runtime_backend_marker() {
    assert_eq!(resolve_libp2p_live_runtime_backend().marker(), "native-libp2p-swarm");
}

fn assert_discovery_converges(
    processor: &mut PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
) {
    let discovered = processor.discover("messages").expect("native discovery should succeed");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].peer_id, "peer-native-listener");
}

fn assert_single_broadcast_delivery(
    processor: &mut PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
    listener: &mut PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
) {
    assert_eq!(broadcast_with_retry(processor, "tx-native-001"), 1);
    let frame = wait_for_listener_frame(listener);
    assert_eq!(frame.sender_peer_id, "peer-native-processor");
    assert_eq!(frame.topic, "messages");
    assert_eq!(frame.payload, "tx-native-001");
}

fn broadcast_with_retry(
    processor: &mut PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
    payload: &str,
) -> usize {
    let started = Instant::now();
    loop {
        match processor.broadcast("messages", payload) {
            Ok(count) => return count,
            Err(P2pTransportError::LiveSocketSendFailed) if started.elapsed() < Duration::from_secs(5) => {
                std::thread::sleep(Duration::from_millis(25));
            }
            Err(error) => panic!("native broadcast should succeed: {error:?}"),
        }
    }
}

fn wait_for_listener_frame(
    listener: &mut PeerLifecycleTransportCoordinator<Libp2pLivePeerLifecycleTransport>,
) -> PeerGossipFrame {
    let started = Instant::now();
    loop {
        let frames = listener.drain_inbox().expect("listener inbox drain should succeed");
        if let Some(frame) = frames.into_iter().next() {
            return frame;
        }
        assert!(started.elapsed() < Duration::from_secs(5), "listener did not receive frame within timeout");
        std::thread::sleep(Duration::from_millis(25));
    }
}
