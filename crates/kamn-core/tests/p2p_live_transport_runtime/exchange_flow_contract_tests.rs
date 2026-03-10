use super::support::*;

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
