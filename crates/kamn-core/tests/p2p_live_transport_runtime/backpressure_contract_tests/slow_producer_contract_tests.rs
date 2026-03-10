use super::*;

#[cfg(not(feature = "libp2p-live-transport"))]
#[test]
fn functional_live_transport_dispatch_slow_producer_suspend_alias_stays_fail_closed() {
    let sender_peer_id = "peer-dispatch-slow-sender";
    let recipient_peer_id = "peer-dispatch-slow-recipient";
    let (sender_transport, _recipient_transport) =
        build_backpressure_pair(sender_peer_id, recipient_peer_id);
    send_frames_expect_success(
        &sender_transport,
        sender_peer_id,
        recipient_peer_id,
        "tx-suspend-alias",
        0..100,
    );
    let error = send_frames_until_error(
        &sender_transport,
        sender_peer_id,
        recipient_peer_id,
        "tx-suspend-alias",
        100..512,
    )
    .expect("expected fail-closed reject beyond slow-producer range");
    assert_backpressure_reason(error, "runtime_backpressure_reject_new_enqueue");
}
