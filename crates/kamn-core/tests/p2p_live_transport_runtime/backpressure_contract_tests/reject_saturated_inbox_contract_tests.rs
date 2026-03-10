use super::*;

#[cfg(not(feature = "libp2p-live-transport"))]
#[test]
fn functional_live_transport_dispatch_backpressure_rejects_saturated_inbox() {
    let sender_peer_id = "peer-dispatch-backpressure-sender";
    let recipient_peer_id = "peer-dispatch-backpressure-recipient";
    let (sender_transport, _recipient_transport) =
        build_backpressure_pair(sender_peer_id, recipient_peer_id);
    let error = send_frames_until_error(
        &sender_transport,
        sender_peer_id,
        recipient_peer_id,
        "tx-backpressure",
        0..512,
    )
    .expect("dispatch should fail closed once inbox saturates");
    assert_backpressure_reason(error, "runtime_backpressure_reject_new_enqueue");
}
