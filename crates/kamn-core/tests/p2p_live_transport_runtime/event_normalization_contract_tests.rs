use super::support::*;

#[test]
fn functional_live_transport_emits_normalized_runtime_events() {
    let (processor_transport, listener_transport) = build_event_transport_pair();
    advertise_event_pair(&processor_transport, &listener_transport);
    assert_listener_discovered(&processor_transport);
    send_event_frame(&processor_transport);
    let _frames = drain_until_count(
        &listener_transport,
        "peer-listener-live-events",
        1,
        Duration::from_secs(5),
    );
    let events = drain_runtime_events_until(&processor_transport, 5, Duration::from_secs(5));
    assert_runtime_event_kinds(&events);
    assert_runtime_event_schema(&events);
}

fn build_event_transport_pair() -> (
    Libp2pLivePeerLifecycleTransport,
    Libp2pLivePeerLifecycleTransport,
) {
    let processor_listen = unique_tcp_listen_address();
    let listener_listen = unique_tcp_listen_address();
    let bootstrap_peers = vec![processor_listen.clone(), listener_listen.clone()];
    let processor_transport = build_live_transport(
        "peer-processor-live-events",
        processor_listen.as_str(),
        bootstrap_peers.clone(),
    );
    pause_for_live_handshake();
    let listener_transport = build_live_transport(
        "peer-listener-live-events",
        listener_listen.as_str(),
        bootstrap_peers,
    );
    (processor_transport, listener_transport)
}

fn advertise_event_pair(
    processor_transport: &Libp2pLivePeerLifecycleTransport,
    listener_transport: &Libp2pLivePeerLifecycleTransport,
) {
    advertise_messages_peer(
        processor_transport,
        "peer-processor-live-events",
        NodeRole::Processor,
    );
    advertise_messages_peer(
        listener_transport,
        "peer-listener-live-events",
        NodeRole::Listener,
    );
}

fn assert_listener_discovered(processor_transport: &Libp2pLivePeerLifecycleTransport) {
    let discovered = processor_transport
        .discover("peer-processor-live-events", "messages")
        .expect("discovery should pass");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].peer_id, "peer-listener-live-events");
}

fn send_event_frame(processor_transport: &Libp2pLivePeerLifecycleTransport) {
    let frame = PeerGossipFrame::new(
        "messages",
        "peer-processor-live-events",
        "peer-listener-live-events",
        "tx-live-events-001",
    )
    .expect("gossip frame should build");
    send_with_retry(processor_transport, &frame, Duration::from_secs(5))
        .expect("gossip send should pass");
}

fn assert_runtime_event_kinds(events: &[kamn_core::Libp2pRuntimeEvent]) {
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
}

fn assert_runtime_event_schema(events: &[kamn_core::Libp2pRuntimeEvent]) {
    assert!(events
        .iter()
        .all(|event| event.schema_marker() == "kamn.libp2p.runtime-event.v1"));
}
