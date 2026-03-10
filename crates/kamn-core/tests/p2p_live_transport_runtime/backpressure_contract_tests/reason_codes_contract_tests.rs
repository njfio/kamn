use super::*;

#[cfg(not(feature = "libp2p-live-transport"))]
#[test]
fn regression_live_transport_dispatch_backpressure_reason_codes_stay_stable() {
    let bootstrap_seed = unique_tcp_listen_address();
    let sender_peer_id = "peer-dispatch-reason-sender";
    let recipient_peer_id = "peer-dispatch-reason-recipient";
    let sender_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            sender_peer_id,
            unique_tcp_listen_address().as_str(),
            bootstrap_seed.as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("sender transport should initialize");
    let recipient_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            recipient_peer_id,
            unique_tcp_listen_address().as_str(),
            bootstrap_seed.as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("recipient transport should initialize");

    sender_transport
        .advertise(
            PeerDiscoveryRecord::new(
                sender_peer_id,
                NodeRole::Processor,
                vec!["messages".to_owned()],
            )
            .expect("sender discovery record should build"),
        )
        .expect("sender advertise should pass");
    recipient_transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient_peer_id,
                NodeRole::Listener,
                vec!["messages".to_owned()],
            )
            .expect("recipient discovery record should build"),
        )
        .expect("recipient advertise should pass");

    let mut saw_reject = false;
    for nonce in 0..512 {
        let frame = PeerGossipFrame::new(
            "messages",
            sender_peer_id,
            recipient_peer_id,
            format!("tx-reason-{nonce}").as_str(),
        )
        .expect("frame should build");
        match sender_transport.send(frame) {
            Ok(()) => {}
            Err(error) => {
                assert_eq!(
                    error.reason_code(),
                    "runtime_backpressure_reject_new_enqueue"
                );
                saw_reject = true;
                break;
            }
        }
    }
    assert!(saw_reject, "expected dispatch backpressure rejection");

    let sender_events = sender_transport
        .drain_runtime_events()
        .expect("sender runtime events should drain");
    assert!(
        sender_events.iter().any(|event| {
            event.kind() == Libp2pRuntimeEventKind::BehaviorFailure
                && event.reason_code() == "runtime_backpressure_reject_new_enqueue"
        }),
        "expected behavior-failure event for dispatch backpressure rejection"
    );
}
