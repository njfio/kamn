use super::support::*;

#[test]
fn performance_libp2p_native_adapter_stays_within_local_heavy_budget() {
    let (sender_transport, recipient_transport) = performance_pair();
    advertise_messages(&sender_transport, "peer-native-perf-sender", NodeRole::Processor);
    advertise_messages(&recipient_transport, "peer-native-perf-recipient", NodeRole::Listener);
    warm_transport_path(&sender_transport, &recipient_transport);
    let started = Instant::now();
    send_performance_frames(&sender_transport);
    let frames = drain_until_count(
        &recipient_transport,
        "peer-native-perf-recipient",
        64,
        Duration::from_secs(2),
    );
    assert_eq!(frames.len(), 64);
    assert!(started.elapsed() <= Duration::from_secs(2), "libp2p native adapter exceeded local-heavy runtime budget");
}

fn performance_pair() -> (
    Libp2pLivePeerLifecycleTransport,
    Libp2pLivePeerLifecycleTransport,
) {
    let sender_listen = unique_listen_address();
    let recipient_listen = unique_listen_address();
    let bootstrap_peers = vec![sender_listen.clone(), recipient_listen.clone()];
    let sender = new_transport_with_bootstrap(
        "peer-native-perf-sender",
        sender_listen.as_str(),
        bootstrap_peers.clone(),
    );
    settle_mesh();
    let recipient = new_transport_with_bootstrap(
        "peer-native-perf-recipient",
        recipient_listen.as_str(),
        bootstrap_peers,
    );
    (sender, recipient)
}

fn warm_transport_path(
    sender_transport: &Libp2pLivePeerLifecycleTransport,
    recipient_transport: &Libp2pLivePeerLifecycleTransport,
) {
    let frame = message_frame(
        "peer-native-perf-sender",
        "peer-native-perf-recipient",
        "tx-native-performance-warmup",
    );
    send_with_retry(sender_transport, &frame, Duration::from_secs(5))
        .expect("warmup frame should send");
    let _ = drain_until_count(recipient_transport, "peer-native-perf-recipient", 1, Duration::from_secs(5));
}

fn send_performance_frames(sender_transport: &Libp2pLivePeerLifecycleTransport) {
    for nonce in 0..64 {
        let payload = format!("tx-native-performance-{nonce}");
        let frame = message_frame("peer-native-perf-sender", "peer-native-perf-recipient", payload.as_str());
        send_with_retry(sender_transport, &frame, Duration::from_secs(2))
            .expect("frame should send");
    }
}
