pub(super) use super::support::*;

pub(super) fn build_backpressure_pair(
    sender_peer_id: &str,
    recipient_peer_id: &str,
) -> (
    Libp2pLivePeerLifecycleTransport,
    Libp2pLivePeerLifecycleTransport,
) {
    let bootstrap_seed = unique_tcp_listen_address();
    let sender_transport = build_seeded_live_transport(sender_peer_id, bootstrap_seed.as_str());
    let recipient_transport =
        build_seeded_live_transport(recipient_peer_id, bootstrap_seed.as_str());
    advertise_messages_peer(&sender_transport, sender_peer_id, NodeRole::Processor);
    advertise_messages_peer(&recipient_transport, recipient_peer_id, NodeRole::Listener);
    (sender_transport, recipient_transport)
}

pub(super) fn assert_backpressure_reason(error: P2pTransportError, expected: &str) {
    assert_eq!(runtime_backpressure_reject(error), Some(expected));
}

pub(super) fn assert_behavior_failure_reason(
    transport: &Libp2pLivePeerLifecycleTransport,
    expected: &str,
) {
    let sender_events = transport
        .drain_runtime_events()
        .expect("sender runtime events should drain");
    assert!(sender_events.iter().any(|event| {
        event.kind() == Libp2pRuntimeEventKind::BehaviorFailure && event.reason_code() == expected
    }));
}

#[path = "backpressure_contract_tests/reason_codes_contract_tests.rs"]
mod reason_codes_contract_tests;
#[path = "backpressure_contract_tests/reject_saturated_inbox_contract_tests.rs"]
mod reject_saturated_inbox_contract_tests;
#[path = "backpressure_contract_tests/slow_producer_contract_tests.rs"]
mod slow_producer_contract_tests;
