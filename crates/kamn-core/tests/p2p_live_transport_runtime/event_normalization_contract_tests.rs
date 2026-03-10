use super::support::*;

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
