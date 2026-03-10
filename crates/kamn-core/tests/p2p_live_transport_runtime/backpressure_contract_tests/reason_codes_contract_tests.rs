use super::*;

#[cfg(not(feature = "libp2p-live-transport"))]
#[test]
fn regression_live_transport_dispatch_backpressure_reason_codes_stay_stable() {
    let sender_peer_id = "peer-dispatch-reason-sender";
    let recipient_peer_id = "peer-dispatch-reason-recipient";
    let (sender_transport, _recipient_transport) =
        build_backpressure_pair(sender_peer_id, recipient_peer_id);
    let error = send_frames_until_error(
        &sender_transport,
        sender_peer_id,
        recipient_peer_id,
        "tx-reason",
        0..512,
    )
    .expect("expected dispatch backpressure rejection");
    assert_backpressure_reason(error, "runtime_backpressure_reject_new_enqueue");
    assert_behavior_failure_reason(&sender_transport, "runtime_backpressure_reject_new_enqueue");
}
